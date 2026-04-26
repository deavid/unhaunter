# 06 – Rules, Regulations, and Content Policies

## Flathub Requirements

Flathub has documented quality guidelines. These are not optional for apps in the main catalogue.
The full docs live at: [https://docs.flathub.org/docs/for-app-authors/](https://docs.flathub.org/docs/for-app-authors/)

### Mandatory Requirements

| Requirement | Details |
|-------------|---------|
| **Valid App ID** | Reverse-DNS, tied to a domain or a hosting service you control (e.g. `io.github.deavid.*`). |
| **AppStream metainfo** | Valid `*.metainfo.xml`. Must pass `appstreamcli validate`. Includes name, summary, description, screenshots, license, OARS ratings, and a release history. |
| **Desktop file** | Valid `.desktop` file, installed to `/app/share/applications/`. |
| **Icons** | At least one PNG icon at 128×128 or 256×256 installed to `/app/share/icons/hicolor/`. SVG is also acceptable. |
| **No bundled system libraries** | Libraries already in `org.freedesktop.Platform` must not be re-bundled. Flathub will reject manifests that include unnecessary shared-library sources. |
| **Sandbox principle of least privilege** | Request only the permissions the app actually needs. Reviewers will push back on `--filesystem=host` or `--socket=session-bus` unless justified. |
| **No network access at build time** | Builds run offline. All sources must be declared upfront (hence `cargo-sources.json`). |
| **Stable app (no crashing)** | Reviewers install and briefly run the app. A crash on launch is grounds for rejection. |

### Content Rules

- **OARS ratings are mandatory** for games. You must accurately rate your game for:
  violence, sexual content, profanity, gambling, drugs, etc. Inaccurate ratings can lead to
  removal.
- **Malware, spyware, or privacy-violating behaviour** will result in permanent ban.
- **Proprietary assets bundled with a claimed FOSS license** will result in rejection or removal.
  All bundled content must be compatible with the declared license.
- **Unhaunter specific:** The game is MIT/Apache-2.0. The assets (sprites, sounds) also need to
  have compatible licenses. Verify that all third-party assets embedded in the `assets/` folder
  have licenses that allow redistribution via Flathub.

### Quality Guidelines (Enforced Loosely, But Matter for Visibility)

- Screenshots should be 1248×702 or 1:1 ratio, no desktop environment visible.
- Description must be in English as the default locale.
- Release notes should be meaningful, not just "bug fixes".
- App should not request `--persist` or `--filesystem` access to sensitive paths without a clear
  reason.

---

## Flatpak Sandbox Permission Rules

Some permissions raise red flags during review:

| Permission | Status |
|-----------|--------|
| `--filesystem=home` | Discouraged – use `xdg-*` paths instead |
| `--filesystem=host` | Rejected unless the app explicitly needs it (e.g. a file manager) |
| `--socket=session-bus` | Requires strong justification |
| `--socket=system-bus` | Almost never approved |
| `--device=all` | Discouraged; prefer `--device=dri` for games |
| `--share=network` | Allowed if the app has network features; add a reason in the PR |

For Unhaunter with its local multiplayer/hub server, `--share=network` may be needed if the
game communicates with a local server or has online features. Document the reason clearly.

---

## Build Reproducibility

Flathub expects builds to be deterministic. Key practices:

- Pin the Rust toolchain version in the SDK extension (it is already versioned by the SDK slot).
- Do **not** use `cargo update` in build commands.
- Set `CARGO_INCREMENTAL=0` (already in the manifest template).
- Use a specific `tag:` + `commit:` in the git source entry, not a branch ref.

---

## Self-Hosted Repos: No Rules (You Own It)

For the self-hosted alpha repo there are no external rules. However, for credibility and user
safety:

- Sign the repo with a GPG key (users can verify).
- Provide a clear disclaimer: "Alpha builds may be unstable / crash / contain placeholder content."
- Use a distinct App ID suffix (`.Alpha`) to prevent confusion with the stable build.

---

## License Compatibility Check

Unhaunter is dual-licensed MIT/Apache-2.0. For Flathub:

- The `project_license` in `metainfo.xml` should say `MIT OR Apache-2.0` (SPDX expression).
- The `metadata_license` (for the metainfo XML itself) should be `CC0-1.0`.
- All Cargo dependencies are compiled in; they must all be compatible with redistribution
  (MIT, Apache-2.0, BSD, etc.). The `cargo deny` tool (`cargo deny check licenses`) can audit
  this automatically.
- Audio, image, and font assets need explicit license files or README notes.

### Auditing Asset Licenses

Create a `LICENSE-ASSETS.md` or `NOTICE.md` that lists all third-party assets, their origin,
and their license. This is good FOSS practice regardless of Flathub, but reviewers may ask
for it.
