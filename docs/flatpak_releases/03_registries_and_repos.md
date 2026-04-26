# 03 – Registries and Repositories: Where to Publish

## The Landscape

There are three practical options for distributing Flatpak apps. They are not mutually exclusive.

| Option | Discoverability | Auto-update | FOSS friendly | Effort |
|--------|----------------|-------------|---------------|--------|
| **Flathub** | Excellent (built into most distros) | Yes | Yes (FOSS track) | Medium (one-time review) |
| **Self-hosted OSTree repo** | Low (users must add it manually) | Yes | Fully self-controlled | Low–Medium |
| **Flathub Beta** | Good (Flathub app, beta branch) | Yes | Yes | Same as Flathub |

---

## Option 1: Flathub (Recommended for Discoverability)

[Flathub](https://flathub.org) is the dominant Flatpak app store. It is:

- Pre-configured as a remote in GNOME Software, KDE Discover, and most desktop distros.
- Used by millions of Linux users.
- Free for FOSS apps.

### How Flathub Works

1. You submit a manifest to `github.com/flathub/<app-id>` (Flathub creates this repo for you
   after approval).
2. Flathub's build infrastructure (buildbot) clones your app repo, builds the Flatpak inside
   a controlled sandbox, and publishes it to `https://dl.flathub.org/repo/`.
3. Users install via: `flatpak install flathub io.github.deavid.unhaunter`
4. Updates happen automatically when Flathub rebuilds a newer commit on your app repo's `master`
   branch.

### FOSS vs Proprietary Track

Flathub has two tracks:

- **FOSS track** (default for open-source games): free, no fees, appears in all software centres.
- **Proprietary / non-free track**: allowed but displayed with a warning; some distros may filter
  it. Unhaunter is FOSS (MIT/Apache-2.0), so it qualifies for the FOSS track with no caveats.

### Recommendation

**Start with Flathub for the stable channel.** It gives you the widest reach with zero per-user
setup. People browsing GNOME Software or KDE Discover will see the game without knowing it's a
Flatpak; they just click Install.

---

## Option 2: Flathub Beta Branch

Flathub supports publishing a `beta` branch alongside `stable` within the same app ID. Users
opt into it with:

```bash
flatpak install flathub io.github.deavid.unhaunter//beta
```

You publish to the beta branch by pushing to a `beta` branch in your Flathub app GitHub repo
(i.e. `github.com/flathub/io.github.deavid.unhaunter`, branch `beta`). Flathub's buildbot
watches it and rebuilds automatically.

**Appropriate for:** the `unhaunter-beta` channel.

---

## Option 3: Self-Hosted OSTree Repository

For maximum control (especially for an `alpha` channel that Flathub doesn't officially
support), you can host your own Flatpak repository:

1. Run `flatpak-builder` locally or in CI.
2. Export the result into an OSTree repo: `flatpak build-update-repo repo/`
3. Host the `repo/` directory on a static file server (GitHub Pages, S3, self-hosted nginx, etc.).
4. Users add it once: `flatpak remote-add --user unhaunter-alpha https://your-host/flatpak/`
5. Then install: `flatpak install unhaunter-alpha io.github.deavid.unhaunter`

For the alpha channel, use a **separate App ID** to avoid confusion:
`io.github.deavid.unhaunter.Alpha`. This way stable, beta, and alpha can coexist on the same
machine.

**Downside:** users must manually add the remote. Discovery is zero unless you advertise it.

**Tool to know:** [`flat-manager`](https://github.com/flatpak/flat-manager) is the same tool
Flathub uses internally. It adds an HTTP API for pushing builds to a repo. Overkill for early
experimentation; useful once you automate CI publishing to a self-hosted repo.

---

## Recommended Channel Strategy

| Channel | App ID | Distribution | Branch / Method |
|---------|--------|--------------|-----------------|
| stable | `io.github.deavid.unhaunter` | Flathub | `master` branch of Flathub app repo |
| beta | `io.github.deavid.unhaunter` | Flathub | `beta` branch of Flathub app repo |
| alpha | `io.github.deavid.unhaunter.Alpha` | Self-hosted | Own OSTree repo on GitHub Pages or similar |

This approach means:

- Stable and beta users get Flathub's infrastructure (signed builds, CDN, GNOME Software
  integration) with zero setup.
- Alpha users add a custom remote once; they get rapid updates pushed directly from CI.
- The alpha App ID suffix `.Alpha` makes it clear to users what they are installing.

---

## Existing Game Examples on Flathub

Looking at current Flathub listings for reference:

- `com.valvesoftware.Steam` – shows Flatpak works fine for heavy gaming workloads
- `io.itch.itch` – itch.io client, FOSS game store wrapper
- Various indie games: `net.velvetcactus.vcmi`, `io.github.dosbox-staging.dosbox-staging`

Searching [https://flathub.org/apps/category/Game](https://flathub.org/apps/category/Game) shows
hundreds of games, including many small FOSS titles similar to Unhaunter in scope.
