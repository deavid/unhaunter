# RELEASE DIST PROCESS

**NOTE:** These are being migrated to "Justfile" which runs with the 'just' command.

These are just "dirty" notes on how to make the builds for releasing.

I will try to automate a script later on.


rust_unhaunter$ cd unhaunter_dist/
rust_unhaunter/unhaunter_dist$ cp ../unhaunter/assets/ . -R
rust_unhaunter/unhaunter_dist$ cp ../unhaunter/*.md .
rust_unhaunter/unhaunter_dist$ ls
assets  CHANGELOG.md  NOTES.md  PROJECT_FILE_DESCRIPTIONS.md  README.md
rust_unhaunter/unhaunter_dist$ cp ../unhaunter/*.png .
rust_unhaunter/unhaunter_dist$ cp ../unhaunter/*.ico .
rust_unhaunter/unhaunter_dist$ cp ../unhaunter/*.html .
rust_unhaunter/unhaunter_dist$ cp ../unhaunter/LICENSE .
rust_unhaunter/unhaunter_dist$ cp ../unhaunter/screenshots . -R
rust_unhaunter/unhaunter_dist$

# Linux x64

cargo build --release

rust_unhaunter/unhaunter_dist$ cp ../unhaunter/target/release/unhaunter_game unhaunter_game_linux_x64


# WASM

rust_unhaunter/unhaunter$ wasm-pack build --release --target web

rust_unhaunter/unhaunter_dist$ cp ../unhaunter/pkg . -R

# Windows builds:
# https://bevy-cheatbook.github.io/setup/cross/linux-windows.html

# Windows x64 - GNU (There's also a MSVC version)
# rustup target add x86_64-pc-windows-gnu
# :: If mingw-w64 is missing: error: Error calling dlltool 'x86_64-w64-mingw32-dlltool': No such file or directory (os error 2)
# sudo apt install mingw-w64

rust_unhaunter/unhaunter$ cargo build --target x86_64-pc-windows-gnu --release

rust_unhaunter/unhaunter_dist$ cp ../unhaunter/target/x86_64-pc-windows-gnu/release/un
haunter_game.exe unhaunter_game_windows_x64_gnu.exe

# Windows x64 - MSVC
# rustup target add x86_64-pc-windows-msvc
# cargo install xwin
# deavid@deavws:~$ xwin --accept-license splat --output $HOME/.xwin

# For Windows MSVC cross compilation - $HOME/.cargo/config:
# [target.x86_64-pc-windows-msvc]
# linker = "lld"
# rustflags = [
#     "-Lnative=/home/me/.xwin/crt/lib/x86_64",
#     "-Lnative=/home/me/.xwin/sdk/lib/um/x86_64",
#     "-Lnative=/home/me/.xwin/sdk/lib/ucrt/x86_64",
# ]

CARGO_FEATURE_PURE=1 cargo build --target=x86_64-pc-windows-msvc --release

# Still fails on zstd-sys saying:
warning: zstd-sys@2.0.15+zstd.1.5.7: GNU compiler is not supported for this target
error occurred in cc-rs: failed to find tool "lib.exe": No such file or directory (os error 2)

zstd-sys is a crate that puts bindings to the zstd library, which might be confused by the
cross compilation to windows MSVC.

This crate is brought by Tiled for de-compression purposes.

# tiled = { version = "0.14", default-features = false }
# Asking for "wasm" was asking for zstd. This is only for zlib compression. Not needed.
# Now it builds.

rust_unhaunter/unhaunter_dist$ cp ../unhaunter/target/x86_64-pc-windows-msvc/release/unhaunter_game.exe unhaunter_game_windows_x64_msvc.exe


###### Packaging

rust_unhaunter/unhaunter_dist$ cd ..
rust_unhaunter$ ln -s unhaunter_dist unhaunter_v0_2_7

rust_unhaunter$ zip -r unhaunter_v0_2_7_wasm_windows_linux.zip unhaunter_v0_2_7

rust_unhaunter$ unlink unhaunter_v0_2_7

Resulting ZIP in the parent folder.
