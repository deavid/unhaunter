# Step 2: Per-IP Room Caps (Blast Radius Limiting)

**Goal:** Limit the number of active rooms a single IP can hold to 2. This ensures that even if an attacker bypasses
other limits, they can only hold 2 server processes hostage at a time.

## 1. Update `HubState`

**File:** `crates/tools/unhub/src/state.rs`

1. Add `rooms_by_ip` and `room_to_ip` to `HubState`:
   ```rust
   pub struct HubState {
       // ... existing fields ...
       pub rooms_by_ip: Arc<DashMap<std::net::IpAddr, Vec<String>>>,
       pub room_to_ip: Arc<DashMap<String, std::net::IpAddr>>,
   }
   ```
2. Initialize them in `HubState::new`:
   ```rust
   impl HubState {
       pub fn new(config: HubConfig) -> Self {
           Self {
               // ... existing fields ...
               rooms_by_ip: Arc::new(DashMap::new()),
               room_to_ip: Arc::new(DashMap::new()),
           }
       }
   }
   ```

## 2. Update `HubConfig`

**File:** `crates/tools/unhub/src/state.rs`

1. Add `max_rooms_per_ip` and `trust_proxy_headers` to `HubConfig`:
   ```rust
   pub struct HubConfig {
       // ... existing fields ...
       pub max_rooms_per_ip: usize,
       pub trust_proxy_headers: bool,
   }
   ```
2. Ensure your config loader (e.g., `crates/tools/unhub/src/config.rs`) provides defaults for these (e.g.,
   `max_rooms_per_ip: 2`, `trust_proxy_headers: false`).

## 3. Update `create_room` API

**File:** `crates/tools/unhub/src/api.rs`

1. Update the `create_room` signature to extract the client's IP address. You will need `axum::extract::ConnectInfo` and
   `axum::http::HeaderMap`:
   ```rust
   pub async fn create_room(
       State(state): State<HubState>,
       axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
       headers: axum::http::HeaderMap,
       Json(payload): Json<CreateRoomRequest>,
   ) -> Result<Json<CreateRoomResponse>, (StatusCode, Json<HubError>)> {
   ```
2. Determine the client IP:
   ```rust
   let config = state.config.read().await;
   let client_ip = if config.trust_proxy_headers {
       headers
           .get("X-Forwarded-For")
           .and_then(|h| h.to_str().ok())
           .and_then(|s| s.split(',').next())
           .and_then(|s| s.trim().parse::<std::net::IpAddr>().ok())
           .unwrap_or(addr.ip())
   } else {
       addr.ip()
   };
   ```
3. Check the IP cap before allocating a room:
   ```rust
   if let Some(rooms) = state.rooms_by_ip.get(&client_ip) {
       if rooms.len() >= config.max_rooms_per_ip {
           return Err((
               StatusCode::TOO_MANY_REQUESTS,
               Json(HubError {
                   error: "room_cap_exceeded".to_string(),
                   message: "You have reached the maximum number of active rooms.".to_string(),
               }),
           ));
       }
   }
   ```
4. After the room is successfully created (when `state.rooms.get(&room_code)` succeeds), add it to the tracking maps:
   ```rust
   state.rooms_by_ip.entry(client_ip).or_default().push(room_code.clone());
   state.room_to_ip.insert(room_code.clone(), client_ip);
   ```

## 4. Update `ProcManMessage::RoomClosed` Handler

**File:** `crates/tools/unhub/src/procman.rs`

1. Locate the `handle_procman_connection` loop where `ProcManMessage::RoomClosed` is handled.
2. When a room is closed, remove it from the tracking maps:
   ```rust
   ProcManMessage::RoomClosed { room_code, .. } => {
       state.rooms.remove(&room_code);
       if let Some((_, ip)) = state.room_to_ip.remove(&room_code) {
           if let Some(mut rooms) = state.rooms_by_ip.get_mut(&ip) {
               rooms.retain(|c| c != &room_code);
               if rooms.is_empty() {
                   drop(rooms); // Release the lock before removing
                   state.rooms_by_ip.remove(&ip);
               }
           }
       }
   }
   ```

## 5. Verification

- Start the Hub and ProcMan.
- Create 2 rooms from the same IP.
- Attempt to create a 3rd room. It should fail with `429 Too Many Requests`.
- Wait 10 seconds for the rooms to expire (fast expiry).
- Attempt to create a room again. It should succeed.
