# Config Module Notes

## Implementation Decisions

1.  **Configuration Format**:
    - **Decision**: Use YAML (`config.yaml`) instead of TOML.
    - **Rationale**: YAML's syntax is often considered more human-readable for nested structures, which our configuration will have (e.g., for registry lists, cache settings). It is also a very popular choice for configuration in the cloud-native ecosystem.

2.  **Dependencies**:
    - `serde` and `serde_yaml`: These are the standard, community-approved crates for serializing and deserializing YAML in Rust.
    - `directories`: This crate provides a reliable, cross-platform way to locate the user's configuration directory (e.g., `~/.config` on Linux, `%APPDATA%` on Windows), which is essential for finding our `config.yaml` file.

3.  **Structure**:
    - The configuration is represented by a set of structs that can be directly deserialized from the YAML file using `serde`. This is a robust and type-safe approach.
    - A `Config::load()` method will be implemented to handle finding and parsing the configuration file, with a `Config::default()` fallback if the file doesn't exist.
