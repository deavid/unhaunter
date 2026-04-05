# Text Chat System Design (2026-02-22)

**Status:** Approved Design

**Goal:** Provide an accessible text-based communication system that serves as a reliable fallback for players without
microphones, while maintaining the game's tension through environmental interference.

## 1. Core Philosophy: The Physical Transmission

Text chat in _Unhaunter_ is primarily an accessibility feature, but it is grounded in the physical reality of the game
world. It is treated as a low-baud radio data transmission (like a rugged PDA or teleprinter). This ensures that players
who cannot use microphones can still participate, but their communication is subject to the same terrifying physical
limitations (EMI, distance) as the rest of the game.

## 2. Implementation Rules

The text chat system follows these core rules:

1. **Universal Delivery (No Equipment Required):** Text chat always works. It does not require the sender or receiver to
   hold a Walkie-Talkie.
2. **Standard HUD Display:** Messages appear in a standard, unobtrusive chat box in the corner of the player's HUD.
3. **Input Isolation (No Double Action):** When the chat input UI is active, it must exclusively capture keyboard input.
   Typing "W" to say "Where are you?" must not cause the player character to walk forward.
4. **The Baud Rate (Sequential Display):** Messages do not appear instantly. They are "printed" to the receiver's screen
   character-by-character at a fixed rate (e.g., 5 characters per second). This turns reading a message into a
   suspenseful event.
5. **Ghost Interference (The Garble):** The transmission is susceptible to Electro Magnetic Interference (EMI).
   - The client computes the garble locally based on the sender's EMI, the receiver's EMI, and the EMI along the
     physical line-of-sight path between them.
   - As the message prints character-by-character, high EMI causes characters to be replaced with symbols (`#`, `%`,
     `&`, `*`).
   - **Garble Decay:** The garbled characters shift and change rapidly as they are printed. If the player pays close
     attention, they might deduce the real letter as it flashes briefly. However, after a few seconds, the text
     "settles" into a final, static garbled state. If you look away and look back, it will be permanently corrupted.

## 3. Technical Implementation

### 3.1 Networking (`unnet-core` / `unnet-plugin`)

A new `NetworkMessage` variant is required. The server acts as a dumb relay, trusting the clients to compute their own
garble.

```rust
pub enum NetworkMessage {
    // ... existing variants ...
    /// A text message sent from one player to the lobby.
    ChatMessage {
        sender_id: NetworkId,
        content: String,
        /// The exact position of the sender at the time of transmission
        sender_position: [f32; 3],
    },
}
```

### 3.2 Limits and Throttling (Anti-Spam)

To prevent abuse and maintain pacing, strict limits are enforced:

- **Length Limits:**
  - Client Send Limit: 2000 characters.
  - Server Relay / Client Receive Limit: 4000 characters (leaves room for future protocol metadata).
- **Rate Limits (Token Bucket):**
  - **Message Rate:** Max 0.5 messages per second (1 message every 2 seconds).
  - **Character Rate:** Max 3 characters per second.
  - **UI Behavior:** The player can type freely, but when they hit "Send", the UI queues the message. They can move and
    play normally, but they cannot queue a second message until the first one has fully "transmitted" based on the 3
    chars/sec budget.

### 3.3 UI (`unui-plugin` / `unui-core`)

- **Chat Box HUD:** A simple, scrolling text box UI element that supports the character-by-character "printing"
  animation and the shifting garble effect.
- **Input State:** The game needs a UI state (e.g., `AppState::TypingChat`) that intercepts keyboard events before they
  reach the player movement systems.

## 4. Why This Matters (The "Mic Failure" Scenario)

As noted in recent design discussions, "it sucks when people are trying to make signs or say 'do you have microphone?'
and you don't know how to reply."

This text chat system solves this by:

1. **Explicit Verification:** Players can quickly type "Mic check" or "I can hear you, but can't talk."
2. **Atmospheric Tension:** Receiving a garbled "HE*P M*" in the chat box while you're in the safety of the truck is far
   scarier than a clear voice line. Watching it print slowly, character by character, while the garble shifts, forces
   the player to pay attention to the UI in a moment of panic.

## 5. Next Steps

- **Phase 1:** Add `ChatMessage` to `NetworkMessage` with spatial data and implement basic host-relay broadcasting with
  length/rate limits.
- **Phase 2:** Implement the Chat Box HUD, the input-blocking logic, and the 1-message transmission queue.
- **Phase 3:** Implement the Client-side "Baud Rate" printing animation.
- **Phase 4:** Integrate `InterferenceReceiver` and raycasting to compute the shifting "Garble Decay" effect.
