# 05 – Account Setup and Human Submission Steps

## What You Need Before Submitting

1. **A GitHub account** — Flathub uses GitHub for all app submissions and repo management.
   Your GitHub username is `deavid`, so this is already covered.

2. **A GPG key for signing** (optional but good practice for a self-hosted repo; not required
   for Flathub itself since Flathub re-signs everything with their own key).

3. **A local test build** — Flathub will reject your submission if it cannot build cleanly.
   Always verify locally first.

---

## One-Time Setup: Getting on Flathub

### Step 1 – Prepare the App Repo Locally

Create a new Git repository that will become the Flathub app repo:

```
io.github.deavid.unhaunter/
├── io.github.deavid.unhaunter.yml   ← the manifest
└── cargo-sources.json               ← generated from Cargo.lock
```

The metadata files (`metainfo.xml`, `.desktop`, icons) should live in the *game* source repo
and be referenced in the manifest's `sources:` section (via the git source entry). Do not
duplicate them in the Flathub repo.

### Step 2 – Validate Everything Locally

```bash
# Install flatpak-builder and the runtime/SDK
flatpak install flathub org.freedesktop.Platform//24.08 \
                         org.freedesktop.Sdk//24.08 \
                         org.freedesktop.Sdk.Extension.rust-stable//24.08

# Build the Flatpak
flatpak-builder --force-clean --repo=local-repo build-dir \
  io.github.deavid.unhaunter.yml

# Test-install and run it
flatpak --user remote-add --no-gpg-verify local local-repo
flatpak --user install local io.github.deavid.unhaunter
flatpak run io.github.deavid.unhaunter
```

Fix any errors before submitting.

### Step 3 – Submit to Flathub

Go to: **[https://github.com/flathub/flathub/issues/new/choose](https://github.com/flathub/flathub/issues/new/choose)**

Choose **"App submission"** and fill in:

- App ID: `io.github.deavid.unhaunter`
- Link to your app repo (a public GitHub fork works, or just attach the files)
- Brief description of the app
- Confirmation that you are the developer or have permission to submit

Alternatively, the newer workflow is via the **Flathub website** itself:
1. Go to [https://flathub.org/apps/submission](https://flathub.org/apps/submission)
2. Sign in with GitHub
3. Submit the App ID + manifest repository URL

### Step 4 – Review Process

Flathub maintainers will review:

- That the app builds correctly on their infrastructure.
- That the manifest follows their guidelines (no unnecessary permissions, correct metadata).
- That AppStream XML is valid.
- That the app runs and is functional.

You will receive comments as GitHub review feedback on the submission PR. Respond to any
requested changes, push updates, and wait for approval.

**Timeline:** review typically takes 1–4 weeks for new apps.

### Step 5 – Flathub Creates Your App Repo

Once approved, Flathub creates `github.com/flathub/io.github.deavid.unhaunter` and gives
you write access to it. This is the repo you push updates to from now on.

---

## Ongoing Updates (After Initial Approval)

Subsequent updates do **not** require a manual review. You:

1. Update the tag/commit in `io.github.deavid.unhaunter.yml`.
2. Regenerate `cargo-sources.json` if `Cargo.lock` changed.
3. Push to `master` of `github.com/flathub/io.github.deavid.unhaunter`.
4. Flathub's CI builds and publishes automatically.

There is a 3-day verification window before the update goes fully live. Users on auto-updates
will receive it within 3 days of your push.

---

## Setting Up the Beta Branch

Once the initial stable submission is live:

```bash
cd flathub-app-repo
git checkout -b beta
# Edit manifest: add `branch: beta`
git add -A && git commit -m "Setup beta branch"
git push origin beta
```

Flathub automatically picks up the `beta` branch and starts building it.

---

## Setting Up a Self-Hosted Alpha Repo

You need a GPG key to sign the OSTree repo if you want verified installs (recommended):

```bash
gpg --gen-key
# use email: unhaunter-flatpak@yourdomain or similar
gpg --export --armor > unhaunter-flatpak.pub.asc
```

When exporting the repo:

```bash
flatpak build-update-repo --gpg-sign=<KEY-ID> --generate-static-deltas repo/
```

Publish the public key alongside the repo so users can import it:

```bash
flatpak remote-add --user --gpg-import=unhaunter-flatpak.pub.asc \
  unhaunter-alpha https://deavid.github.io/unhaunter-flatpak/
```

Without a GPG key, use `--no-gpg-verify` which works but shows a warning.

---

## Reference Links

- Flathub submission guidelines: [https://docs.flathub.org/docs/for-app-authors/submission/](https://docs.flathub.org/docs/for-app-authors/submission/)
- Flathub quality guidelines: [https://docs.flathub.org/docs/for-app-authors/appstream/](https://docs.flathub.org/docs/for-app-authors/appstream/)
- Flatpak documentation: [https://docs.flatpak.org/](https://docs.flatpak.org/)
- flatpak-builder-tools (cargo generator): [https://github.com/flatpak/flatpak-builder-tools](https://github.com/flatpak/flatpak-builder-tools)
