# Contributing

Contributions to OmniGen are welcome.

## Development Setup

```bash
git clone https://github.com/user/OmniGen.git
cd OmniGen
cargo build
cargo test
```

## Code Style

- Follow `cargo fmt` formatting
- Pass `cargo clippy` without warnings
- Document all public items
- No `unwrap()` in production code
- Use proper error handling with `anyhow`/`thiserror`

## Adding a New Model

1. Create `src/models/your_model.rs`
2. Implement the `Model` trait
3. Register in `src/models/mod.rs`
4. Add to the model parser in `src/main.rs`
5. Add CLI support in `src/cli.rs`

## Testing

```bash
cargo test               # Run all tests
cargo test -- --nocapture  # Show test output
cargo clippy              # Lint
cargo fmt --check        # Check formatting
```

## Pull Request Process

1. Create a feature branch
2. Write tests for new functionality
3. Ensure all tests pass
4. Update documentation if needed
5. Submit pull request
