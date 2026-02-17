# Critical User Journeys (CUJ) — Multiplayer Hub

This document defines the critical paths for users interacting with the Unhaunter Hub ecosystem. These journeys serve as
the acceptance criteria for the user experience.

## 1. The "Friday Night Host" (Creating a Room)

**Persona:** Alex, a regular player who wants to host a game for 3 friends. **Goal:** Create a private room and get
everyone in with minimal friction.

1. **Launch:** Alex launches the game and clicks "Play Online".
2. **Create:** Alex clicks "Create Room".
   - _System:_ Connects to Hub, requests room allocation.
   - _UI:_ Shows a spinner/loading state ("Creating room...").
3. **Lobby:** Alex lands in the Lobby screen.
   - _UI:_ A large, readable Room Code (e.g., `K7WRP`) is displayed prominently at the top right.
   - _UI:_ The code is easy to read (Safe-Vocal font/colors).
4. **Share:** Alex says "Code is K7WRP" over Discord voice.
5. **Wait:** As friends join, their names/colors appear in the player list instantly.
6. **Play:** Once everyone is in, Alex selects a map and clicks "Start Mission".

**Success Criteria:**

- Total time from "Create Room" to Lobby < 5 seconds.
- Room code is visible without searching.
- No IP addresses or port forwarding required.

## 2. The "Late Joiner" (Joining a Room)

**Persona:** Sam, joining Alex's game. **Goal:** Enter the code and be in the lobby.

1. **Launch:** Sam launches the game and clicks "Play Online".
2. **Join:** Sam clicks "Join Room".
3. **Input:** A code entry screen appears.
   - _UI:_ Large text input.
   - _Interaction:_ Sam types `k7wrp` (case insensitive).
   - _Feedback:_ Input formats automatically (e.g., uppercase).
4. **Connect:** Sam presses Enter.
   - _System:_ Queries Hub for `K7WRP`, gets IP, connects to dedicated server.
   - _UI:_ "Connecting..." -> "Joining Lobby..."
5. **Lobby:** Sam appears in Alex's lobby.
   - _UI:_ Sees the same map selection and player list as Alex.

**Success Criteria:**

- Typing the code feels robust (no ambiguous characters).
- Connection errors (typo, full room) give clear, human-readable feedback ("Room not found", "Room full").

## 3. The "Community Sysadmin" (Hosting a Hub)

**Persona:** Jordan, a community leader who wants to run a private Hub for their Discord server. **Goal:** Deploy the
infrastructure on a Linux VPS.

1. **Setup:** Jordan clones the repo or downloads binaries.
2. **Config:** Jordan edits `hub_config.ron` and `procman_config.ron`.
   - _Config:_ Sets `port_range`, `public_addr`, and `allowed_procman_uuids`.
3. **Run:** Jordan starts `unhub` and `unprocman` (e.g., via systemd).
   - _Logs:_ Clear startup messages indicating listening ports and successful connection between ProcMan and Hub.
4. **Verify:** Jordan checks `GET /health` on the Hub.
   - _Response:_ JSON with version and uptime.
5. **Distribute:** Jordan tells community members to launch with `--universe https://hub.jordan-community.com`.

**Success Criteria:**

- No compilation required (binaries provided).
- Configuration is self-documenting or documented.
- Logs clearly show if something is wrong (e.g., port conflict, auth failure).

## 4. The "Solo Explorer" (Testing Connectivity)

**Persona:** Casey, a solo player testing if their internet works with the Hub. **Goal:** Verify connection without
needing a second person.

1. **Launch:** Casey launches the game.
2. **Check:** Casey looks at the "Play Online" button.
   - _UI:_ A small indicator (green dot or text) shows "Hub Connected".
3. **Create:** Casey creates a room just to see if it works.
   - _Result:_ Successfully enters a lobby alone.
4. **Exit:** Casey leaves the lobby.
   - _System:_ Room is cleaned up after timeout.

**Success Criteria:**

- Hub status is visible before attempting actions.
- Creating a room alone is a valid operation.

## 5. The "Reconnection" (Resilience)

**Persona:** Alex (Host) and Sam (Client). **Goal:** Recover from a temporary internet blip.

1. **Scenario:** Alex's internet drops for 10 seconds during a mission.
2. **Disconnect:** Sam sees "Connection Lost".
3. **Reconnect:** Alex's internet comes back.
   - _System:_ Game client attempts auto-reconnect to the dedicated server.
   - _Result:_ If within timeout, session resumes. If not, returned to Main Menu with clear message.

**Success Criteria:**

- Disconnects are handled gracefully (no crash).
- Error messages explain _why_ (Timeout vs Kick vs Hub Error).

---

## Reviewer Checklist: UX Pitfalls & Broken Flows

This section lists common UX gaps, missing affordances, and broken flows that reviewers should actively look for when
validating the CUJs above. These are the things most likely to slip through developer self-testing.

### Room Code Visibility & Usability

- [ ] **"Where's my room code?"** — Is the code prominently displayed in the lobby, or buried among other UI elements? A
      host who can't find the code within 2 seconds will ask "wait, what's the code?" on voice chat.
- [ ] **Code readability** — Are ambiguous characters avoided (0/O, 1/I/l, 5/S)? Is the font large enough to read at a
      glance? Is there sufficient contrast against the background?
- [ ] **Code copyability** — Can the code be copied to clipboard with a single click or button? Players sharing over
      text chat (Discord, Steam) will need this. Is there a "Copied!" confirmation?
- [ ] **Code persistence** — Does the room code remain visible throughout the entire lobby session, or does it disappear
      after a transition, resize, or state change?

### Navigation & Escape Hatches

- [ ] **Back button at every step** — Can the user go back from every screen? Specifically: from "Join Room" input, from
      "Connecting..." spinner, from the lobby itself. If any screen is a dead-end without a back/cancel option, that's a
      blocker.
- [ ] **Escape key behavior** — Does pressing Escape do something sensible at every point? Does it cancel a pending
      connection? Does it open a "Leave lobby?" confirmation? Or does it do nothing?
- [ ] **What happens after leaving a lobby?** — Does the user land back at the "Play Online" screen, the main menu, or
      somewhere unexpected? Is the flow consistent for host and joiner?
- [ ] **"Play Online" when hub is unreachable** — Is the button grayed out, or does the user click it and get an error 3
      seconds later? The error should be immediate and clear, not a timeout.

### Loading & Connection States

- [ ] **No feedback during connection** — Is there a visible spinner or status text while connecting? A blank screen or
      frozen UI during a 2-3 second connection feels broken.
- [ ] **Stuck spinner** — If the hub is unreachable or the dedicated server never starts, does the spinner spin forever?
      There must be a timeout that leads to a clear error message and a way back.
- [ ] **Double-click protection** — Can "Create Room" or "Join Room" be clicked multiple times rapidly? Does this create
      duplicate rooms or duplicate connection attempts?
- [ ] **Progress indication on join** — Does the joiner see distinct phases ("Resolving room..." → "Connecting to
      server..." → "Joining lobby...") or just a generic "Loading..."? Distinct states help users diagnose where things
      fail.

### Error Messages & Unhappy Paths

- [ ] **Wrong room code** — Is the error "Room not found" or is it a raw HTTP status / internal error string?
- [ ] **Room full** — Is there a specific "Room is full" message, or does it silently fail / show a generic error?
- [ ] **Hub down** — Does the game say "Cannot reach server" or does it show a Rust panic, a connection refused error,
      or nothing at all?
- [ ] **Dedicated server fails to start** — If ProcMan can't allocate a server (port exhaustion, binary missing), does
      the host get a clear error, or does "Create Room" just hang?
- [ ] **Error messages actionable** — Do errors tell the user what to _do_ ("Check the room code and try again") or just
      state what happened ("Error 404")?

### Lobby State & Player List

- [ ] **Player list updates in real time** — When a player joins, does their name appear immediately or only after a
      manual refresh / screen transition?
- [ ] **Player who left still showing** — If a player disconnects or leaves, does their entry disappear from the lobby
      list, or does it linger as a ghost entry?
- [ ] **Host indicator** — Is it clear who the host is? Can the host tell they are the host? Can joiners tell who the
      host is?
- [ ] **Player names/colors distinguishable** — Are default names/colors distinct enough, or do two players show up as
      "Player" with the same color?
- [ ] **Empty lobby feels intentional** — A host alone in a lobby should see something like "Waiting for players..."
      rather than just an empty list that looks broken.

### Start Mission Flow

- [ ] **Only host can start** — Is the "Start Mission" button hidden or disabled for non-host players? Or can anyone
      click it?
- [ ] **Start with no map selected** — What happens if the host clicks "Start Mission" without selecting a map? Error?
      Silent no-op? Default map?
- [ ] **Map selection visible to joiners** — Can joiners see which map the host has selected, or is it a surprise?
- [ ] **Start during player mid-join** — If a player is in "Connecting..." state when the host starts, do they get
      stranded in the lobby while the mission begins without them?

### Sysadmin Experience

- [ ] **Missing or invalid config file** — Does the hub/procman crash with a stack trace, or does it print a
      human-readable error pointing at the bad field?
- [ ] **Config file documentation** — Are all fields in `hub_config.ron` and `procman_config.ron` commented or
      self-explanatory? Or are there opaque field names with no explanation?
- [ ] **Startup log clarity** — On successful start, do the logs confirm: listening address, port, connected procman
      instances, version? Or is it silent until something happens?
- [ ] **Health endpoint completeness** — Does `/health` return useful info (version, uptime, active rooms, connected
      procmans) or just `{"status":"ok"}`?

### Cleanup & Resource Leaks

- [ ] **Room cleanup after host quits** — If the host closes the game (gracefully or via Alt+F4), does the room get
      cleaned up on the hub? Or does it appear in room listings indefinitely?
- [ ] **Dedicated server process cleanup** — When a room ends, does ProcMan actually kill the dedicated server process?
      Or do zombie processes accumulate?
- [ ] **Port range exhaustion feedback** — If all ports in the configured range are in use, does "Create Room" give a
      clear "Server capacity full" error?

### Second-Attempt & Re-entry Flows

- [ ] **Join after failed join** — After getting "Room not found", can the user immediately type a new code and try
      again, or is the UI stuck in an error state requiring navigation away and back?
- [ ] **Create after failed create** — If room creation fails, can the user retry from the same screen?
- [ ] **Rejoin after disconnect** — If disconnected from a lobby, can the user re-enter the same room code and rejoin,
      or is the room in a broken state with their old session lingering?
- [ ] **Full game restart** — Close the game entirely, reopen, and repeat all CUJs. Does everything work the same as the
      first session, or does stale state (cached UUIDs, lingering connections) cause issues?
