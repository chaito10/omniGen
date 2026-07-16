//! Pipeline execution engine.
//!
//! Reads YAML pipeline definitions, builds an execution graph,
//! and orchestrates node execution with progress tracking.

use crate::config::{Config, PipelineStep};
use crate::graph::{ExecutionGraph, GraphNode};
use crate::model::ModelRegistry;
use crate::nodes;
use crate::traits::NodeOutputs;
use crate::scheduler::Scheduler;
use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// The pipeline executor.
pub struct Pipeline {
    /// The execution graph.
    graph: ExecutionGraph,
    /// Model registry reference.
    _registry: Arc<ModelRegistry>,
    /// Multi-progress bar for tracking.
    _progress: MultiProgress,
}

impl Pipeline {
    /// Build a pipeline from a YAML configuration file.
    pub fn from_config(config: &Config, registry: Arc<ModelRegistry>) -> Result<Self> {
        let mut graph = ExecutionGraph::new();
        let mut step_counter = 0;

        for (i, step) in config.pipeline.iter().enumerate() {
            let nodes = build_nodes_from_step(step, i, &mut step_counter)?;
            for node in nodes {
                graph.add_node(node)?;
            }
        }

        // Connect sequential dependencies
        connect_sequential_nodes(&mut graph)?;

        let progress = MultiProgress::new();

        Ok(Self {
            graph,
            _registry: registry,
            _progress: progress,
        })
    }

    /// Build a pipeline from a single command (image, video, etc.).
    pub fn from_command(
        steps: Vec<PipelineStep>,
        registry: Arc<ModelRegistry>,
    ) -> Result<Self> {
        let config = Config {
            pipeline: steps,
            ..Config::default_config()
        };
        Self::from_config(&config, registry)
    }

    /// Execute the pipeline.
    pub fn execute(&self) -> Result<HashMap<String, NodeOutputs>> {
        info!(
            "executing pipeline: {} nodes, {} edges",
            self.graph.node_count(),
            self.graph.edge_count()
        );

        let order = self
            .graph
            .resolve_execution_order()
            .context("failed to resolve pipeline execution order")?;

        let total_levels = order.len();
        let pb = self._progress.add(ProgressBar::new(total_levels as u64));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .expect("invalid progress template"),
        );
        pb.set_message("executing pipeline");

        let mut outputs_cache: HashMap<String, NodeOutputs> = HashMap::new();

        for (level_idx, level) in order.iter().enumerate() {
            debug!("executing level {}/{}: {:?}", level_idx + 1, total_levels, level);

            let _scheduler = Scheduler::new();

            for node_id in level {
                let inputs = self.graph.collect_node_inputs(node_id, &outputs_cache)?;
                let node = self.graph.get_node(node_id)?;

                let outputs = node.node.execute(&inputs)?;
                outputs_cache.insert(node_id.clone(), outputs);
            }

            pb.inc(1);
        }

        pb.finish_with_message("pipeline complete");
        info!("pipeline execution complete");

        Ok(outputs_cache)
    }

    /// Get a reference to the execution graph.
    pub fn graph(&self) -> &ExecutionGraph {
        &self.graph
    }
}

/// Build graph nodes from a pipeline step.
fn build_nodes_from_step(
    step: &PipelineStep,
    _step_index: usize,
    counter: &mut usize,
) -> Result<Vec<GraphNode>> {
    let mut nodes_out = Vec::new();

    if let Some(ref prompt_step) = step.prompt {
        let id = format!("prompt_{}", *counter);
        *counter += 1;
        nodes_out.push(GraphNode {
            id: id.clone(),
            node: Box::new(nodes::prompt::PromptNode::new(id, prompt_step.text.clone())),
            inputs: vec![],
            outputs: vec!["text".to_string()],
        });
    }

    if let Some(ref text_step) = step.text {
        let id = format!("text_{}", *counter);
        *counter += 1;
        nodes_out.push(GraphNode {
            id: id.clone(),
            node: Box::new(nodes::text::TextNode::new(
                id,
                text_step.model.clone(),
                text_step.max_tokens,
                text_step.temperature,
            )),
            inputs: vec![("prompt".to_string(), "prompt_0".to_string(), "text".to_string())],
            outputs: vec!["text".to_string()],
        });
    }

    if let Some(ref image_step) = step.image {
        let id = format!("image_{}", *counter);
        *counter += 1;
        nodes_out.push(GraphNode {
            id: id.clone(),
            node: Box::new(nodes::image::ImageNode::new(
                id,
                image_step.model.clone(),
                image_step.width,
                image_step.height,
                image_step.steps,
            )),
            inputs: vec![("prompt".to_string(), "prompt_0".to_string(), "text".to_string())],
            outputs: vec!["image".to_string()],
        });
    }

    if let Some(ref video_step) = step.video {
        let id = format!("video_{}", *counter);
        *counter += 1;
        nodes_out.push(GraphNode {
            id: id.clone(),
            node: Box::new(nodes::video::VideoNode::new(
                id,
                video_step.model.clone(),
                video_step.frames,
                video_step.width,
                video_step.height,
                video_step.steps,
            )),
            inputs: vec![("prompt".to_string(), "prompt_0".to_string(), "text".to_string())],
            outputs: vec!["video".to_string()],
        });
    }

    if let Some(ref audio_step) = step.audio {
        let id = format!("audio_{}", *counter);
        *counter += 1;
        nodes_out.push(GraphNode {
            id: id.clone(),
            node: Box::new(nodes::audio::AudioNode::new(
                id,
                audio_step.model.clone(),
                audio_step.input.clone(),
                audio_step.language.clone(),
            )),
            inputs: vec![],
            outputs: vec!["text".to_string()],
        });
    }

    if let Some(ref save_step) = step.save {
        let id = format!("save_{}", *counter);
        *counter += 1;
        nodes_out.push(GraphNode {
            id: id.clone(),
            node: Box::new(nodes::save::SaveNode::new(
                id,
                save_step.path.clone(),
                save_step.format.clone(),
            )),
            inputs: vec![],
            outputs: vec![],
        });
    }

    Ok(nodes_out)
}

/// Connect sequential nodes in the graph based on naming conventions.
fn connect_sequential_nodes(graph: &mut ExecutionGraph) -> Result<()> {
    let node_ids = graph.node_ids();

    // Connect prompt nodes to their consumers
    for i in 1..node_ids.len() {
        let prev = &node_ids[i - 1];
        let curr = &node_ids[i];
        if let Ok(node) = graph.get_node(curr) {
            if node.inputs.is_empty() {
                if let Ok(prev_node) = graph.get_node(prev) {
                    if !prev_node.outputs.is_empty() {
                        graph.add_edge(prev, curr)?;
                    }
                }
            }
        }
    }

    Ok(())
}
