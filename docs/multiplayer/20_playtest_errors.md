# First playtest session in multiplayer across the internet

This was tested with RTT: ~60ms

## Symptomps

- Objects when picked up sometimes disappear from the client with tons of errors that entity was not found. Sometimes
  when joining an ongoing map we see duplicated gear on the ground.
  - Client if joined and there was gear on the ground before joining, if this gear is picked up (by Host or Client) does
    not remove the sprites somehow. The gear is transerred to the right place, the Host is updated correctly, but the
    client still sees gear in the ground.

MIGHT BE FIXED - aditional testing required:

- Host slows down a lot
  - Fixed - probably by adding TCP_NODELAY and a few optimizations.
  - Can't reproduce. TCP_NODELAY might have been the issue. Needs re-testing.

FIXED - needs more verification:

- Trying to activate gear from the client, pressing [R] or Right click was painful as it was not reponsive, or it was
  bouncy (activates then deactivates)
  - FIXED: Made it client-local
- Impossible to control whether listening should happen in IPv4 or IPv6. We need to listen on BOTH at the same time. Do
  we have a flag to provide a list of source IP addresses for the listening part?
  - Fixed, socket listening reworked.
  - Now we see an error of address already in use, after opening IPv6 successfully and attempting IPv4.
    - This needs additional checking. It is possible that the host has some "if open in IPv4 by default open IPv6" or
      vice-versa.
- Errors for despawining on the client: See Appendix A
  - Fixed: These seem gone after the fixes.
- On the client, When hiding:
  - The "eye icon" does not seem to disappear after stop hiding.
  - The client seems to be able to move even when hidden.
- When the other player is hiding, they should be way more transparent to make it clear.

## Dedicated Server

We should put as a first priority making a dedicated server that supports at least 1-4 players. With this testing would
be much easier since we could directly deploy in a VPS and test as a client.
