# Default recipes - can be overridden by environment variables from GH Actions
_target_dir := 'target'
_dist_dir := 'dist'
_releases_dir := 'releases'
_assets_dir := 'assets'
_readme := 'README.md'
_license := 'LICENSE'
_changelog := 'CHANGELOG.md'
_version := `cargo pkgid  | sed 's/.*#//'`

default:
  just --list

# Ensure dist dir exists
ensure-dist-dir:
    echo "Unhaunter Version {{_version}}"
    mkdir -p {{_dist_dir}}
    mkdir -p {{_dist_dir}}/common
    mkdir -p {{_dist_dir}}/linux
    mkdir -p {{_dist_dir}}/windows
    mkdir -p {{_dist_dir}}/wasm
    mkdir -p {{_releases_dir}}

# == Build Recipes ==

# Upscale assets for release
upscale-assets:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "${SKIP_UPSCALE:-0}" = "1" ]; then
        echo "::group::upscale-assets"
        echo "Skipping upscale-assets due to SKIP_UPSCALE=1"
        echo "::endgroup::"
        exit 0
    fi
    echo "::group::upscale-assets"
    echo "Ensuring upscaled assets are up to date..."
    step_start=$SECONDS
    ./upscale_assets.sh auto
    cargo run -p assetidx_updater --release
    echo "[timing] upscale-assets=$((SECONDS - step_start))s"
    echo "::endgroup::"

# Build Linux Release Binary
build-linux:
    #!/usr/bin/env bash
    set -euo pipefail
    recipe_start=$SECONDS
    echo "::group::build-linux"
    echo "Building Linux release..."
    step_start=$SECONDS
    cargo build --release --target x86_64-unknown-linux-gnu
    echo "[timing] build-linux:cargo-build=$((SECONDS - step_start))s"
    echo "Linux build complete."
    echo "[timing] build-linux:total=$((SECONDS - recipe_start))s"
    echo "::endgroup::"

# Build Windows Release Binary (GNU toolchain)
# Assumes mingw-w64 is installed (either locally or in GH Actions runner)
build-windows:
    #!/usr/bin/env bash
    set -euo pipefail
    recipe_start=$SECONDS
    echo "::group::build-windows"
    echo "Building Windows release..."
    step_start=$SECONDS
    cargo build --release --target x86_64-pc-windows-gnu
    echo "[timing] build-windows:cargo-build=$((SECONDS - step_start))s"
    echo "Windows build complete."
    echo "[timing] build-windows:total=$((SECONDS - recipe_start))s"
    echo "::endgroup::"

# Build WASM Package
build-wasm:
    #!/usr/bin/env bash
    set -euo pipefail
    recipe_start=$SECONDS
    echo "::group::build-wasm"
    echo "Building WASM package..."
    step_start=$SECONDS
    cargo install wasm-pack # Ensure wasm-pack is available
    echo "[timing] build-wasm:install-wasm-pack=$((SECONDS - step_start))s"
    step_start=$SECONDS
    wasm-pack build --release --target web
    echo "[timing] build-wasm:wasm-pack-build=$((SECONDS - step_start))s"
    echo "WASM build complete."
    echo "[timing] build-wasm:total=$((SECONDS - recipe_start))s"
    echo "::endgroup::"

# Build Dedicated Server Binary (Linux only)
build-server:
    #!/usr/bin/env bash
    set -euo pipefail
    recipe_start=$SECONDS
    echo "::group::build-server"
    echo "Building server release..."
    step_start=$SECONDS
    cargo build --release --target x86_64-unknown-linux-gnu --bin unhaunter_dedicated
    echo "[timing] build-server:cargo-build=$((SECONDS - step_start))s"
    echo "Server build complete."
    echo "[timing] build-server:total=$((SECONDS - recipe_start))s"
    echo "::endgroup::"

# == Packaging Recipes ==

package-common: ensure-dist-dir upscale-assets
    #!/usr/bin/env bash
    set -euo pipefail
    recipe_start=$SECONDS
    echo "::group::package-common"
    echo "Packaging Common artifacts for {{_version}}..."
    step_start=$SECONDS
    rm -rf {{_dist_dir}}/common/*
    cp -r {{_assets_dir}} {{_dist_dir}}/common/assets
    cp {{_readme}} {{_dist_dir}}/common/README.md
    cp {{_license}} {{_dist_dir}}/common/LICENSE
    cp {{_changelog}} {{_dist_dir}}/common/CHANGELOG.md
    cp favicon*.png favicon*.ico {{_dist_dir}}/common/
    cp screenshots {{_dist_dir}}/common/screenshots -R
    echo "[timing] package-common:copy-assets-and-docs=$((SECONDS - step_start))s"
    echo "[timing] package-common:total=$((SECONDS - recipe_start))s"
    echo "::endgroup::"

# Package Linux Artifacts into .tar.gz
package-linux: build-linux package-common
    #!/usr/bin/env bash
    set -euo pipefail
    recipe_start=$SECONDS
    echo "::group::package-linux"
    echo "Packaging Linux artifact for {{_version}}..."
    step_start=$SECONDS
    rm -rf {{_dist_dir}}/linux/*
    mkdir -p {{_dist_dir}}/linux/unhaunter-{{_version}}
    cp {{_dist_dir}}/common/* {{_dist_dir}}/linux/unhaunter-{{_version}}/ -R
    cp {{_target_dir}}/x86_64-unknown-linux-gnu/release/unhaunter_game {{_dist_dir}}/linux/unhaunter-{{_version}}/unhaunter_game
    echo "[timing] package-linux:prepare-files=$((SECONDS - step_start))s"
    step_start=$SECONDS
    unlink {{_releases_dir}}/unhaunter-{{_version}}-linux-x86_64.tar.gz || true
    tar -czf {{_releases_dir}}/unhaunter-{{_version}}-linux-x86_64.tar.gz -C {{_dist_dir}}/linux unhaunter-{{_version}}
    echo "[timing] package-linux:create-tarball=$((SECONDS - step_start))s"
    echo "Linux package created: {{_releases_dir}}/unhaunter-{{_version}}-linux-x86_64.tar.gz"
    echo "[timing] package-linux:total=$((SECONDS - recipe_start))s"
    echo "::endgroup::"

# Package Windows Artifacts into .zip
package-windows: build-windows package-common
    #!/usr/bin/env bash
    set -euo pipefail
    recipe_start=$SECONDS
    echo "::group::package-windows"
    echo "Packaging Windows artifact for {{_version}}..."
    step_start=$SECONDS
    rm -rf {{_dist_dir}}/windows/*
    mkdir -p {{_dist_dir}}/windows/unhaunter-{{_version}}
    cp {{_dist_dir}}/common/* {{_dist_dir}}/windows/unhaunter-{{_version}}/ -R
    cp {{_target_dir}}/x86_64-pc-windows-gnu/release/unhaunter_game.exe {{_dist_dir}}/windows/unhaunter-{{_version}}/unhaunter_game.exe
    echo "[timing] package-windows:prepare-files=$((SECONDS - step_start))s"
    step_start=$SECONDS
    unlink {{_releases_dir}}/unhaunter-{{_version}}-windows-x86_64.zip || true
    cd {{_dist_dir}}/windows && zip -rq ../../{{_releases_dir}}/unhaunter-{{_version}}-windows-x86_64.zip unhaunter-{{_version}}
    cd ../../
    echo "[timing] package-windows:create-zip=$((SECONDS - step_start))s"
    echo "Windows package created: {{_releases_dir}}/unhaunter-{{_version}}-windows-x86_64.zip"
    echo "[timing] package-windows:total=$((SECONDS - recipe_start))s"
    echo "::endgroup::"

# Package WASM Artifacts into .zip
package-wasm: build-wasm package-common
    #!/usr/bin/env bash
    set -euo pipefail
    recipe_start=$SECONDS
    echo "::group::package-wasm"
    echo "Packaging WASM artifact for {{_version}}..."
    step_start=$SECONDS
    rm -rf {{_dist_dir}}/wasm/*
    cp {{_dist_dir}}/common/* {{_dist_dir}}/wasm/ -R
    cp -r pkg {{_dist_dir}}/wasm/pkg
    cp index.html {{_dist_dir}}/wasm/index.html
    echo "[timing] package-wasm:prepare-files=$((SECONDS - step_start))s"
    step_start=$SECONDS
    unlink {{_releases_dir}}/unhaunter-{{_version}}-wasm.zip || true
    cd {{_dist_dir}}/wasm && zip -rq ../../{{_releases_dir}}/unhaunter-{{_version}}-wasm.zip *
    cd ../../
    echo "[timing] package-wasm:create-zip=$((SECONDS - step_start))s"
    echo "WASM package created: {{_releases_dir}}/unhaunter-{{_version}}-wasm.zip"
    echo "[timing] package-wasm:total=$((SECONDS - recipe_start))s"
    echo "::endgroup::"

# Package Dedicated Server Artifacts into .tar.gz
package-server: ensure-dist-dir build-server
    #!/usr/bin/env bash
    set -euo pipefail
    recipe_start=$SECONDS
    echo "::group::package-server"
    echo "Packaging server artifact for {{_version}}..."
    step_start=$SECONDS
    rm -rf {{_dist_dir}}/server
    mkdir -p {{_dist_dir}}/server/unhaunter-{{_version}}
    cp {{_target_dir}}/x86_64-unknown-linux-gnu/release/unhaunter_dedicated {{_dist_dir}}/server/unhaunter-{{_version}}/unhaunter_dedicated
    cp -r {{_assets_dir}} {{_dist_dir}}/server/unhaunter-{{_version}}/assets
    echo "[timing] package-server:prepare-files=$((SECONDS - step_start))s"
    step_start=$SECONDS
    unlink {{_releases_dir}}/unhaunter-{{_version}}-server-linux-x86_64.tar.gz || true
    tar -czf {{_releases_dir}}/unhaunter-{{_version}}-server-linux-x86_64.tar.gz -C {{_dist_dir}}/server unhaunter-{{_version}}
    echo "[timing] package-server:create-tarball=$((SECONDS - step_start))s"
    echo "Server package created: {{_releases_dir}}/unhaunter-{{_version}}-server-linux-x86_64.tar.gz"
    echo "[timing] package-server:total=$((SECONDS - recipe_start))s"
    echo "::endgroup::"


# == Combined Recipes ==

# Build all targets
build-all: build-linux build-windows build-wasm

# Create all release packages
package-all: package-linux package-windows package-wasm package-server
