# RELEASE DIST PROCESS



deavid@deavws:~/git/rust$ cd unhaunter_dist/
deavid@deavws:~/git/rust/unhaunter_dist$ cp ../unhaunter/assets/ . -R
deavid@deavws:~/git/rust/unhaunter_dist$ cp ../unhaunter/*.md .
deavid@deavws:~/git/rust/unhaunter_dist$ ls
assets  CHANGELOG.md  NOTES.md  PROJECT_FILE_DESCRIPTIONS.md  README.md
deavid@deavws:~/git/rust/unhaunter_dist$ cp ../unhaunter/*.png .
deavid@deavws:~/git/rust/unhaunter_dist$ cp ../unhaunter/*.ico .
deavid@deavws:~/git/rust/unhaunter_dist$ cp ../unhaunter/*.html .
deavid@deavws:~/git/rust/unhaunter_dist$ cp ../unhaunter/LICENSE .
deavid@deavws:~/git/rust/unhaunter_dist$ cp ../unhaunter/screenshots . -R
deavid@deavws:~/git/rust/unhaunter_dist$ 

# Linux x64

cargo build --release

deavid@deavws:~/git/rust/unhaunter_dist$ cp ../unhaunter/target/release/unhaunter_game unhaunter_game_linux_x64


# WASM

deavid@deavws:~/git/rust/unhaunter$ wasm-pack build --release --target web

deavid@deavws:~/git/rust/unhaunter_dist$ cp ../unhaunter/pkg . -R

# Windows builds:
# https://bevy-cheatbook.github.io/setup/cross/linux-windows.html

# Windows x64 - GNU (There's also a MSVC version)
# rustup target add x86_64-pc-windows-gnu
# :: If mingw-w64 is missing: error: Error calling dlltool 'x86_64-w64-mingw32-dlltool': No such file or directory (os error 2)
# sudo apt install mingw-w64

deavid@deavws:~/git/rust/unhaunter$ cargo build --target x86_64-pc-windows-gnu --release

deavid@deavws:~/git/rust/unhaunter_dist$ cp ../unhaunter/target/x86_64-pc-windows-gnu/release/un
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

deavid@deavws:~/git/rust/unhaunter_dist$ cp ../unhaunter/target/x86_64-pc-windows-msvc/release/unhaunter_game.exe unhaunter_game_windows_x64_msvc.exe 


###### Packaging

deavid@deavws:~/git/rust/unhaunter_dist$ cd ..
deavid@deavws:~/git/rust$ ln -s unhaunter_dist unhaunter_v0_2_7

deavid@deavws:~/git/rust$ zip -r unhaunter_v0_2_7_wasm_windows_linux.zip unhaunter_v0_2_7

Resulting ZIP in the parent folder.
