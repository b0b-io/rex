# OCI Module Notes

## Implementation Decisions

1.  **Re-export from `oci-spec`**:
    - **Rationale**: The primary purpose of this module is to act as a facade for the OCI data types provided by the `oci-spec` crate. Instead of implementing our own structs, we are re-exporting the battle-tested, spec-compliant types directly from the library.
    - **Benefits**: This approach keeps our internal API clean. Other modules within `librex` can now depend on `librex::oci` instead of directly on `oci-spec`. This reduces coupling to the external dependency and gives us a single place to manage which OCI types are used throughout the application. If we ever needed to swap out the `oci-spec` crate or add custom logic to a type, we could do so here with minimal disruption to the rest of the codebase.
