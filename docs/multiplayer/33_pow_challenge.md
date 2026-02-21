# Step 3: Proof-of-Work (PoW) Challenge (Economic Friction)

**Goal:** Make room creation computationally expensive for attackers (~100ms) while remaining imperceptible to
legitimate players.

## 1. Update Protocol Definitions

**File:** `crates/unhub-client/src/protocol.rs`

1. Add `ChallengeRequest` and `ChallengeResponse`:

   ```rust
   #[derive(Serialize, Deserialize, Debug, Clone)]
   pub struct ChallengeRequest {
       pub player_uuid: Uuid,
   }

   #[derive(Serialize, Deserialize, Debug, Clone)]
   pub struct ChallengeResponse {
       pub nonce: String,
       pub difficulty: u32,
   }
   ```

2. Update `CreateRoomRequest` to include the PoW solution:
   ```rust
   #[derive(Serialize, Deserialize, Debug, Clone)]
   pub struct CreateRoomRequest {
       pub player_uuid: Uuid,
       pub game_version: String,
       pub nonce: String,
       pub solution: String,
   }
   ```

## 2. Add PoW Solver to Client Library

**File:** `crates/unhub-client/src/lib.rs`

1. Add `sha2` to `crates/unhub-client/Cargo.toml`:
   ```toml
   [dependencies]
   sha2 = "0.10"
   ```
2. Add the `solve_pow` function:

   ```rust
   use sha2::{Digest, Sha256};

   pub fn solve_pow(nonce: &str, difficulty: u32) -> String {
       let mut i = 0u64;
       loop {
           let candidate = format!("{}:{}", nonce, i);
           let hash = Sha256::digest(candidate.as_bytes());

           // Check leading zero bits
           let mut zero_bits = 0;
           for byte in hash {
               if byte == 0 {
                   zero_bits += 8;
               } else {
                   zero_bits += byte.leading_zeros();
                   break;
               }
           }

           if zero_bits >= difficulty {
               return i.to_string();
           }
           i += 1;
       }
   }
   ```

## 3. Update Hub State

**File:** `crates/tools/unhub/src/state.rs`

1. Add `nonces` to `HubState`:

   ```rust
   pub struct NonceEntry {
       pub player_uuid: Uuid,
       pub client_ip: std::net::IpAddr,
       pub issued_at: std::time::Instant,
   }

   pub struct HubState {
       // ... existing fields ...
       pub nonces: Arc<DashMap<String, NonceEntry>>,
   }
   ```

2. Initialize it in `HubState::new`:
   ```rust
   impl HubState {
       pub fn new(config: HubConfig) -> Self {
           Self {
               // ... existing fields ...
               nonces: Arc::new(DashMap::new()),
           }
       }
   }
   ```
3. Add `pow_difficulty` to `HubConfig`:
   ```rust
   pub struct HubConfig {
       // ... existing fields ...
       pub pow_difficulty: u32,
   }
   ```

## 4. Add `/v1/challenge` Endpoint

**File:** `crates/tools/unhub/src/api.rs`

1. Add the `challenge` handler:

   ```rust
   pub async fn challenge(
       State(state): State<HubState>,
       axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
       headers: axum::http::HeaderMap,
       Json(payload): Json<ChallengeRequest>,
   ) -> Result<Json<ChallengeResponse>, (StatusCode, Json<HubError>)> {
       let config = state.config.read().await;

       // Check ban list
       if config.banned_uuids.contains(&payload.player_uuid) {
           return Err((StatusCode::FORBIDDEN, Json(HubError { error: "banned".to_string(), message: "Banned".to_string() })));
       }

       // Extract IP (same logic as create_room)
       let client_ip = addr.ip(); // Add proxy logic if needed

       // Enforce max 2 outstanding nonces per UUID and IP
       let mut uuid_count = 0;
       let mut ip_count = 0;
       for entry in state.nonces.iter() {
           if entry.player_uuid == payload.player_uuid { uuid_count += 1; }
           if entry.client_ip == client_ip { ip_count += 1; }
       }

       if uuid_count >= 2 || ip_count >= 2 {
           return Err((StatusCode::TOO_MANY_REQUESTS, Json(HubError { error: "rate_limited".to_string(), message: "Too many challenges".to_string() })));
       }

       let nonce = uuid::Uuid::new_v4().to_string();
       state.nonces.insert(nonce.clone(), NonceEntry {
           player_uuid: payload.player_uuid,
           client_ip,
           issued_at: std::time::Instant::now(),
       });

       Ok(Json(ChallengeResponse {
           nonce,
           difficulty: config.pow_difficulty,
       }))
   }
   ```

2. Update `create_room` to verify the PoW:

   ```rust
   // Inside create_room, before checking IP caps:
   let entry = state.nonces.remove(&payload.nonce).map(|(_, v)| v).ok_or((
       StatusCode::BAD_REQUEST,
       Json(HubError { error: "invalid_pow".to_string(), message: "Invalid or expired nonce".to_string() }),
   ))?;

   if entry.player_uuid != payload.player_uuid || entry.client_ip != client_ip {
       return Err((StatusCode::BAD_REQUEST, Json(HubError { error: "invalid_pow".to_string(), message: "Nonce mismatch".to_string() })));
   }

   if entry.issued_at.elapsed().as_secs() > 120 {
       return Err((StatusCode::BAD_REQUEST, Json(HubError { error: "invalid_pow".to_string(), message: "Nonce expired".to_string() })));
   }

   let candidate = format!("{}:{}", payload.nonce, payload.solution);
   let hash = sha2::Sha256::digest(candidate.as_bytes());
   let mut zero_bits = 0;
   for byte in hash {
       if byte == 0 { zero_bits += 8; } else { zero_bits += byte.leading_zeros(); break; }
   }

   if zero_bits < config.pow_difficulty {
       return Err((StatusCode::BAD_REQUEST, Json(HubError { error: "invalid_pow".to_string(), message: "Incorrect solution".to_string() })));
   }
   ```

## 5. Update Game Client

**File:** `crates/unhub-plugin/src/hub_client.rs`

1. Update the `HubRequest::CreateRoom` handler to first call `/v1/challenge`, solve the PoW, and then call
   `/v1/rooms/create` with the solution.

   ```rust
   // Inside the worker thread loop:
   HubRequest::CreateRoom { player_uuid, game_version } => {
       // 1. Request Challenge
       let challenge_res = client.post(format!("{}/v1/challenge", worker_hub_url))
           .json(&unhub_client::protocol::ChallengeRequest { player_uuid })
           .send()
           .await;

       if let Ok(resp) = challenge_res {
           if resp.status().is_success() {
               if let Ok(challenge) = resp.json::<unhub_client::protocol::ChallengeResponse>().await {
                   // 2. Solve PoW
                   let solution = unhub_client::solve_pow(&challenge.nonce, challenge.difficulty);

                   // 3. Create Room
                   let create_res = client.post(format!("{}/v1/rooms/create", worker_hub_url))
                       .json(&CreateRoomRequest { player_uuid, game_version, nonce: challenge.nonce, solution })
                       .send()
                       .await;

                   // ... handle create_res as before ...
               }
           }
       }
   }
   ```

## 6. Verification

- Start the Hub and ProcMan.
- Attempt to create a room from the game client. It should take ~100ms longer but succeed.
- Attempt to create a room using `curl` without a valid nonce/solution. It should fail with `400 Bad Request`.
