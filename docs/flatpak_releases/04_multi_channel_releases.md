# 04 – Multi-Channel Releases: stable / beta / alpha

## Goal

Maintain three independent update channels so different groups of users can opt into different
levels of stability:

| Channel | Audience | Cadence | Source branch |
|---------|----------|---------|---------------|
| **stable** | Everyone / default | Rare, well-tested releases | `main` (tagged `v0.X.X`) |
| **beta** | Testers, enthusiasts | Moderate, milestone releases | `beta` (tagged `v0.X.X-beta.N`) |
| **alpha** | Developers, bug hunters | Frequent, possibly broken | `next` or `alpha` branch |

---

## How Flatpak Branches Map to OSTree Refs

An OSTree ref has this shape:

```
app/<app-id>/<arch>/<branch>
```

For example:

```
app/io.github.deavid.unhaunter/x86_64/stable
app/io.github.deavid.unhaunter/x86_64/beta
app/io.github.deavid.unhaunter.Alpha/x86_64/stable
```

The default branch (when a user just runs `flatpak install`) is `stable`. You publish to
other branches via the `--default-branch` flag in `flatpak-builder` or by declaring `branch:`
in the manifest.

---

## Flathub: stable + beta in One App ID

Flathub supports exactly **two branches** per app: `stable` (default) and `beta`.

### stable channel

The `master` (or `main`) branch of the app's Flathub repo
(`github.com/flathub/io.github.deavid.unhaunter`) is built as `stable`. Every commit to that
branch triggers Flathub's CI and eventually publishes to the stable branch.

Users install with:

```bash
flatpak install flathub io.github.deavid.unhaunter
# or explicitly:
flatpak install flathub io.github.deavid.unhaunter//stable
```

### beta channel

Create a `beta` branch in `github.com/flathub/io.github.deavid.unhaunter`. Flathub watches it
and publishes to the `beta` branch of the same Flatpak app.

In the manifest on the `beta` branch, add:

```yaml
branch: beta
```

Users install with:

```bash
flatpak install flathub io.github.deavid.unhaunter//beta
```

Auto-updates work independently: stable users get stable updates; beta users get beta updates.

---

## Alpha Channel: Separate App ID on a Self-Hosted Repo

Flathub does not support a third branch, so alpha needs its own setup:

- **App ID:** `io.github.deavid.unhaunter.Alpha`
- **Hosted at:** a self-hosted OSTree repo (e.g. on GitHub Pages)

The manifest for alpha is a copy of the stable one with these changes:

```yaml
app-id: io.github.deavid.unhaunter.Alpha
branch: alpha
```

And in the `.desktop` / `.metainfo.xml` files, change the displayed name to include "(Alpha)"
so users can tell them apart on their desktop.

Users set up the alpha remote once:

```bash
flatpak remote-add --user unhaunter-alpha \
  https://deavid.github.io/unhaunter-flatpak/
flatpak install unhaunter-alpha io.github.deavid.unhaunter.Alpha
```

---

## Naming Convention Summary

| Channel | App ID | Manifest `branch:` | Distribution |
|---------|--------|---------------------|--------------|
| stable | `io.github.deavid.unhaunter` | `stable` (default, can omit) | Flathub |
| beta | `io.github.deavid.unhaunter` | `beta` | Flathub beta branch |
| alpha | `io.github.deavid.unhaunter.Alpha` | `alpha` | Self-hosted repo |

---

## Triggering a Release on Each Channel

Because Flathub's CI is triggered by commits to the app's GitHub repo, the typical flow is:

1. **stable release**: CI (GitHub Actions) on the main repo detects a `v0.X.X` tag → updates
   `cargo-sources.json` + tag in the Flathub app repo's `master` branch → Flathub builds.

2. **beta release**: CI detects a `v0.X.X-beta.N` tag → updates the `beta` branch of the
   Flathub app repo → Flathub builds.

3. **alpha release**: CI on any push to the `next` branch → builds locally, exports to OSTree
   repo, pushes the repo to GitHub Pages (or wherever).

This is documented as a *suggestion only* here; the CI scripts are intentionally left unmodified
per the project's current preference. The workflow files would need new jobs added in
`.github/workflows/release-*.yml` to drive Flathub repo updates.

---

## User Experience of Switching Channels

A user who wants to move from stable to beta:

```bash
flatpak install flathub io.github.deavid.unhaunter//beta
```

Flatpak allows multiple branches of the same app to be installed simultaneously, but most users
will run one. The branch is tracked in the user's Flatpak installation, so `flatpak update` keeps
them on their chosen branch.

A user who wants both stable and alpha at once can install both because the App IDs differ
(`io.github.deavid.unhaunter` vs `io.github.deavid.unhaunter.Alpha`); they will show as
separate items in the software centre.
