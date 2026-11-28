# LumaTune

<p align="center">
  <a href="README.zh.md">简体中文</a> · <b>English</b>
</p>

> A picture tuner for Windows games with display adjustments and post-processing for borderless windows. LumaTune runs outside the game process.

LumaTune ships as a small desktop app built with [Slint](https://slint.dev/) and the [`windows`](https://crates.io/crates/windows) crate. Lighting and color adjustments use a 256-entry gamma ramp applied to the display.

- **Post-processing** — adjust HDR Toning, Bloom, Clarity, Saturation, and Vibrance in a bound app while the game continues receiving mouse and keyboard input.
- **Preset library** — browse preset cards, save favorites, and set your own cover images. Built-in presets can return to their original parameters.
- **Multiple displays** — apply or clear a desktop preset on each display from Home. LumaTune recalls previously applied settings on launch.
- **Per-app presets** — bind a preset to an executable and apply it automatically while that app is in the foreground.
- **Crash recovery** — on launch, restore the original ramps saved for the affected displays before loading their desktop presets.
- **Live A/B preview** — select a display or a window in the preset editor, then drag the divider to compare the captured frame with the draft preset applied.

## Post-processing

Post-processing requires Windows 10 version 2004 or later, an SDR display, and borderless windowed mode. Adjust any effect in the preset editor, save, and bind that preset to the game. A nonzero effect takes effect when the bound game comes to the foreground. HDR Toning adjusts local contrast and Bloom softens highlights. Clarity (−1 to 1) softens or strengthens local luminance contrast, especially in midtones. Saturation ranges from −1 (grayscale) through 0 (original saturation) to 1 (stronger colors). Vibrance (0 to 1) gives muted colors a larger boost than already vivid colors. When all effects are zero, the original game window stays visible. Windows Graphics Capture reads the bound window and Direct3D 11 draws the result in a click-through overlay. The overlay closes when the game loses focus.

## Display adjustments

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
> Saturation uses the post-processing path. The display gamma ramp adjusts channels independently. LumaTune shows a notice when HDR is active.

## Explicit non-goals

Process injection, Present hooks, and DXGI proxies are rejected — the anti-cheat risk is unacceptable. Low-level keyboard hooks, `.cube` LUT import, exposure as a separate parameter, and any server-side account/cloud/community features are likewise not part of v1.

## Repository layout

```
apps/lumatune          Slint desktop app (UI + runtime wiring)
crates/lumatune-core   Platform-agnostic filter model: params, operators, curve, presets
crates/lumatune-win    Windows integration: display tuning, window capture, post-processing, and storage
crates/gamma-probe     Standalone diagnostic CLI for verifying ramp support and latency
docs/                  Design notes, references, and ADRs
```

## Build & run

> [!IMPORTANT]
> LumaTune targets Windows only. Building requires stable Rust, the MSVC C++ build tools, and a Windows SDK with C++/WinRT headers.

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

Closing the main window keeps display adjustments running in the Windows notification area. Left-click the LumaTune tray icon to reopen the window. Right-click it and choose **退出 LumaTune** to quit and restore the display settings.

### Windows installer

Build the x64 application with the C/C++ runtime linked statically, then package it with NSIS 3.

```powershell
cargo --config 'target.x86_64-pc-windows-msvc.rustflags=["-C","target-feature=+crt-static"]' build --locked --release -p lumatune --target x86_64-pc-windows-msvc
makensis /INPUTCHARSET UTF8 installer/LumaTune.nsi
```

The installer is written to `target/LumaTune-<version>-Setup-x64.exe`. On first install, the folder defaults to `C:\Program Files\LumaTune` and can be changed. Updates reuse the registered installation folder. The installer compares the Windows file versions of the installed and bundled executables. It skips installation when the same or a newer version is already installed. Quit LumaTune from the tray before updating or uninstalling.

## Tests

```sh
cargo test
```

The core model (parameters, curve interpolation, presets) and the Windows settings layer carry unit tests. Correctness of the end-to-end loop is validated by hand: launch a game, confirm the preset auto-applies, tweak live, exit and confirm the previous ramp returns, then kill the process and confirm self-healing on restart.

## Data & files

Settings and presets live under `%APPDATA%\LumaTune\`. Before applying an adjustment, LumaTune records the original ramp for that display in `display-recovery.json`. It clears the entry after restoring the ramp.

## Documentation

- Design rationale and acceptance criteria: `docs/design.md`
- Architecture decision records: `docs/adr/`
- Shared vocabulary: `GLOSSARY.md`
