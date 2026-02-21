# Step 1: Bounded Codecs (OOM Prevention)

**Goal:** Prevent a single attacker from causing an Out-Of-Memory (OOM) crash by sending an infinite string without a
newline over the TCP connection.

## 1. Update `unhub` ProcMan Listener

**File:** `crates/tools/unhub/src/procman.rs`

1. Locate the `handle_procman_connection` function.
2. Find the line where the `Framed` codec is initialized:
   ```rust
   let mut framed = Framed::new(stream, LinesCodec::new());
   ```
3. Replace it with a bounded codec. A safe limit for our JSON messages is 64KB (65536 bytes):
   ```rust
   let mut framed = Framed::new(stream, LinesCodec::new_with_max_length(65536));
   ```

## 2. Update `unprocman` Hub Communicator

**File:** `crates/tools/unprocman/src/hub_comm.rs`

1. Locate the `handle_hub_connection` function.
2. Find the line where the `Framed` codec is initialized:
   ```rust
   let mut framed = Framed::new(stream, LinesCodec::new());
   ```
3. Replace it with the same bounded codec:
   ```rust
   let mut framed = Framed::new(stream, LinesCodec::new_with_max_length(65536));
   ```

## 3. Verification

- Compile both `unhub` and `unprocman` to ensure there are no syntax errors.
- Run both and verify they can still connect and exchange the `ProcManHello` handshake successfully.
