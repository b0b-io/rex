# Digest Module Notes

## Implementation Decisions

1.  **Use `oci-spec` Crate**:
    - **Rationale**: Instead of implementing our own digest parsing and validation logic, we are using the `Digest` type directly from the `oci-spec` crate. This ensures that our implementation is compliant with the OCI specification, reduces our maintenance burden, and allows us to leverage a well-tested, community-standard library.
    - **Implementation**: The `digest` module in `librex` will act as a thin wrapper. It will expose functionality from `oci_spec::image::Digest` and translate any errors into our internal `RexError` type for consistency within the application.
