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

# Build Linux Release Binary
build-linux:
    echo "Building Linux release..."
    cargo build --release --target x86_64-unknown-linux-gnu
    echo "Linux build complete."

# Build Windows Release Binary (GNU toolchain)
# Assumes mingw-w64 is installed (either locally or in GH Actions runner)
build-windows:
    echo "Building Windows release..."
    cargo build --release --target x86_64-pc-windows-gnu
    echo "Windows build complete."

# Build WASM Package
build-wasm:
    echo "Building WASM package..."
    cargo install wasm-pack # Ensure wasm-pack is available
    wasm-pack build --release --target web
    echo "WASM build complete."

# == Packaging Recipes ==

package-common: ensure-dist-dir
    echo "Packaging Common artifacts for {{_version}}..."
    rm -rf {{_dist_dir}}/common/*
    cp -r {{_assets_dir}} {{_dist_dir}}/common/assets
    cp {{_readme}} {{_dist_dir}}/common/README.md
    cp {{_license}} {{_dist_dir}}/common/LICENSE
    cp {{_changelog}} {{_dist_dir}}/common/CHANGELOG.md
    cp favicon*.png favicon*.ico {{_dist_dir}}/common/
    cp screenshots {{_dist_dir}}/common/screenshots -R

# Package Linux Artifacts into .tar.gz
package-linux: build-linux package-common
    echo "Packaging Linux artifact for {{_version}}..."
    rm -rf {{_dist_dir}}/linux/*
    cp {{_dist_dir}}/common/* {{_dist_dir}}/linux/ -R
    cp {{_target_dir}}/x86_64-unknown-linux-gnu/release/unhaunter_game {{_dist_dir}}/linux/unhaunter_game
    tar -czvf {{_releases_dir}}/unhaunter-{{_version}}-linux-x86_64.tar.gz -C {{_dist_dir}}/linux .
    echo "Linux package created: {{_releases_dir}}/unhaunter-{{_version}}-linux-x86_64.tar.gz"

# Package Windows Artifacts into .zip
package-windows: build-windows package-common
    echo "Packaging Windows artifact for {{_version}}..."
    rm -rf {{_dist_dir}}/windows/*
    cp {{_dist_dir}}/common/* {{_dist_dir}}/windows/ -R
    cp {{_target_dir}}/x86_64-pc-windows-gnu/release/unhaunter_game.exe {{_dist_dir}}/windows/unhaunter_game.exe
    zip -r {{_releases_dir}}/unhaunter-{{_version}}-windows-x86_64.zip {{_dist_dir}}/windows/*
    echo "Windows package created: {{_releases_dir}}/unhaunter-{{_version}}-windows-x86_64.zip"

# Package WASM Artifacts into .zip
package-wasm: build-wasm package-common
    echo "Packaging WASM artifact for {{_version}}..."
    rm -rf {{_dist_dir}}/wasm/*
    cp {{_dist_dir}}/common/* {{_dist_dir}}/wasm/ -R
    cp -r pkg {{_dist_dir}}/wasm/pkg
    cp index.html {{_dist_dir}}/wasm/index.html
    zip -r {{_releases_dir}}/unhaunter-{{_version}}-wasm.zip {{_dist_dir}}/wasm/*
    echo "WASM package created: {{_releases_dir}}/unhaunter-{{_version}}-wasm.zip"


# == Combined Recipes ==

# Build all targets
build-all: build-linux build-windows build-wasm

# Create all release packages
package-all: package-linux package-windows package-wasm
