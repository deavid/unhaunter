# 02 – The Flatpak Manifest for Unhaunter

This document shows a concrete, annotated Flatpak manifest for Unhaunter. It is a starting point;
details will evolve as the project matures.

## File: `io.github.deavid.unhaunter.yml`

```yaml
app-id: io.github.deavid.unhaunter

# --- Runtime ---
# org.freedesktop.Platform is the lightest reasonable base for a game.
# It ships Mesa (Vulkan + OpenGL), ALSA/PulseAudio client libs, X11, Wayland.
runtime: org.freedesktop.Platform
runtime-version: '24.08'
sdk: org.freedesktop.Sdk

# Rust compiler toolchain as an SDK extension (only available at build time).
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable

# The entry point. We use a wrapper script to set the working directory
# because Bevy looks for `./assets` relative to CWD.
command: unhaunter

# --- Sandbox permissions ---
finish-args:
  - --share=ipc           # MIT-SHM for X11 performance
  - --socket=x11          # X11 display server
  - --socket=wayland      # Wayland display server
  - --socket=pulseaudio   # Audio (works for PulseAudio AND PipeWire-pulse)
  - --device=dri          # GPU / DRM render nodes
  - --filesystem=xdg-config/unhaunter:create  # config directory (~/.config/unhaunter)
  - --filesystem=xdg-data/unhaunter:create    # save data (~/.local/share/unhaunter)

# --- Build-time environment ---
build-options:
  # Put the Rust SDK extension binaries first on PATH.
  append-path: /usr/lib/sdk/rust-stable/bin
  env:
    # Redirect Cargo's cache inside the build directory (required in the sandbox).
    CARGO_HOME: /run/build/unhaunter/cargo
    RUST_BACKTRACE: "1"
    # Disable incremental compilation for reproducible/deterministic builds.
    CARGO_INCREMENTAL: "0"

modules:
  - name: unhaunter
    buildsystem: simple

    build-commands:
      # 1. Pre-fetch all crates into CARGO_HOME using the vendored sources.
      #    The `--offline` flag is critical – no network during build.
      - cargo --offline fetch --manifest-path Cargo.toml --verbose

      # 2. Build the release binary.
      - cargo --offline build --release --target x86_64-unknown-linux-gnu --bin unhaunter_game

      # 3. Install the binary.
      - install -Dm755 target/x86_64-unknown-linux-gnu/release/unhaunter_game
                       /app/bin/unhaunter_game

      # 4. Install assets (game data).
      - mkdir -p /app/share/unhaunter
      - cp -r assets /app/share/unhaunter/assets

      # 5. Install the wrapper script that sets the correct working directory.
      - install -Dm755 flatpak/unhaunter.sh /app/bin/unhaunter

      # 6. Install AppStream metadata.
      - install -Dm644 io.github.deavid.unhaunter.metainfo.xml
                       /app/share/metainfo/io.github.deavid.unhaunter.metainfo.xml

      # 7. Install desktop file.
      - install -Dm644 io.github.deavid.unhaunter.desktop
                       /app/share/applications/io.github.deavid.unhaunter.desktop

      # 8. Install icons (add more sizes as needed).
      - install -Dm644 favicon-256x256.png
                       /app/share/icons/hicolor/256x256/apps/io.github.deavid.unhaunter.png
      - install -Dm644 favicon-128x128.png
                       /app/share/icons/hicolor/128x128/apps/io.github.deavid.unhaunter.png
      - install -Dm644 favicon-64x64.png
                       /app/share/icons/hicolor/64x64/apps/io.github.deavid.unhaunter.png

    sources:
      # The game source code, pinned to a specific tag.
      - type: git
        url: https://github.com/deavid/unhaunter.git
        tag: v0.X.X          # replace with the version being built
        commit: ABCDEF...    # optional but good for reproducibility

      # All Cargo crate tarballs, pre-declared so flatpak-builder downloads
      # them before entering the network-isolated sandbox.
      # Generated with: python3 flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
      - cargo-sources.json
```

## Companion Files

### `flatpak/unhaunter.sh` (wrapper script)

```bash
#!/bin/bash
# Change to the directory that contains the assets folder, then launch the game.
cd /app/share/unhaunter
exec /app/bin/unhaunter_game "$@"
```

### `io.github.deavid.unhaunter.desktop`

```ini
[Desktop Entry]
Name=Unhaunter
GenericName=Paranormal Investigation Game
Comment=A 2D isometric ghost-hunting game
Exec=unhaunter
Icon=io.github.deavid.unhaunter
Type=Application
Categories=Game;
Keywords=ghost;horror;paranormal;investigation;
StartupNotify=true
```

### Minimal `io.github.deavid.unhaunter.metainfo.xml`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>io.github.deavid.unhaunter</id>
  <name>Unhaunter</name>
  <summary>A 2D isometric paranormal investigation game</summary>
  <metadata_license>CC0-1.0</metadata_license>
  <project_license>MIT OR Apache-2.0</project_license>

  <description>
    <p>
      Unhaunter is a 2D isometric ghost-hunting game where you investigate haunted
      locations, gather evidence, and identify the type of ghost before banishing it.
    </p>
  </description>

  <launchable type="desktop-id">io.github.deavid.unhaunter.desktop</launchable>

  <screenshots>
    <screenshot type="default">
      <image>https://raw.githubusercontent.com/deavid/unhaunter/main/screenshots/...</image>
    </screenshot>
  </screenshots>

  <!-- OARS content ratings: adjust values to match actual game content -->
  <content_rating type="oars-1.1">
    <content_attribute id="violence-cartoon">mild</content_attribute>
    <content_attribute id="violence-fantasy">mild</content_attribute>
  </content_rating>

  <releases>
    <release version="0.X.X" date="YYYY-MM-DD">
      <description>
        <p>Release notes here.</p>
      </description>
    </release>
  </releases>

  <url type="homepage">https://deavid.github.io/unhaunter</url>
  <url type="bugtracker">https://github.com/deavid/unhaunter/issues</url>
</component>
```

## Notes on the Build

- `cargo --offline fetch` will fail if `cargo-sources.json` is stale. Regenerate it whenever
  `Cargo.lock` changes.
- The `x86_64-unknown-linux-gnu` target is explicit even though the build host is also x86_64,
  because it matches the CI cross-compilation target already used in the project.
- Bevy's asset loader uses `std::fs` path resolution. If the game ever gains support for
  `BEVY_ASSET_ROOT` or similar env var, the wrapper script can be simplified to just setting
  that variable rather than `cd`-ing.
- Upscaled assets (`upscale_assets.sh`) should be run **before** the Flatpak build so the
  `assets/` directory is already complete. The Flatpak build itself should not run the
  upscaler (ImageMagick / xbrzscale are not in the SDK).
