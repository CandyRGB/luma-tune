# LumaTune

<p align="center">
  <a href="README.zh.md">简体中文</a> · <b>English</b>
</p>

> A display-level picture tuner for Windows games. LumaTune improves shadow visibility and viewing comfort by writing a GPU gamma ramp — without injecting into any process, so it is invisible to anti-cheat.

LumaTune ships as a small desktop app built with [Slint](https://slint.dev/) and the [`windows`](https://crates.io/crates/windows) crate. Every adjustment is composited into a 256-entry `Gamma Ramp` and pushed to the display adapter.

- **No injection** — purely the display path, the same lookup applied to desktop, games, and screenshots alike.
- **Preset library** — browse preset cards, save favorites, and set your own cover images. Built-in presets can return to their original parameters.
- **Per-app presets** — bind a preset to an executable and apply it automatically while that app is in the foreground.
- **Self-healing** — if the app is killed, a dirty flag on disk restores the identity ramp on next launch, so the screen never keeps a leftover tint.
- **A/B compare** — use the editor’s compare button to switch between the preview and the original display state.

## How it works

A filter is a fixed-order **operator chain**. Users adjust operator parameters, not the curve directly. Each operator is a unary scalar function evaluated per channel, producing the final 256-entry lookup:

```
in black/white -> out black/white -> brightness -> contrast -> gamma -> per-channel gain -> curve -> clamp
```

All math runs in **encoded space** (no sRGB decode), because the downstream lookup has no color management at all. The curve at the end uses monotonic (PCHIP / Fritsch–Carlson) interpolation to avoid banding and reversal between control points.

| Parameter | Range | Neutral |
| --- | --- | --- |
| Input black / white | 0…0.5 / 0.5…1 | 0 / 1 |
| Brightness | −0.12…0.12 | 0 |
| Contrast | 0.2…3.0 | 1 |
| Shadows / Highlights | −0.3…0.3 | 0 |
| Gamma | 0.2…5.0 | 1 |
| Temperature / Tint | 1000K…10000K / −1…1 | 6500K / 0 |
| Channel gains | 0.5…1.5 | 1 |
| Output black / white | 0…0.15 / 0.7…1 | 0 / 1 |
| Curve | up to 8 control points | identity |

> [!NOTE]
> The display path cannot do channel-crossing math, so saturation, vibrance, sharpening, or color-blind simulation are out of scope. HDR is detected and the filter is disabled with a clear message instead of silently doing nothing.

## Explicit non-goals

Process injection, Present hooks, and DXGI proxies are rejected — the anti-cheat risk is unacceptable. Low-level keyboard hooks, `.cube` LUT import, exposure as a separate parameter, and any server-side account/cloud/community features are likewise not part of v1.

## Repository layout

```
apps/lumatune          Slint desktop app (UI + runtime wiring)
crates/lumatune-core   Platform-agnostic filter model: params, operators, curve, presets
crates/lumatune-win    Windows glue: display enumeration, ramp read/write, dirty state, settings
crates/gamma-probe     Standalone diagnostic CLI for verifying ramp support and latency
docs/                  Design notes, references, and ADRs
```

## Build & run

> [!IMPORTANT]
> LumaTune targets Windows only. It needs the stable Rust toolchain and the MSVC build tools (the `windows` crate links against the Windows SDK).

```sh
# Build the workspace
cargo build --release

# Run the desktop app
cargo run -p lumatune --release

# Verify gamma-ramp support and readback accuracy on this machine
cargo run -p gamma-probe --release -- report
```

`gamma-probe` also offers `visual` (on-screen curve check), `watch` (continuous write/verify loop), `focus` (foreground process detection), and `restore` (write back the identity ramp).

The desktop UI uses the published [`slint-winui` crate](https://crates.io/crates/slint-winui) for its navigation components.

## Tests

```sh
cargo test
```

The core model (parameters, curve interpolation, presets) and the Windows settings layer carry unit tests. Correctness of the end-to-end loop is validated by hand: launch a game, confirm the preset auto-applies, tweak live, exit and confirm the previous ramp returns, then kill the process and confirm self-healing on restart.

## Data & files

Settings, the dirty flag, and presets live under `%APPDATA%\LumaTune\`. The dirty flag (`active.flag`) records the device currently under a non-identity ramp; it is written before any non-identity ramp and removed only after the identity ramp is restored.

## Documentation

- Design rationale and acceptance criteria: `docs/design.md`
- Architecture decision records: `docs/adr/`
- Shared vocabulary: `GLOSSARY.md`
