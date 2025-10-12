# Reference Module Notes

## Implementation Decisions

1.  **Use `oci-spec` Crate**:
    - **Rationale**: To ensure compliance with the OCI specification for image references, we are using the `Reference` type from the `oci-spec` crate. This avoids the need to write and maintain complex parsing and validation logic, which is brittle and error-prone.
    - **Implementation**: The `reference` module in `librex` acts as a thin wrapper around `oci_spec::image::Reference`. Its primary roles are to provide a consistent API within `rex`, integrate with our custom `RexError` type, and expose the necessary components of a reference (registry, repository, tag/digest).
