# Hosting an Unhaunter Multiplayer Hub

This guide is for community members and developers who want to run the Unhaunter multiplayer infrastructure on a Linux
VPS. This stack allows players to create and join rooms using short codes instead of sharing their IP addresses.

**Status: Early / Unstable.** This infrastructure was implemented recently and keeps changing.

---

## What You're Deploying

Three separate programs work together in this stack:

```
Players ──HTTPS/HTTP──► Caddy Reverse Proxy
                         │
                         │  Local proxy (localhost:3000)
                         ▼
                       Hub (unhub)
                         ▲
                         │  persistent TCP (ProcMan connects OUT to Hub)
                         │
                    ProcMan (unprocman)
                         │
                         │  stdin/stdout (ProcMan spawns and manages these)
                         │
               Dedicated Servers (unhaunter_dedicated)
                         ▲
                         │
                    Players connect directly via TCP
                    (after Hub gives them the address)
```

| Program                   | What it does                                                                                                                                                                                           |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **`unhub`**               | The central API server. Players talk to it to create/join rooms. It never runs game code itself, but acts as a tracker, routing players to dedicated servers based on the room codes.                  |
| **`unprocman`**           | The Process Manager. Connects outbound to the Hub over TCP. It manages a pool of idle `unhaunter_dedicated` game servers, assigning rooms to them, and terminating/restarting them as needed.          |
| **`unhaunter_dedicated`** | The actual headless game server running Bevy (no GPU, no audio, just logic). Each game room is processed as its own OS process, securely bound to exactly one TCP port. Players directly connect here. |

---

## Deployment Strategy: Local Build + Ansible

Because building a headless Bevy server requires compiling the entire Rust and Bevy codebase, it is incredibly
resource-heavy (often locking up or using 100% of the CPU of smaller VPS nodes).

To keep things **fast, easy to update, and highly automated**, our standard hosting strategy uses **Ansible**:

1. You build the three core binaries locally on your development machine where you have a fast CPU.
2. An Ansible playbook pushes the binaries and the `assets/` folder to your VPS.
3. Ansible configures **Caddy** to automatically provision Let's Encrypt certificates and reverse-proxy traffic locally
   to the `unhub` API.
4. Ansible pre-configures and ties together the networking and authentication components, entirely bypassing any
   "bootstrap dance".

---

## Prerequisites

### On your Dev Machine (Local)

- You need this Git repository checked out.
- The `rust` toolchain installed to compile the binaries.
- Packages `ansible` and `rsync` installed for deployment.

### On your Server (VPS)

- A basic recent Linux distribution (Debian/Ubuntu recommended).
- A public IPv4 address with SSH access.
- A Domain Name pointing to the server's public IP (needed for Caddy to pull Let's Encrypt SSL certs).
- Open Ports: Make sure `80` / `443` (HTTP/HTTPS) and `12000-12100` (Game Servers) are open and accessible on the VPS.

---

## Step 1: Build the Binaries Locally

From the root directory on your local development machine, fully compile the required release binaries using Cargo:

```bash
cargo build --release -p unhub -p unprocman -p unhaunter --bin unhub --bin unprocman --bin unhaunter_dedicated
```

This may take some time. When complete, three binaries will exist in `target/release/`.

---

## Step 2: Deploy with Ansible

We have provided a fully prepared playbook inside `deploy/multiplayer.yml`.

By running this script, Ansible will remotely:

- Create a dedicated standard `unhaunter` user on your server.
- Copy your `assets/` and the newly built `target/release/` binaries to `/opt/unhaunter/`.
- Generate custom `hub_config.ron` and `procman_config.ron` files, locking in a shared pre-generated UUID ensuring
  immediate and safe internal auth.
- Bind `unhub` internally to `127.0.0.1:3000` for API isolation, closing off direct internet access.
- Install Caddy and mount it to your secure Domain Name.
- Setup auto-restarting systemd daemons for the background services.

### Running the Ansible Playbook

Create an inventory file `hosts.ini` (optional) containing your VPS IP, or simply construct the command inline.

Run the playbook from the root of the repository:

```bash
cd deploy/
ansible-playbook multiplayer.yml -i "YOUR_VPS_IP," -u YOUR_SSH_USER -e "domain=hub.yourdomain.com"
```

_Notes:_

- Replace `YOUR_VPS_IP` with the IP of your node. The trailing comma `,` is required for a dynamic host.
- Replace `YOUR_SSH_USER` with the administrative username on your VPS (e.g., `debian`, `ubuntu`, or `root`). It must
  have sudo privileges.
- Replace `hub.yourdomain.com` with your pre-configured and A-record mapped domain.

Once it completes successfully, **you are done**. The server is fully installed, certs are pulled, the Hub API is live
over HTTPS via Caddy, and `unprocman` has loaded a game server.

### Doing Updates

For developers, rolling out an update means literally running two commands:

1. `cargo build --release -p unhub -p unprocman -p unhaunter --bin unhub --bin unprocman --bin unhaunter_dedicated`
2. Run the Ansible script again.

Since Ansible is idempotent, it will skip all unchanged setup steps, instantly copy your new binaries and assets, and
restart the daemons.

---

## How It Works (Deep Dive)

### Config Files & UUID Bootstrapping

If you check `/opt/unhaunter/hub_config.ron` on your VPS, you will see two key components:

```ron
(
    api_bind: "127.0.0.1:3000",
    procman_bind: "127.0.0.1:11000",
    allowed_procman_uuids: ["0ba7eb..."]
    // ...
)
```

The config locks both `unhub`'s APIs to `127.0.0.1`. Meaning outside traffic must hit the reverse proxy mapping to touch
it. When `unprocman` starts, it expects its `installation_id` to be explicitly added to `allowed_procman_uuids`. The
ansible playbook syncs these UUIDs upon the very first rollout to avoid any friction.

### Caddy Reverse Proxy & HTTPS

We use `caddy` precisely for its lightweight footprint and native Let's Encrypt support. Its mapping, written to
`/etc/caddy/Caddyfile`, is automatically mapped simply like so:

```caddyfile
hub.yourdomain.com {
    reverse_proxy 127.0.0.1:3000
}
```

### The Game Port Matrix

**The `12000 - 12100` range is exposed.** When `unprocman` loads pool processes, it sequentially cycles through these
ports. When clients obtain the matching port via the secure API URL call via `caddy (443/3000)`, their final link
directly targets `YOUR_VPS_IP:12001`. That's why Caddy does not route these - performance routing happens over naked TCP
straight into Bevy!

### Diagnosing Issues & Logs

System logs capture all output out of both tools. Since they are run securely as the low interaction `unhaunter` user
via systemd, finding crashes uses native linux tools.

By default, it is highly recommended to monitor both the Hub and the Process Manager (and its game servers)
simultaneously:

```bash
# Follow logs for both services at once (Recommended)
sudo journalctl -u unhub.service -u unprocman.service -f
```

You can also isolate them if needed:

```bash
# To follow error logs live for ONLY unhub
sudo journalctl -u unhub.service -f

# Looking exclusively at ONLY ProcMan server logs
sudo journalctl -u unprocman.service -f
```

_(Note: Logs generated from the Bevy processes spawned within `unhaunter_dedicated` bubble directly up and are appended
to the logs of `unprocman.service` inside of bracketed process tags)._

---

## Testing Connectivity Setup

To verify everything has launched properly, query your Hub endpoint publicly:

```bash
curl https://hub.yourdomain.com/health
```

Expected Response:

```json
{ "hub_version": "...", "uptime_seconds": ... }
```

### Join with a game client

Run the game on your dev machine, routing directly to the custom URL we configured via Caddy:

```bash
cargo run --bin unhaunter_game -- --hub-url https://hub.yourdomain.com
```

In the main menu, navigate to **Play Online**, and it should securely connect via your HTTPS wrapper.

---

## Manual Execution and Tweaking (Advanced)

If you need to test the stack without Ansible, or tweak things manually on a node, here's how the inner configuration
system natively behaves:

### Config Variables (Manual Bootstrap)

Running `unhub` once by itself creates a `hub_config.ron`. Running `unprocman` creates `procman_config.ron`.

If you were doing this entirely manually (without the Ansible templates), you must deal with the **Bootstrap Dance**:

1. Run `unprocman` once. Let it fail to connect. Note the random `installation_id` (a UUID) it wrote to
   `procman_config.ron`.
2. Add that UUID to `hub_config.ron`'s `allowed_procman_uuids` list.
3. Restart both services (they do not hot-reload these configurations).

`procman_config.ron` Reference:

- `hub_addr`: Where procman looks for the hub. Must match the IP/port of the Hub's `procman_bind`.
- `public_addr`: **CRITICAL.** This must be the public IP or domain of your machine. This string is what the Hub
  physically returns to your players to connect to the dedicated server.
- `port_range`: A tuple like `(12000, 12100)`. Every active game room securely grabs an exclusive port here.
- `game_binary_path`: A string relative path mapped to the `unhaunter_dedicated` executable.
- `idle_pool_size`: Number of ready-to-go headless servers. Default is 1. Bevy pulls ~10% CPU per idle instance.

### System Failure Consequences

All room association state inside `unhub` is currently held strictly in-memory. If `unhub` crashes or restarts, all
active room mappings are lost. However, pre-existing games that have already loaded their players inside
`unhaunter_dedicated` will **continue unharmed**, as those are direct TCP links bypassing the Hub. Players just won't be
able to generate new rooms until the hub returns.

## Other useful stuff

Since we deployed the applications as `systemd` services using Ansible, all logs are automatically collected by
`journald`.

You can view the logs on your server using the `journalctl` command:

**To view live logs for `unhub`:**

```bash
sudo journalctl -u unhub.service -f
```

**To view live logs for `unprocman` (which also includes output from the game servers):**

```bash
sudo journalctl -u unprocman.service -f
```
