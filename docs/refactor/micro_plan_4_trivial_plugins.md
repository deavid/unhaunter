# Micro-Plan: Rename 4 Trivial Plugins

## Objective
Rename 4 "trivial" leaf crates to `*-plugin` to enforce the architectural policy. These crates have no internal dependencies within the `crates/` folder (except the root app).

## Scope
The following crates will be renamed:
1.  `crates/uncampaign` -> `crates/uncampaign-plugin`
2.  `crates/unmenu` -> `crates/unmenu-plugin`
3.  `crates/unnpc` -> `crates/unnpc-plugin`
4.  `crates/untmxmap` -> `crates/untmxmap-plugin`

## Steps

### 1. `uncampaign`
1.  **Rename Directory**: `crates/uncampaign` -> `crates/uncampaign-plugin`
2.  **Update Manifest**: `crates/uncampaign-plugin/Cargo.toml` -> `name = "uncampaign-plugin"`
3.  **Update Root Manifest**:
    - `workspace.members`: `"crates/uncampaign"` -> `"crates/uncampaign-plugin"`
    - `dependencies`: `uncampaign = { path = "crates/uncampaign" }` -> `uncampaign-plugin = { path = "crates/uncampaign-plugin" }`
4.  **Update Code**: `unhaunter/src/app.rs` -> `use uncampaign_plugin::plugin::UnhaunterCampaignPlugin;`

### 2. `unmenu`
1.  **Rename Directory**: `crates/unmenu` -> `crates/unmenu-plugin`
2.  **Update Manifest**: `crates/unmenu-plugin/Cargo.toml` -> `name = "unmenu-plugin"`
3.  **Update Root Manifest**:
    - `workspace.members`: `"crates/unmenu"` -> `"crates/unmenu-plugin"`
    - `dependencies`: `unmenu = { path = "crates/unmenu" }` -> `unmenu-plugin = { path = "crates/unmenu-plugin" }`
4.  **Update Code**: `unhaunter/src/app.rs` -> `use unmenu_plugin::plugin::UnhaunterMenuPlugin;`

### 3. `unnpc`
1.  **Rename Directory**: `crates/unnpc` -> `crates/unnpc-plugin`
2.  **Update Manifest**: `crates/unnpc-plugin/Cargo.toml` -> `name = "unnpc-plugin"`
3.  **Update Root Manifest**:
    - `workspace.members`: `"crates/unnpc"` -> `"crates/unnpc-plugin"`
    - `dependencies`: `unnpc = { path = "crates/unnpc" }` -> `unnpc-plugin = { path = "crates/unnpc-plugin" }`
4.  **Update Code**: `unhaunter/src/app.rs` -> `use unnpc_plugin::plugin::UnhaunterNPCPlugin;`

### 4. `untmxmap`
1.  **Rename Directory**: `crates/untmxmap` -> `crates/untmxmap-plugin`
2.  **Update Manifest**: `crates/untmxmap-plugin/Cargo.toml` -> `name = "untmxmap-plugin"`
3.  **Update Root Manifest**:
    - `workspace.members`: `"crates/untmxmap"` -> `"crates/untmxmap-plugin"`
    - `dependencies`: `untmxmap = { path = "crates/untmxmap" }` -> `untmxmap-plugin = { path = "crates/untmxmap-plugin" }`
4.  **Update Code**: `unhaunter/src/app.rs` -> `use untmxmap_plugin::plugin::UnhaunterTmxMapPlugin;`

## Verification
- Run `cargo check` to ensure the workspace compiles.
