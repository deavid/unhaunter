# 07 – What Gets Bundled: Runtime vs App vs OS

## The Layered Model

A running Flatpak app is composed of three layers:

```
┌─────────────────────────────────────┐
│             Your App                │  ← /app  (you control this)
│   binary + assets + wrapper script  │
├─────────────────────────────────────┤
│           Runtime                   │  ← /usr  (shared, downloaded once)
│  org.freedesktop.Platform //24.08   │
│  Mesa, glibc, X11/Wayland, PulseAudio, etc. │
├─────────────────────────────────────┤
│           Host OS kernel            │  ← provided by the user's Linux distro
│   kernel, DRM drivers, firmware     │
└─────────────────────────────────────┘
```

The middle layer (runtime) is the key: it is a curated set of OS-level libraries that are
**shared by all Flatpak apps** using that runtime. You do not ship them; users download them
once and all apps reuse them.

---

## What `org.freedesktop.Platform //24.08` Already Provides

Because the runtime includes these, you must **not** re-bundle them:

| Library | Notes |
|---------|-------|
| glibc | C runtime |
| libpthread, libdl, etc. | POSIX threading |
| libGL / Mesa (OpenGL + Vulkan) | GPU rendering stack |
| Vulkan loader + Mesa Vulkan drivers | For Bevy's Vulkan backend |
| libX11, libXext, libXi, libXrandr, libXcursor, libXinerama | X11 |
| libwayland-client, libwayland-egl | Wayland |
| xkbcommon | Keyboard handling |
| ALSA (libasound) | Low-level audio |
| PulseAudio client libraries | High-level audio |
| PipeWire PulseAudio bridge (on modern setups) | Modern audio stack |
| libz, libssl/openssl | Compression, TLS |
| fontconfig, freetype | Text rendering |
| libdbus | D-Bus IPC |

### ALSA Specifically

> "Some libraries are needed, but generally OS-only ones such as ALSA libs."

ALSA (`libasound.so`) is in `org.freedesktop.Platform`. You do **not** need to bundle it. The
`--socket=pulseaudio` finish-arg gives the app access to PulseAudio (and transitively to
PipeWire's PulseAudio compatibility mode). On systems where users only have ALSA (very rare,
mostly embedded), `--socket=alsa` can be added as a fallback.

Bevy uses `cpal` (via `bevy_audio`/`rodio`), which supports ALSA, PulseAudio, and PipeWire on
Linux. The runtime + `--socket=pulseaudio` permission covers the vast majority of users.

---

## What You DO Bundle (in `/app`)

Everything that is **not** in the runtime but is required to run:

| What | Why |
|------|-----|
| `unhaunter_game` binary | Your compiled Rust binary |
| `assets/` directory | Game data (maps, sprites, sounds, fonts) |
| Any Rust-statically-linked libraries | Bevy and all Cargo deps are statically linked into the binary. No extra `.so` files needed. |
| Wrapper shell script | Sets working directory for asset resolution |

### Static Linking is Your Friend

Rust compiles most dependencies statically into the final binary by default. This means
**all** of your Cargo dependency code (Bevy, rodio, image decoding libraries, etc.) ends up
inside `unhaunter_game` as a single self-contained ELF binary. The only dynamic libraries
loaded at runtime are the ones provided by the `org.freedesktop.Platform` runtime listed above.

You can verify this:

```bash
ldd target/x86_64-unknown-linux-gnu/release/unhaunter_game
# Expect to see: libGL, libasound, libwayland-*, libX11-*, libpthread, libc, libdl
# All of these are in org.freedesktop.Platform. Nothing exotic.
```

If `ldd` shows unexpected `.so` files that are not in the runtime, those need to be bundled or
eliminated.

---

## What the Host Kernel Provides (Never Bundled)

- Kernel syscalls (io_uring, epoll, futex, etc.)
- DRM/KMS GPU drivers (nvidia.ko, i915.ko, amdgpu.ko)
- USB, input device drivers

These are never part of a Flatpak. The sandbox transparently accesses the host kernel through
the bubblewrap layer. `--device=dri` gives the sandbox access to `/dev/dri/*` (GPU nodes).

---

## SDK Extensions (Build-Time Only)

`org.freedesktop.Sdk.Extension.rust-stable` provides the Rust compiler toolchain at build time.
It is **not** installed in the final app – it is only available during `flatpak-builder` runs.
The compiled binary is the only output that ends up in `/app/bin/`.

---

## Practical Size Expectations

For a Bevy game of Unhaunter's current scale:

| Component | Estimated size |
|-----------|---------------|
| Binary (`unhaunter_game`, stripped) | ~30–80 MB (Bevy compiles a lot) |
| Assets (maps, sprites, audio) | Depends on content; currently likely 10–50 MB |
| Runtime (`org.freedesktop.Platform`) | ~300 MB (shared with all other Flatpak apps) |
| **App download size** | ~40–130 MB compressed |

> The runtime is downloaded once and shared. Users who already have it (very likely) pay nothing
> for that component. Only the app layer is freshly downloaded on install or update.

You can reduce binary size with `strip` and link-time optimisation (LTO), which are standard
for release builds. Bevy defaults to reasonable settings in release mode.
