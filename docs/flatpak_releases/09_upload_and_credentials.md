# 09 – Upload Automation and Safe Credential Management

## The Two Upload Targets

### A) Flathub (stable + beta)

You do not push a built `.flatpak` binary to Flathub. Instead, you **push a commit** to the
Flathub GitHub app repo (`github.com/flathub/io.github.deavid.unhaunter`). Flathub's own
build infrastructure (buildbot) then fetches your source, builds the Flatpak in its own
sandboxed environment, and publishes it to `dl.flathub.org`. You never touch Flathub's servers
directly.

### B) Self-Hosted OSTree Repo (alpha)

You push built OSTree repo files to a file host (e.g. GitHub Pages, or any static web server).
The user's Flatpak client then pulls from that URL directly.

---

## Automating Flathub Pushes from GitHub Actions

The pattern is:

1. After a tagged release, a CI job checks out the Flathub app repo.
2. It updates the `tag:` and `commit:` fields in the manifest.
3. It regenerates `cargo-sources.json`.
4. It commits and pushes to the Flathub app repo's `master` (or `beta`) branch.

This requires a **deploy key** with write access to the Flathub app repo.

### Setting Up the Deploy Key

```bash
# Generate a dedicated key pair (no passphrase for CI)
ssh-keygen -t ed25519 -f flathub-deploy-key -N "" \
  -C "unhaunter-ci-flathub-push"
```

- **Add the public key** (`flathub-deploy-key.pub`) as a deploy key to
  `github.com/flathub/io.github.deavid.unhaunter` (Settings → Deploy keys → Add deploy key;
  enable "Allow write access").
- **Store the private key** (`flathub-deploy-key`) as an **encrypted GitHub Actions secret**
  in the `deavid/unhaunter` repo:
  - Settings → Secrets and variables → Actions → New repository secret
  - Name: `FLATHUB_DEPLOY_KEY`
  - Value: contents of `flathub-deploy-key` (the private key)

Delete the local key files after copying the values to GitHub.

### Suggested CI Job Snippet (not wired in yet)

```yaml
# To be added to .github/workflows/release-main.yml when ready
- name: Update Flathub app repo
  env:
    FLATHUB_DEPLOY_KEY: ${{ secrets.FLATHUB_DEPLOY_KEY }}
    VERSION: ${{ steps.get_version.outputs.VERSION }}
    COMMIT_SHA: ${{ github.sha }}
  run: |
    # Write deploy key
    mkdir -p ~/.ssh
    echo "$FLATHUB_DEPLOY_KEY" > ~/.ssh/flathub_key
    chmod 600 ~/.ssh/flathub_key
    export GIT_SSH_COMMAND="ssh -i ~/.ssh/flathub_key -o StrictHostKeyChecking=no"

    # Clone Flathub app repo
    git clone git@github.com:flathub/io.github.deavid.unhaunter.git flathub-app
    cd flathub-app

    # Regenerate cargo-sources.json
    python3 ../flatpak/flatpak-cargo-generator.py ../Cargo.lock -o cargo-sources.json

    # Update manifest tag and commit
    sed -i "s|tag: v.*|tag: v${VERSION}|" io.github.deavid.unhaunter.yml
    sed -i "s|commit: .*|commit: ${COMMIT_SHA}|" io.github.deavid.unhaunter.yml

    git config user.email "ci@github.com"
    git config user.name "Unhaunter CI"
    git add -A
    git commit -m "Release v${VERSION}"
    git push origin master
    # Clean up key
    rm ~/.ssh/flathub_key
```

> **Note:** This snippet is provided as documentation only. The CI files are not modified
> in the current state of the project.

---

## Automating Alpha (Self-Hosted OSTree) Pushes

For the alpha channel, the build artifact (an OSTree repo directory) is pushed to a static host.

### Option A: GitHub Pages

1. Create a separate repository `deavid/unhaunter-flatpak` with GitHub Pages enabled.
2. In CI, after running `flatpak-builder` and `flatpak build-update-repo`:

```yaml
- name: Publish alpha Flatpak repo to GitHub Pages
  env:
    FLATPAK_PAGES_KEY: ${{ secrets.FLATPAK_PAGES_KEY }}
  run: |
    mkdir -p ~/.ssh
    echo "$FLATPAK_PAGES_KEY" > ~/.ssh/pages_key
    chmod 600 ~/.ssh/pages_key
    export GIT_SSH_COMMAND="ssh -i ~/.ssh/pages_key -o StrictHostKeyChecking=no"

    git clone git@github.com:deavid/unhaunter-flatpak.git pages-repo
    rsync -a --delete flatpak-repo/ pages-repo/
    cd pages-repo
    git add -A
    git commit -m "Alpha release $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    git push origin gh-pages
    rm ~/.ssh/pages_key
```

### Option B: `flat-manager` + a VPS

`flat-manager` provides an HTTP API for pushing signed builds to a managed OSTree repo.
Credentials are a bearer token stored as a GitHub Actions secret. This is the approach used
internally by Flathub and is appropriate once the alpha channel sees real traffic.

---

## Secret Management Best Practices

| Secret | What it is | Where to store |
|--------|-----------|----------------|
| `FLATHUB_DEPLOY_KEY` | SSH private key for Flathub app repo | GitHub Actions secret |
| `FLATPAK_PAGES_KEY` | SSH private key for `unhaunter-flatpak` repo | GitHub Actions secret |
| `FLATPAK_GPG_PRIVATE_KEY` | GPG key for signing the self-hosted repo | GitHub Actions secret |
| `FLATPAK_GPG_KEY_ID` | Key fingerprint (public, not sensitive) | Can be in workflow YAML |

**Rules:**

- Never commit private keys or secrets to the repository.
- Use `chmod 600` on key files written to the runner's filesystem.
- Delete key files at the end of the job step that uses them (or use a `try/finally` pattern).
- Prefer short-lived deploy keys (one key per deployment target) over personal access tokens.
- Rotate keys if a workflow run's logs are publicly visible and you suspect exposure.

---

## Manual Upload as a Fallback

Until CI is wired up, you can trigger updates manually:

```bash
# Clone and update the Flathub app repo by hand
git clone git@github.com:flathub/io.github.deavid.unhaunter.git
cd io.github.deavid.unhaunter
# ... edit manifest, regen cargo-sources.json ...
git add -A && git commit -m "Release v0.X.X" && git push origin master
```

This is perfectly fine for the early stages of Flatpak adoption. You do not need CI automation
to maintain a Flathub listing; many apps are updated manually by their developers.
