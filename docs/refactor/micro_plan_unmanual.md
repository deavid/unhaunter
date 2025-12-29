# Micro-Plan: Rename `unmanual` to `unmanual-plugin`

## Objective
Rename the `unmanual` crate to `unmanual-plugin` to enforce the architectural policy for leaf logic crates. This is a "trivial" slice of the larger refactoring plan.

## Scope
- **Crate**: `crates/unmanual`
- **Type**: Leaf Plugin (Logic only)
- **Dependencies**: Depends on core crates.
- **Dependents**: Only the root `unhaunter` app.

## Steps
1.  **Rename Directory**:
    - Move `crates/unmanual` to `crates/unmanual-plugin`.
2.  **Update Crate Manifest** (`crates/unmanual-plugin/Cargo.toml`):
    - Change `name = "unmanual"` to `name = "unmanual-plugin"`.
3.  **Update Root Manifest** (`Cargo.toml`):
    - Update `workspace.members`: Change `"crates/unmanual"` to `"crates/unmanual-plugin"`.
    - Update `dependencies`: Change `unmanual = { path = "crates/unmanual" }` to `unmanual-plugin = { path = "crates/unmanual-plugin" }`.
4.  **Update Code** (`unhaunter/src/app.rs`):
    - Change `use unmanual::plugin::UnhaunterManualPlugin;` to `use unmanual_plugin::plugin::UnhaunterManualPlugin;`.

## Verification
- Run `cargo check` to ensure the workspace compiles.
