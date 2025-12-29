# Micro-Plan: Rename 5 More Trivial Plugins

## Objective
Rename 5 more leaf logic crates to `*-plugin`.

## Scope
1.  `unmenusettings` -> `unmenusettings-plugin`
2.  `unroot` -> `unroot-plugin`
3.  `unwalkie` -> `unwalkie-plugin`
4.  `ungame` -> `ungame-plugin`
5.  `unmapload` -> `unmapload-plugin`

## Steps

### 1. Update Crate Manifests (Internal Name)
- `crates/unmenusettings/Cargo.toml`: `name = "unmenusettings-plugin"`
- `crates/unroot/Cargo.toml`: `name = "unroot-plugin"`
- `crates/unwalkie/Cargo.toml`: `name = "unwalkie-plugin"`
- `crates/ungame/Cargo.toml`: `name = "ungame-plugin"`
- `crates/unmapload/Cargo.toml`: `name = "unmapload-plugin"`

### 2. Update Root Manifest (`Cargo.toml`)
- Update `workspace.members` to use `-plugin` suffix and paths.
- Update `dependencies` to use `-plugin` suffix and paths.

### 3. Update Code (`unhaunter/src/app.rs`)
- Update `use` statements to use the new crate names (with underscores).

## Note
The user will move the directories manually after these changes.
