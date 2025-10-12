# Config Module Notes

## Implementation Decisions

1.  **Configuration Format**:
    - **Decision**: Use YAML (`config.yaml`).
    - **Rationale**: YAML's syntax is often considered more human-readable for the nested structures our configuration requires.

2.  **Refactor to `config-rs` Crate**:
    - **Initial Implementation**: The module was first implemented with a manual loading function using `serde_yaml` and the `directories` crate.
    - **Decision**: We have refactored the module to use the `config-rs` crate.
    - **Rationale**: This decision was made because `config-rs` provides a robust mechanism for merging multiple configuration sources (files, environment variables, etc.), which is a core requirement from our `design.md`. Using a dedicated, mature library aligns better with our "Don't Reinvent the Wheel" principle and will simplify future development, particularly when adding support for environment variable overrides. The structs we defined with `serde` are fully compatible with this new approach.

## Dependencies

- `config-rs`: The core library for handling layered configuration.
- `serde`: Used for deserializing the configuration into our Rust structs.
