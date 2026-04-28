# 08 – Justfile Recipes for Flatpak Builds

## Overview

These are **suggested** Justfile recipes you can add when ready to experiment. The existing
CI pipeline is intentionally unchanged (see note at the end of this doc).

The recipes build on top of the existing `package-linux` workflow: first produce the Linux
tarball the normal way, then wrap that output into a Flatpak.

---

## Prerequisites (Developer Machine)

```bash
# Install flatpak-builder
sudo apt-get install flatpak-builder   # Debian/Ubuntu
sudo dnf install flatpak-builder       # Fedora

# Install the required runtime and SDK
flatpak install flathub \
  org.freedesktop.Platform//24.08 \
  org.freedesktop.Sdk//24.08 \
  org.freedesktop.Sdk.Extension.rust-stable//24.08

# Install Python tool for generating cargo sources
pip3 install --user aiohttp toml  # deps of flatpak-cargo-generator.py
# Get the generator script:
# curl -O https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py
```

---

## Suggested Justfile Recipes

Add these to the existing `Justfile`. They do not conflict with any current recipe.

```just
# ── Flatpak Recipes ─────────────────────────────────────────────────────

_flatpak_app_id := "io.github.deavid.unhaunter"
_flatpak_manifest := "flatpak/" + _flatpak_app_id + ".yml"
_flatpak_build_dir := "flatpak-build"
_flatpak_repo_dir := "flatpak-repo"

# Regenerate cargo-sources.json from current Cargo.lock (run when Cargo.lock changes)
flatpak-gen-sources:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "::group::flatpak-gen-sources"
    python3 flatpak/flatpak-cargo-generator.py Cargo.lock \
        -o flatpak/cargo-sources.json
    echo "cargo-sources.json updated."
    echo "::endgroup::"

# Build a local Flatpak (for testing, does NOT use the pre-built tarball — full build)
flatpak-build: flatpak-gen-sources
    #!/usr/bin/env bash
    set -euo pipefail
    echo "::group::flatpak-build"
    rm -rf {{_flatpak_build_dir}}
    flatpak-builder \
        --force-clean \
        --repo={{_flatpak_repo_dir}} \
        {{_flatpak_build_dir}} \
        {{_flatpak_manifest}}
    echo "Flatpak build complete. Repo at: {{_flatpak_repo_dir}}"
    echo "::endgroup::"

# Install the locally built Flatpak for manual testing
flatpak-install: flatpak-build
    #!/usr/bin/env bash
    set -euo pipefail
    flatpak --user remote-add --if-not-exists \
        --no-gpg-verify unhaunter-local {{_flatpak_repo_dir}}
    flatpak --user install --reinstall -y \
        unhaunter-local {{_flatpak_app_id}}
    echo "Installed. Run with: flatpak run {{_flatpak_app_id}}"

# Run the locally installed Flatpak
flatpak-run:
    flatpak run {{_flatpak_app_id}}

# Uninstall the local test Flatpak
flatpak-uninstall:
    flatpak --user uninstall -y {{_flatpak_app_id}} || true
    flatpak --user remote-delete --force unhaunter-local || true

# Build a distributable single-file .flatpak bundle (for sharing without a repo)
flatpak-bundle: flatpak-build
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION={{_version}}
    OUT="releases/unhaunter-${VERSION}-linux-x86_64.flatpak"
    flatpak build-bundle {{_flatpak_repo_dir}} "$OUT" {{_flatpak_app_id}}
    echo "Bundle created: $OUT"

# Export the Flatpak build to a static OSTree repo (for self-hosted alpha)
flatpak-export-repo: flatpak-build
    #!/usr/bin/env bash
    set -euo pipefail
    flatpak build-update-repo --generate-static-deltas {{_flatpak_repo_dir}}
    echo "Repo ready at: {{_flatpak_repo_dir}}"
    echo "Point a web server at that directory, or copy to GitHub Pages."

# Validate AppStream metadata
flatpak-validate-metadata:
    #!/usr/bin/env bash
    set -euo pipefail
    appstreamcli validate \
        flatpak/io.github.deavid.unhaunter.metainfo.xml
    desktop-file-validate \
        flatpak/io.github.deavid.unhaunter.desktop
    echo "Metadata valid."
```

---

## Directory Layout for Flatpak Files

```
unhaunter/
├── flatpak/
│   ├── io.github.deavid.unhaunter.yml         ← manifest (symlinked in Flathub repo)
│   ├── cargo-sources.json                      ← generated, committed
│   ├── flatpak-cargo-generator.py              ← vendored copy of the generator script
│   ├── unhaunter.sh                            ← wrapper script
│   ├── io.github.deavid.unhaunter.desktop      ← desktop entry
│   └── io.github.deavid.unhaunter.metainfo.xml ← AppStream XML
├── Justfile                                    ← add recipes above
...
```

Keeping everything under `flatpak/` keeps the root clean and makes the Flathub app repo easy to
populate (it just references files from here via the git source entry).

---

## Relationship to the Existing Linux Build

The Flatpak build is independent from `package-linux`. Both compile from source:

- `package-linux` → `dist/linux/` tarball → GitHub Release asset
- `flatpak-build` → `flatpak-repo/` → Flathub or `.flatpak` bundle

There is no dependency between them. You do not feed the tarball into Flatpak; `flatpak-builder`
runs its own `cargo build` from the declared git source or local sources.

---

## Note on CI Integration

The recipes above are designed for **local developer use**. The existing CI scripts
(`.github/workflows/release-*.yml`) are intentionally not modified. When ready to automate
Flatpak publishing in CI, the new steps would typically be added as new jobs in those workflow
files (see doc 09 for the upload half of that story).
