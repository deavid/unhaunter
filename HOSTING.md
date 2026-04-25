# Hosting an Unhaunter Multiplayer Hub

This is the operations runbook for deploying and updating the Unhaunter multiplayer stack on a Linux VPS.

Status: active runbook for the current multi-version architecture.

---

## What Gets Deployed

The stack has two singleton infrastructure services and a versioned dedicated-server library:

1. unhub: public HTTP API (behind Caddy) + procman TCP control endpoint.
2. unprocman: process manager that scans a server library and spawns dedicated servers on demand.
3. unhaunter_dedicated bundles in a library tree under /opt/unhaunter/library.

Network flow:

```text
Client --HTTPS--> Caddy --> unhub
                     |
                     +-- room ticket / address --> Client --TCP--> spawned unhaunter_dedicated

unprocman --persistent TCP--> unhub (outbound control channel)
```

Important: unprocman keeps zero idle servers. It only spawns when unhub sends CreateRoom.

---

## Library Layout on VPS

```text
/opt/unhaunter/
  bin/
    unhub
    unprocman
  hub_config.ron
  procman_config.ron
  library/
    stable/<version>/
      unhaunter_dedicated
      assets/
      VERSION
      PROTOCOL_HASH
    beta/<version>/
      unhaunter_dedicated
      assets/
      VERSION
      PROTOCOL_HASH
    alpha/
      unhaunter_dedicated
      assets/
      VERSION
      PROTOCOL_HASH
    dev/
      unhaunter_dedicated
      assets/
      VERSION
      PROTOCOL_HASH
  secrets/
    procman_hmac_secret
```

Channel behavior:

1. stable and beta: version history retained with cleanup rules.
2. alpha and dev: single rolling copy, overwritten on deploy.

---

## Deployment Model

Use one playbook for everything:

1. Pushes local infrastructure binaries (unhub, unprocman) to /opt/unhaunter/bin.
2. Pushes local dev dedicated binary + assets to /opt/unhaunter/library/dev.
3. Fetches stable and beta dedicated bundles from GitHub Releases.
4. Fetches rolling alpha bundle from alpha-latest release.
5. Generates VERSION and PROTOCOL_HASH files in each deployed channel directory.
6. Applies retention cleanup for stable and beta channels.
7. Sends SIGHUP to unprocman to rescan library without restart (if active).
8. Templates hub/procman config and systemd services.
9. Configures Caddy reverse proxy for TLS.

---

## Prerequisites

Local machine:

1. Repository checked out.
2. Rust toolchain installed.
3. ansible installed.
4. rsync installed.

VPS:

1. Linux host with sudo-capable SSH user.
2. Public DNS A/AAAA record for your hub domain.
3. Ports open: 80, 443, and game port range 12000-12100/TCP.

---

## Build and Deploy

Build local binaries (from repository root):

```bash
cargo build --release -p unhub -p unprocman -p unhaunter --bin unhub --bin unprocman --bin unhaunter_dedicated
```

Deploy (from repository root):

```bash
ansible-playbook deploy/multiplayer.yml -i "YOUR_VPS_IP," -u YOUR_SSH_USER -e "domain=hub.yourdomain.com"
```

Rotate HMAC secret (invalidates existing connection tickets):

```bash
ansible-playbook deploy/multiplayer.yml -i "YOUR_VPS_IP," -u YOUR_SSH_USER -e "domain=hub.yourdomain.com rotate_hmac_secret=true"
```

Notes:

1. Keep the trailing comma in -i "HOST," so Ansible treats it as an inline host list.
2. Run from repository root so playbook relative paths resolve correctly.

---

## Config Reference

hub_config.ron is templated with:

1. api_bind = 127.0.0.1:3000
2. procman_bind = 127.0.0.1:11000
3. allowed_procman_uuids = [server-derived UUID]

procman_config.ron is templated with:

1. hub_addr = 127.0.0.1:11000
2. public_addr = inventory hostname
3. installation_id = server-derived UUID
4. ticket_hmac_secret = (64-char hex secret; Ansible reads /opt/unhaunter/secrets/procman_hmac_secret and inlines it
   here)
5. port_range = (12000, 12100)
6. library_dir = /opt/unhaunter/library
7. max_total_instances = 8

Legacy keys game_binary_path and idle_pool_size are obsolete and must not be used.

---

## Manual Bootstrap (Advanced)

The playbook handles bootstrap automatically. If you run services manually:

1. Create /opt/unhaunter/bin and /opt/unhaunter/library/[channel] directories.
2. Place unhub and unprocman in /opt/unhaunter/bin.
3. Place dedicated bundles in library directories.
4. For each bundle/channel directory, generate metadata:

   ```bash
   ./unhaunter_dedicated --print-version > VERSION
   ./unhaunter_dedicated --print-protocol-hash > PROTOCOL_HASH
   ```

5. Ensure hub_config.ron includes procman installation_id in allowed_procman_uuids.
6. Ensure procman_config.ron points to library_dir, not a single binary path.
7. Start unhub and unprocman.

When changing library contents while unprocman is running, send reload signal:

```bash
sudo systemctl kill -s HUP unprocman
```

---

## Operations and Verification

Health check:

```bash
curl https://hub.yourdomain.com/health
```

Service logs:

```bash
sudo journalctl -u unhub.service -u unprocman.service -f
```

Inspect generated metadata quickly:

```bash
sudo find /opt/unhaunter/library -maxdepth 3 -type f \( -name VERSION -o -name PROTOCOL_HASH \) -print
```

---

## Update Workflow

Typical update:

```bash
cargo build --release -p unhub -p unprocman -p unhaunter --bin unhub --bin unprocman --bin unhaunter_dedicated && \
ansible-playbook deploy/multiplayer.yml -i "hub.unhaunter.com," -u debian -e "domain=hub.unhaunter.com"
```

Security rotation update:

```bash
cargo build --release -p unhub -p unprocman -p unhaunter --bin unhub --bin unprocman --bin unhaunter_dedicated && \
ansible-playbook deploy/multiplayer.yml -i "hub.unhaunter.com," -u debian -e "domain=hub.unhaunter.com rotate_hmac_secret=true"
```

HMAC secret storage:

1. File path: /opt/unhaunter/secrets/procman_hmac_secret
2. Owner/mode: unhaunter:unhaunter, 0600
