# 11 – Gotchas, FOSS Best Practices, and Things Nobody Told You

## Cargo Sources Must Stay in Sync

The most common Flatpak breakage for Rust projects: you bump a dependency or run `cargo update`,
`Cargo.lock` changes, but `cargo-sources.json` is stale. The build then fails with a cryptic
"package not found" or network error inside the sandbox.

**Fix:** Make regenerating `cargo-sources.json` part of the process any time `Cargo.lock`
changes. A `just flatpak-gen-sources` recipe (see doc 08) makes this easy. Consider adding a CI
check that detects a stale `cargo-sources.json` (diff it after regenerating and fail if changed).

---

## The Offline Build Constraint is Strict

`flatpak-builder` intercepts network calls inside the sandbox. Any `build-command` that tries
to reach the internet will silently get an empty response or hang. This includes:

- `cargo build` fetching crates → mitigated by `cargo-sources.json`
- `cargo install` inside the build → do not run `cargo install` in the manifest
- `wasm-pack` / `npm` / any other package manager → do not call these in the Flatpak manifest
- The `upscale_assets.sh` script → must be run **before** building the Flatpak; pre-built assets
  must be in the git source or supplied as a separate source

For the upscaled assets specifically, the options are:
1. Run `upscale_assets.sh` in CI before building the Flatpak and commit the results to a release
   branch / tag that Flatpak builds from.
2. Bundle the upscaler tools (xbrzscale, imagemagick) as extra sources in the manifest — but
   this is complex.
3. Decide that Flatpak builds use un-upscaled assets (simpler, lower quality).

Option 1 is the standard approach.

---

## Working Directory and Asset Path Resolution

Bevy's asset loader uses relative paths (e.g., `assets/maps/foo.tmx`). The binary's working
directory must be set to the directory that contains `assets/`. Without the wrapper script
setting `cd /app/share/unhaunter`, the game will fail to find any asset and crash silently.

Always test with `flatpak run io.github.deavid.unhaunter` from a terminal to see startup errors.

---

## GPU / Vulkan Blacklist Issues

Some Mesa driver versions have known bugs with certain Vulkan features. Bevy uses Vulkan by
default but falls back to OpenGL if Vulkan init fails. In Flatpak:

- Mesa is provided by the runtime, not the host. The runtime version (24.08) has a fixed Mesa.
  This is usually more up-to-date than what older distros ship, which is a **feature** – it
  avoids host driver bugs on distros like Ubuntu LTS.
- If the game needs to force OpenGL (e.g. for compatibility), add to `finish-args`:
  `--env=WGPU_BACKEND=gl`

---

## Wayland vs X11

The manifest should include both `--socket=x11` and `--socket=wayland`. Bevy/wgpu will use
Wayland when available (most modern setups) and fall back to X11 via XWayland. Including both
sockets ensures the app works regardless of the user's display server.

For pure Wayland support, you may also need:
```yaml
  - --env=DISPLAY=          # unset DISPLAY to force Wayland (optional, may break some setups)
```
Avoid this until Bevy's Wayland support is well-tested for the game.

---

## The 3-Day Delay on Flathub Updates

Flathub enforces a 3-day "verification window" after a new build is produced before it propagates
to all users. This is a safety net — if you push a broken update you have time to push a fix
before everyone receives the bad build. Plan releases accordingly: pushing to Flathub 3 days
before a public announcement is a good habit.

This delay does **not** apply to your self-hosted alpha repo.

---

## Initial Review vs Ongoing Updates

- **First submission:** full manual review by Flathub maintainers. They check metadata,
  permissions, build success, and that the app actually runs. This happens once.
- **Subsequent updates:** automated. You push to the Flathub app repo; their CI builds it and
  publishes it. No human review. However, automated checks still run (lint, build validation).
  If a build fails, you get a notification and the old version stays live.

There is no audit/review requirement for the self-hosted alpha repo.

---

## `cargo deny` for License and Vulnerability Auditing

Before submitting to Flathub, run:

```bash
cargo deny check licenses
cargo deny check advisories
```

This ensures:
- All Cargo dependencies have FOSS-compatible licenses (no GPL v2-only deps that conflict with
  MIT/Apache-2.0 redistribution).
- No known CVEs in the dependency tree.

Flathub does not currently run this automatically, but it is good practice and aligns with FOSS
culture.

---

## Binary Stripping

The release binary should be stripped of debug symbols to reduce download size. `cargo build
--release` applies some optimisations, but explicit stripping helps:

```bash
strip target/x86_64-unknown-linux-gnu/release/unhaunter_game
```

Or add to `Cargo.toml` (workspace):

```toml
[profile.release]
strip = true       # strip debug symbols from the binary
lto = true         # link-time optimisation (slower build, smaller/faster binary)
codegen-units = 1  # better optimisation
```

These settings are already common in Bevy game projects and can meaningfully reduce binary size
from ~100 MB down to ~30–50 MB.

---

## Single-File `.flatpak` Bundle (for Itch.io or GitHub Releases)

`flatpak build-bundle` produces a self-contained `.flatpak` file that users can install without
adding a remote:

```bash
flatpak install --user unhaunter-0.X.X-linux-x86_64.flatpak
```

This is a great complement to the GitHub Release tarball: upload both the tarball **and** the
`.flatpak` bundle to GitHub Releases. Users who prefer Flatpak over manual extraction get a
first-class option. They do not get auto-updates this way, but it is zero-friction for trying
the game.

---

## Delta Updates

OSTree (the storage layer) supports "static deltas": pre-computed binary diffs between versions
that dramatically reduce update download size. Enable them with:

```bash
flatpak build-update-repo --generate-static-deltas repo/
```

Flathub generates deltas automatically. For the self-hosted alpha repo, add this flag to the
Justfile recipe (already included in the suggested `flatpak-export-repo` recipe in doc 08).

---

## Flathub Verified Badge

Once your app is on Flathub and you own the `io.github.deavid.*` namespace (which is verified
because it corresponds to your GitHub account), Flathub will show a "Verified" badge next to
the app. This increases trust and discoverability in software centres.

---

## itch.io + Flatpak = Good Combination

The existing itch.io presence (via GitHub Releases assets) and a Flatpak listing are
complementary, not competing:

- itch.io users who discover Unhaunter there can download the `.tar.gz` or `.flatpak` bundle.
- Flathub / software centre users who had never heard of Unhaunter can find it there.
- The Flatpak on Flathub also lists the homepage URL, which points back to GitHub / itch.io.

A mature FOSS game distribution pattern: GitHub Releases + Flathub + itch.io, all pointing at
the same upstream release tag. The three audiences have low overlap, so all three channels
add reach.

---

## FOSS Alignment Notes

- Flathub's FOSS track is genuinely free (no fees, no percentage cut).
- The Flathub submission process is public and transparent (GitHub PRs, public CI logs).
- Submitting to Flathub does not require transferring any rights or exclusivity.
- The app remains 100% under your MIT/Apache-2.0 license; Flathub just re-distributes it.
- Keeping the Flathub app repo public (`github.com/flathub/io.github.deavid.unhaunter`) means
  anyone can see how the build works and file issues. This is standard FOSS practice.
