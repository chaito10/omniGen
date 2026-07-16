//! DAG (Directed Acyclic Graph) for pipeline execution.
//!
//! Builds a dependency graph from YAML pipeline definitions and
//! resolves execution order for parallel and sequential node execution.

use crate::traits::{Node, NodeInputs, NodeOutputs};
use anyhow::Result;
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// A node in the execution graph with its type and configuration.
pub struct GraphNode {
    /// Unique identifier for this node.
    pub id: String,
    /// The actual node implementation.
    pub node: Box<dyn Node>,
    /// Input port mappings: (local_port, source_node_id, source_port).
    pub inputs: Vec<(String, String, String)>,
    /// Output port names.
    pub outputs: Vec<String>,
}

impl std::fmt::Debug for GraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphNode")
            .field("id", &self.id)
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .finish()
    }
}

/// The execution graph built from a pipeline definition.
pub struct ExecutionGraph {
    /// The underlying petgraph directed graph.
    graph: DiGraph<String, ()>,
    /// Nodes indexed by their petgraph index.
    nodes: HashMap<NodeIndex, GraphNode>,
    /// Nodes indexed by their string ID.
    id_index: HashMap<String, NodeIndex>,
}

impl std::fmt::Debug for ExecutionGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionGraph")
            .field("node_count", &self.nodes.len())
            .field("edge_count", &self.graph.edge_count())
            .finish()
    }
}

impl ExecutionGraph {
    /// Create a new empty execution graph.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            nodes: HashMap::new(),
            id_index: HashMap::new(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, graph_node: GraphNode) -> Result<()> {
        let id = graph_node.id.clone();
        let idx = self.graph.add_node(id.clone());
        self.nodes.insert(idx, graph_node);
        self.id_index.insert(id, idx);
        Ok(())
    }

    /// Add an edge (dependency) between two nodes.
    pub fn add_edge(&mut self, from_id: &str, to_id: &str) -> Result<()> {
        let from_idx = self
            .id_index
            .get(from_id)
            .ok_or_else(|| anyhow::anyhow!("source node not found: {}", from_id))?;
        let to_idx = self
            .id_index
            .get(to_id)
            .ok_or_else(|| anyhow::anyhow!("target node not found: {}", to_id))?;
        self.graph.add_edge(*from_idx, *to_idx, ());
        Ok(())
    }

    /// Resolve execution order via topological sort.
    ///
    /// Returns nodes in execution order, with independent nodes
    /// that can run in parallel grouped together in each level.
    pub fn resolve_execution_order(&self) -> Result<Vec<Vec<String>>> {
        let sorted = toposort(&self.graph, None)
            .map_err(|_| anyhow::anyhow!("cycle detected in pipeline graph"))?;

        // Group into parallel levels
        let mut levels: Vec<Vec<String>> = Vec::new();
        let mut node_level: HashMap<NodeIndex, usize> = HashMap::new();

        for &idx in &sorted {
            let level = if self
                .graph
                .neighbors_directed(idx, petgraph::Direction::Incoming)
                .count()
                == 0
            {
                0
            } else {
                let max_parent_level = self
                    .graph
                    .neighbors_directed(idx, petgraph::Direction::Incoming)
                    .map(|parent| node_level[&parent])
                    .max()
                    .unwrap_or(0);
                max_parent_level + 1
            };

            node_level.insert(idx, level);

            if levels.len() <= level {
                levels.resize(level + 1, Vec::new());
            }
            levels[level].push(self.nodes[&idx].id.clone());
        }

        Ok(levels)
    }

    /// Get a node by its ID.
    pub fn get_node(&self, id: &str) -> Result<&GraphNode> {
        let idx = self
            .id_index
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("node not found: {}", id))?;
        Ok(&self.nodes[idx])
    }

    /// Get the number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Get all node IDs in graph order.
    pub fn node_ids(&self) -> Vec<String> {
        self.graph
            .node_indices()
            .filter_map(|idx| self.nodes.get(&idx).map(|n| n.id.clone()))
            .collect()
    }

    /// Collect outputs from a node by ID.
    pub fn collect_node_inputs(
        &self,
        node_id: &str,
        outputs_cache: &HashMap<String, NodeOutputs>,
    ) -> Result<NodeInputs> {
        let node = self.get_node(node_id)?;
        let mut inputs = NodeInputs::new();

        for (local_port, source_node, source_port) in &node.inputs {
            if let Some(source_outputs) = outputs_cache.get(source_node) {
                if let Some(tensor) = source_outputs.get(source_port) {
                    inputs.push(local_port.clone(), tensor.clone());
                }
            }
        }

        Ok(inputs)
    }
}

impl Default for ExecutionGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyNode {
        id: String,
    }

    impl Node for DummyNode {
        fn id(&self) -> &str {
            &self.id
        }
        fn node_type(&self) -> &str {
            "dummy"
        }
        fn execute(&self, _inputs: &NodeInputs) -> Result<NodeOutputs> {
            Ok(NodeOutputs::new())
        }
        fn input_ports(&self) -> Vec<&str> {
            vec![]
        }
        fn output_ports(&self) -> Vec<&str> {
            vec![]
        }
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = ExecutionGraph::new();

        graph
            .add_node(GraphNode {
                id: "a".into(),
                node: Box::new(DummyNode { id: "a".into() }),
                inputs: vec![],
                outputs: vec![],
            })
            .unwrap();

        graph
            .add_node(GraphNode {
                id: "b".into(),
                node: Box::new(DummyNode { id: "b".into() }),
                inputs: vec![],
                outputs: vec![],
            })
            .unwrap();

        graph.add_edge("a", "b").unwrap();

        let order = graph.resolve_execution_order().unwrap();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], vec!["a"]);
        assert_eq!(order[1], vec!["b"]);
    }
}
