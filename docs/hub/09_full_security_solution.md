# Hub End-to-End Security Solution

- **Date:** 2026-02-17
- **Status:** Draft / Consolidated Security Model
- **Depends on:** `01_proposal.md`, `05_critical_user_journeys.md`, `08_rate_limits.md`, [HOSTING.md](../../HOSTING.md)

---

## 1. Objective

Define a practical, layered security model for the Hub stack that:

1. Prevents trivial REST-only denial-of-service.
2. Survives realistic multi-IP scripted abuse.
3. Minimizes resource waste during player spikes.
4. Can run unattended with predictable operator actions when needed.

This document is the full model across all fronts (architecture, protocol, rate limiting, lifecycle, infrastructure,
observability, and response).

---

## 2. Threat Model (What We Defend Against)

### In scope

- API flooding of room creation.
- Multi-IP scripted room hoarding.
- Fake clients that never become real players.
- Persistent TCP clients that attempt to pin capacity.
- Abuse by UUID churn (no account system).

### Out of scope

- Volumetric L3/L4 DDoS that saturates uplink.
- Kernel-level exploit chains.
- Nation-state adversaries with effectively unlimited rotating infrastructure.

---

## 3. Capacity Baseline (Measured)

Using your VPS test results:

- Dedicated at 1 Hz idle lobby:
  - **CPU:** ~0.11%
  - **RAM:** ~115 MiB per process
- With 8 cores, CPU headroom is not the binding constraint at 1 Hz.
- **RAM is the hard cap:** approximately 213 dedicated processes at once.

**Security implication:** All anti-DoS policy must be designed around protecting ~213 room slots (RAM-bound), not
CPU-bound theoretical maxima. We are "RAM-poor" but "CPU-rich."

---

## 4. The Strategy: Flipping the Economic Asymmetry

In a naive REST-only model, the attacker has a **3000:1 advantage**:

- **Attacker cost:** ~100ms (one HTTP request).
- **Your cost:** 5 minutes (300,000ms) of reserved RAM for one room.

This is an **unwinnable equation.** An attacker with modest hardware can generate requests faster than you can reclaim
resources.

Our solution reconstructs the economics by moving the "cost of entry" from the Hub REST API to the ProcMan TCP
handshake. We make the attacker prove liveness and pay a computational price upfront, **before** a single byte of your
precious 213 RAM slots is consumed.

---

## 5. Final Security Architecture

## 5.1 Primary layer: Server Pool + Room Recycling (Idea 1)

- ProcMan maintains an idle pool of already-running dedicated servers.
- A room code is assigned to an idle server instead of spawning a new process.
- If unused, room code is wiped and server returns to idle state quickly.
- No expensive spawn/kill churn in the hot path.

### Tick policy by room state

- Room = None: 1 Hz
- Lobby, 0-1 players: 1 Hz
- Lobby, 2+ players: 5 Hz
- Mission, 1 player: 15 Hz
- Mission, 2+ players: 60 Hz

Goal: keep idle and near-idle state cheap, escalate only when human gameplay is present.

## 5.2 Secondary layer: ProcMan as in-path TCP proxy (Idea 2)

- Hub returns ProcMan address for room connectivity.
- ProcMan stays in data path for full session.
- Client must complete TCP handshake and initial protocol negotiation before assignment finalization.
- ProcMan can inspect and gate early handshake, then relay raw TCP bytes thereafter.

Security value:

- **Attack Class Jump:** Shifts the attack from **Stateless** (trivially scriptable with `curl` or HTTP clients) to
  **Stateful** (requires a custom distributed TCP application with persistent connections, heartbeat logic, and error
  recovery).
- Room allocation is no longer driven by REST call alone.
- Client liveness is proven before expensive capacity is consumed.
- Fast revocation becomes safe and immediate.

## 5.3 Tertiary layer: Economic friction (PoW)

- PoW challenge moved to ProcMan connection path (not only REST path).
- Target cost: ~1 second per successful room-claim attempt.
- Single-use challenges with short TTL.

Security value:

- Every new claim (or re-claim after drop) costs real computation.
- Reconnection churn becomes expensive for attackers.

## 5.4 Quaternary layer: Per-IP caps

- Max active rooms per IP = 2.
- Max outstanding challenges per IP = 2.
- Max outstanding challenges per UUID = 2.

Security value:

- Hard-limits blast radius from one IP.
- Forces distributed infrastructure for large-scale abuse.

---

## 6. End-to-End Flow (Final)

1. Client calls Hub create-room.
2. Hub creates lightweight reservation only (no dedicated assignment yet).
3. Reservation has short timeout (for example 5 seconds) if no ProcMan TCP progress.
4. Client connects to ProcMan TCP endpoint for that room.
5. ProcMan issues/validates PoW challenge (~1 second target effort).
6. On successful PoW + protocol sanity checks, ProcMan binds client to idle dedicated server.
7. ProcMan remains in path and enforces heartbeat/liveness policy.
8. If liveness fails, room is reclaimed immediately and capacity returns to pool.

---

## 7. Attack Cost After Full Model

To pin all ~213 room slots under per-IP cap = 2, attacker needs roughly:

- ~107 IPs maintaining parallel TCP sessions.
- Custom concurrent client implementing handshake, PoW solve, heartbeat.
- Continuous operation (stateful attack), not stateless HTTP spam.
- Re-solve PoW on reconnect churn.

This shifts attack complexity from "curl loop" to "operated distributed system".

### When do rooms get reclaimed?

| Scenario                        | Old Model                   | New Model                          | Impact                |
| ------------------------------- | --------------------------- | ---------------------------------- | --------------------- |
| Unused room (no TCP connect)    | 300s timeout                | 5s fast expiry                     | 60× faster            |
| Room with dropped client        | 300s idle timeout           | Immediate (heartbeat fail)         | Instant               |
| 1 IP holding 2 rooms (attacker) | 300s each = 600s total hold | Re-solve PoW every 5s or lose them | Forces constant churn |

In the old model, one IP could park 2 servers for 5 minutes. In the new model, an unused server is reclaimed in 5
seconds. Unused = "no one is playing a real game here."

---

## 8. Abuse Detection and Auto-Response (Required for Unattended Operation)

Without this section, the stack is hardened but not fully self-healing.

## 8.1 Behavioral signals: The "Smoking Gun"

The unambiguous signal of abuse is **rooms that never become real games.** These are the telltale signs:

- **"Zero-mission zone":** An IP creates 10 rooms and 0 missions ever start. Bots create lobbies they never populate.
  Humans create rooms and friends join them.
- **Lobby only (1 player forever):** A room sits in lobby state with the attacker as the only player, and no second
  human ever connects. This is a bot holding a slot.
- **PoW churn without follow-through:** Client solves a challenge but then never actually TCP-connects to the assigned
  server. They're probing without commitment.
- **Frequent rapid disconnect/reconnect:** Attacker connects, gets rejected or kicked, immediately reconnects. This is
  not legitimate player behavior.

These signals are nearly impossible to fake convincingly because they require coordinating **actual human gameplay**
with your attack.

## 8.2 Auto-mitigation policy

Example baseline:

- Temporary IP deny for 1 hour if repeated room creation never results in real sessions.
- Escalate to longer deny on repeated offense.
- Immediate reclaim on heartbeat failure.
- Optional cool-down before same IP can create another room.

## 8.3 Why this matters

With auto-ban heuristics, high-effort attacks decay automatically over time instead of requiring operator wake-up.

---

## 9. Infrastructure Hardening (Host and Network)

## 9.1 Reverse proxy and edge

- Place Hub HTTP behind reverse proxy.
- Enable request size limits and conservative per-IP connection limits.
- Preserve real client IP only when proxy is trusted.

## 9.2 OS and service limits

- Set file descriptor limits for Hub/ProcMan.
- Tune listen backlog and TCP defaults for expected concurrency.
- Run services under systemd with restart policies.
- Ensure child-process cleanup on ProcMan restart/failure.

## 9.3 Firewall

- Allow only required ports (Hub API, ProcMan TCP, dedicated range as needed).
- Keep administration endpoints private.

---

## 10. Observability and Alerts

Minimum telemetry:

- Idle pool size over time.
- Active rooms by state.
- Reclaims by reason (timeout, heartbeat, policy).
- Challenge issued/solved/failed rates.
- Room create success/failure by error code.
- Top IPs by room creation and failed challenges.

Minimum alerts:

- Idle pool near zero for sustained period.
- High ratio of created rooms that never become mission.
- Sudden spike in challenge failures/timeouts.
- ProcMan disconnect from Hub.

---

## 11. Incident Response Runbook

1. Confirm whether event is app-layer abuse or network-layer saturation.
2. If app-layer:
   - Enable stricter temporary thresholds.
   - Apply/verify automatic denies.
   - Manually block top abusive IP ranges if needed.
3. If network-layer:
   - Shift to provider/CDN mitigation controls.
4. Verify recovery:
   - Idle pool restored.
   - Legitimate room creation success back to normal.
5. Capture indicators and update policy thresholds.

---

## 12. Rollout Plan

### Phase 1 (must-have)

- Server pool + room code recycling.
- Tick-rate state policy.
- Fast unused reservation expiry.

### Phase 2 (must-have)

- ProcMan full-session TCP proxy.
- ProcMan-side PoW handshake challenge (~1s target).
- Heartbeat-driven immediate reclaim.

### Phase 3 (must-have for unattended)

- Behavioral scoring.
- Automatic temporary denies.
- Metrics + alerts.

### Phase 4 (optional hardening)

- Adaptive PoW difficulty under load.
- Reputation-based throttling.
- Multi-node ProcMan sharding.

---

## 13. Residual Risks and Operator Posture

After all phases, residual risk is mainly high-budget, continuously rotating distributed abuse.

**Recommended posture:**

- Reasonably safe for unattended operation **only if** auto-mitigation + alerts are active.
- If auto-mitigation is absent, periodic human supervision is still advised.

---

## 14. Canonical Decisions Snapshot

- Primary defense is lifecycle architecture (pooling/recycling), not PoW.
- ProcMan in-path proxy is additive and kept for full session.
- PoW is economic friction, not identity or trust.
- Per-IP cap remains at 2 by default.
- Capacity planning is RAM-first.
- Unattended operation requires behavioral auto-response.
