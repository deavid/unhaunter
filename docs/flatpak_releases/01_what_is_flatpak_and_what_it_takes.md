# 01 – What is Flatpak and What it Takes to Package Unhaunter

## What is Flatpak?

Flatpak is a universal Linux application packaging system. Unlike `.deb` or `.rpm`, a Flatpak
package is distro-agnostic: the same `.flatpak` file works on Fedora, Ubuntu, Arch, NixOS, and any
other desktop Linux that has the Flatpak runtime installed. The application runs inside a sandbox
(using bubblewrap + cgroups), isolated from the host system, with declared access to resources
such as GPU, audio, display server, and user data.

The key idea: **the app brings its own dependency stack** (everything except the lowest OS
primitives and the chosen *runtime*), so it will work identically on all distros.

## Core Concepts

| Term | Meaning |
|------|---------|
| **Runtime** | A shared base layer (e.g. `org.freedesktop.Platform`) that many apps share. Provides glibc, X11/Wayland, Mesa, PulseAudio, etc. |
| **SDK** | The build-time counterpart to a runtime. Used only during `flatpak-builder` compilation. |
| **App ID** | Reverse-DNS identifier, e.g. `io.github.deavid.unhaunter`. Unique per app. |
| **Manifest** | A YAML or JSON file that describes how to build the app and what permissions it needs. |
| **flatpak-builder** | The CLI tool that reads a manifest and produces a finished Flatpak. |
| **OSTree** | The content-addressed storage system that Flatpak repos are built on (like Git, but for file trees). |
| **Repository** | An OSTree repo that clients pull updates from. Flathub is the biggest public one. |
| **Ref** | A named branch in an OSTree repo, e.g. `app/io.github.deavid.unhaunter/x86_64/stable`. |

## What it Takes for Unhaunter Specifically

### 1. App ID

Choose a reverse-DNS ID tied to something you control. Because the code is at
`github.com/deavid/unhaunter`, the canonical choice is:

```
io.github.deavid.unhaunter
```

This is also what Flathub requires for GitHub-hosted projects.

### 2. A Flatpak Manifest

You need a file named `io.github.deavid.unhaunter.yml` (or `.json`) at the root of the Flathub
app repo. It declares:

- Which runtime/SDK to use
- Build instructions (compile Rust, copy assets)
- Finish-args (sandbox permissions)
- Sources (git tag + vendored Cargo crates)

### 3. Offline Cargo Deps (the Big Rust-Specific Hurdle)

Flatpak builds happen in a network-isolated sandbox. `cargo build` cannot fetch crates from
crates.io during the build. You must pre-declare all Cargo dependencies as sources. The standard
tool for this is `flatpak-cargo-generator.py` from the
[flatpak-builder-tools](https://github.com/flatpak/flatpak-builder-tools) repo:

```bash
python3 flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
```

This produces a JSON file listing every crate tarball that Flatpak will download before entering
the sandbox. That file is committed alongside the manifest in the Flathub app repo. It must be
regenerated every time `Cargo.lock` changes.

### 4. Asset Placement

Unhaunter's binary (`unhaunter_game`) looks for assets in `./assets` relative to its working
directory. Inside the Flatpak, the binary sits at `/app/bin/unhaunter_game` and the assets at
`/app/share/unhaunter/assets`. A tiny wrapper script handles the working-directory mismatch:

```bash
#!/bin/bash
cd /app/share/unhaunter
exec /app/bin/unhaunter_game "$@"
```

Place this wrapper at `/app/bin/unhaunter` and set `command: unhaunter` in the manifest.

### 5. AppStream Metadata

A machine-readable XML description of the app is mandatory:

- `io.github.deavid.unhaunter.metainfo.xml` – description, screenshots, OARS ratings, release list
- `io.github.deavid.unhaunter.desktop` – desktop integration file
- App icons (PNG, 64×64, 128×128, 256×256 minimum)

These live in the source tree and the manifest installs them to `/app/share/`.

### 6. Runtime Choice

For a game that only needs display + audio + GPU:

```yaml
runtime: org.freedesktop.Platform
runtime-version: '24.08'
sdk: org.freedesktop.Sdk
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
```

`org.freedesktop.Platform 24.08` is the current LTS-aligned runtime (tied to freedesktop-sdk
24.08). It includes Mesa, ALSA, PulseAudio client libs, and Wayland/X11. No need for the heavier
GNOME or KDE runtimes.

### 7. Finish-Args (Sandbox Permissions)

Minimum set for a Bevy game:

```yaml
finish-args:
  - --share=ipc          # shared memory / MIT-SHM (X11 performance)
  - --socket=x11         # X11 display
  - --socket=wayland     # Wayland display (better on modern setups)
  - --socket=pulseaudio  # audio via PulseAudio / PipeWire-pulse
  - --device=dri         # GPU / render node access
  - --filesystem=xdg-config/unhaunter:create   # per-user config
  - --filesystem=xdg-data/unhaunter:create     # save files / scores
```

> Note: `--socket=alsa` is an alternative but PulseAudio is preferred because PipeWire exposes a
> PulseAudio-compatible socket on all modern distros, covering both legacy and modern setups.

### 8. Build Environment

```yaml
build-options:
  append-path: /usr/lib/sdk/rust-stable/bin
  env:
    CARGO_HOME: /run/build/unhaunter/cargo
    RUST_BACKTRACE: 1
    CARGO_INCREMENTAL: "0"  # deterministic builds
```

### 9. Summary of Files Needed

| File | Where | Purpose |
|------|-------|---------|
| `io.github.deavid.unhaunter.yml` | Flathub app repo root | The manifest |
| `cargo-sources.json` | Flathub app repo root | Pre-fetched crate list |
| `io.github.deavid.unhaunter.metainfo.xml` | Game source tree | AppStream metadata |
| `io.github.deavid.unhaunter.desktop` | Game source tree | Desktop integration |
| `io.github.deavid.unhaunter.png` (multiple sizes) | Game source tree | App icons |
| Wrapper shell script | Game source tree or inline in manifest | Fix working directory |

### 10. Quick Sanity Checklist

- [ ] `flatpak-builder` installed locally
- [ ] `org.freedesktop.Platform//24.08` and `org.freedesktop.Sdk//24.08` installed
- [ ] `org.freedesktop.Sdk.Extension.rust-stable` installed
- [ ] `cargo-sources.json` generated from current `Cargo.lock`
- [ ] AppStream XML validates (`appstreamcli validate`)
- [ ] Desktop file validates (`desktop-file-validate`)
- [ ] Local test build passes: `flatpak-builder --force-clean build-dir io.github.deavid.unhaunter.yml`
