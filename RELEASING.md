Release Process Main/Stable
============================

**Pre-checks**

- [ ] Merge origin/beta and origin/main to ensure no changes are being left behind.
  `git merge origin/beta origin/main`

- [ ] Run `cargo test` and ensure no failures.

- [ ] Run `cargo clippy` and ensure no warnings or errors.

- [ ] Run `cargo fmt --all` and ensure no errors.

**Changelog + Version bump**

- [ ] Review and update `CHANGELOG.md` for the new version

- [ ] Review `README.md` to see if anything needs updating.

- [ ] Update the version in `/Cargo.toml` on the `[workspace.package]` section.

**Post-checks**

- [ ] Run `RUSTFLAGS="-D warnings" just build-all` and ensure no warnings or errors.

**Commit**

- [ ] Commit the changes as "Release 0.X.X"

- [ ] Open pull request to merge these changes into origin/next, then onto origin/beta, and finally to origin/main.

- [ ] Checkout the origin/main branch and create a tag "v0.x.x". `git tag v0.x.x`

- [ ] Push the new tag to Github - this will trigger the release. `git push origin v0.x.x`

- [ ] Wait for release (GitHub actions) confirm that everything looks correct, that the release was created.

**Website**

- [ ] Go to website branch and add announcement, update the website after the new release.


### Notes: Windows MSVC cross build from Linux (Optional)

If anyone is interested in this variant, here are the original notes I took.

Very poorly tested. Do at your own risk.

`rustup target add x86_64-pc-windows-msvc`

`cargo install xwin`

`xwin --accept-license splat --output $HOME/.xwin`

For Windows MSVC cross compilation in your `$HOME/.cargo/config` add:

```
[target.x86_64-pc-windows-msvc]
linker = "lld"
rustflags = [
    "-Lnative=/home/me/.xwin/crt/lib/x86_64",
    "-Lnative=/home/me/.xwin/sdk/lib/um/x86_64",
    "-Lnative=/home/me/.xwin/sdk/lib/ucrt/x86_64",
]
```

To build it, execute:

`CARGO_FEATURE_PURE=1 cargo build --target=x86_64-pc-windows-msvc --release`


