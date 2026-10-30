<p align="center"><img src="assets/logo/gearwise.svg" width="96" alt="Gearwise logo"></p>

# Gearwise

[![ARM64 builds](https://github.com/Dhiva-Labs/gearwise/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Dhiva-Labs/gearwise/actions/workflows/ci.yml)
![Linux x64 | arm64](https://img.shields.io/badge/Linux-x64%20%7C%20arm64-informational?logo=linux&logoColor=white)
![Windows x64 | arm64](https://img.shields.io/badge/Windows-x64%20%7C%20arm64-informational?logo=windows&logoColor=white)

One lightweight app to configure gaming mice and keyboards from **Logitech,
Razer, Corsair and SteelSeries** on **Linux and Windows** (x64 and Arm64) — DPI stages,
report rate, RGB lighting (zones and per-key), button assignments, onboard
profiles, macros and per-application profiles. An open alternative to
G HUB, Synapse, iCUE and GG that ships as a single small native binary.

![Button assignments on a real G502 HERO, read from its onboard memory](app/screenshots/00-real-g502-hero.png)

| | |
|---|---|
| ![DPI stages](app/screenshots/03-sensitivity.png) | ![Mouse lighting](app/screenshots/02-g502-hero-lighting.png) |
| ![Per-key RGB](app/screenshots/06-lighting-per-key.png) | ![Zone lighting](app/screenshots/04-k55-zone-lighting.png) |
| ![MMO side grid](app/screenshots/18-naga-button-grid.png) | ![Macros](app/screenshots/11-macros.png) |

## Highlights

- **Native and light** — one Rust binary (egui + glow), no Electron, no
  background services, no account. Idle CPU is ~0; it only redraws on
  input, device events or while an animated lighting preview is on screen.
- **Every control in one place**
  - *Sensitivity*: up to 5 DPI stages, active stage, X/Y split where the
    sensor allows it, snapped to the sensor's real step.
  - *Lighting*: per-zone effects (static, breathing, colour cycle, wave,
    reactive — whatever the device supports), colour, speed, brightness;
    per-key painting for RGB keyboards.
  - *Assignments*: any button to a mouse button, keystroke with modifiers,
    media key, DPI up/down/cycle/shift, profile cycle, disable, or a
    software macro.
  - *Performance*: report rate up to 8 kHz, lift-off distance, angle
    snapping, keyboard game mode.
  - *Onboard profiles*: switch and save device-stored profiles; settings
    survive moving the mouse to another PC.
- **Macros** — recorded or hand-edited (keys, mouse buttons, delays, text),
  play once / while held / toggle; played back through `uinput` on Linux and
  `SendInput` on Windows.
- **Per-app profiles** — switch automatically when a game gains focus
  (Windows, and X11/XWayland windows on Linux).
- **Profiles follow you** — every change is recorded in the active
  profile and re-applied when a device connects, sending only what
  differs (so onboard memory isn't rewritten on every launch). Colours on
  software-lit keyboards stay while the app runs.
- **Demo devices** — explore everything without hardware (Settings → Show
  demo devices).

## Supported hardware

| Brand | Protocol | Coverage |
|---|---|---|
| Logitech | HID++ 2.0 (wired, Lightspeed / Unifying / Bolt receivers, Bluetooth) | Discovered from the device's feature table, so G-series and MX devices work without a model list: DPI, report rate incl. 2/4/8 kHz, LED zones, per-key RGB, onboard profiles with button bindings, G-keys and diverted buttons for software actions, battery |
| Razer | Chroma 90-byte feature reports | 34 models — DeathAdder, Viper, Basilisk, Naga, Cobra, BlackWidow, Huntsman, Ornata: DPI and stages, polling incl. HyperPolling, zone effects, per-key RGB, battery |
| Corsair | Bragi and legacy (NXP) | ~45 models — M65, Dark Core, Katar, Ironclaw, Harpoon, Scimitar, K55/K65/K70/K95/K100, Slipstream receivers: DPI, polling, static zone and per-key colour, battery |
| SteelSeries | Rival / Sensei / Aerox / Prime | Sensitivity presets, polling, colours, button mapping, save to device |

Tested on real hardware so far: Logitech G502 HERO (wired, onboard
profiles — read, write, restore verified byte-for-byte), Lightspeed
receiver (C539), Corsair K55 RGB PRO (zone lighting). Everything else is implemented from
the published protocol work of libratbag, Solaar, OpenRazer, OpenRGB,
ckb-next and rivalcfg and covered by byte-exact unit tests against emulated
firmware — reports from owners of other devices are very welcome.

## Device artwork

Supported mice are drawn as original vector renditions of the real product,
with every button labelled where it physically is and lighting zones glowing
in their live colours:

| Brand | Models drawn |
|---|---|
| Logitech | G502 HERO / Lightspeed, G502 X / X Plus, G Pro X Superlight 2 / G Pro Wireless, G305 / G304, G703 / G403, G903, G600, MX Master 3S, MX Anywhere 3S |
| Razer | DeathAdder V3 (Pro), Viper V3 Pro, Viper Ultimate / V2 Pro, Basilisk V3 (Pro), Naga V2 Pro / Pro, Cobra (Pro) |
| Corsair | M65 RGB Ultra / Elite / Pro, Dark Core RGB Pro, Scimitar Elite, Katar Pro, Harpoon RGB |
| SteelSeries | Rival 3, Rival 600 / 650 / 310, Aerox 3 / 5 / 9, Prime family, Sensei Ten / 310 |

Keyboards are drawn from their physical layout (macro columns, media keys,
wrist rest — e.g. the K55 RGB PRO) or their per-key layout. Anything else
falls back to a generic silhouette.

## Install

### Ubuntu (PPA)

```bash
sudo add-apt-repository ppa:dhiva-labs/apps
sudo apt install gearwise
```

The package installs a udev rule that lets the logged-in user open the
devices and `/dev/uinput` (for macros) — replug your devices afterwards.

### Debian / Ubuntu (.deb) and other Linux

Download from the [latest release](https://github.com/Dhiva-Labs/gearwise/releases/latest):

| | x64 (x86-64) | Arm64 (aarch64) |
|---|---|---|
| Debian / Ubuntu package | `gearwise_*_amd64.deb` | `gearwise_*_arm64.deb` |
| Any Linux | `gearwise-*-linux-x86_64.tar.gz` | `gearwise-*-linux-aarch64.tar.gz` |

* `.deb` → `sudo apt install ./gearwise_*_<arch>.deb` (`amd64` or `arm64`)
* tarball → extract, run `sudo ./install-udev-rules.sh` once, then `./gearwise`

Linux builds need glibc 2.35+ (Ubuntu 22.04 or newer), on x86-64 or Arm64
(aarch64). Every release also lists a `SHA256SUMS` file for all packages.

### Snap Store

Submitted; the snap needs a store review for raw HID and uinput access
before it can be published (see [packaging/SNAP.md](packaging/SNAP.md)).

### Windows

Download `gearwise-*-windows-x86_64.zip` (most PCs) or
`gearwise-*-windows-aarch64.zip` (Windows on Arm, e.g. Snapdragon X laptops)
from the [latest release](https://github.com/Dhiva-Labs/gearwise/releases/latest)
and run `gearwise.exe`. Close G HUB / Synapse / iCUE / GG first: they can
hold devices exclusively or overwrite settings.

### From source

```bash
cargo build --release -p gearwise
sudo packaging/linux/install-udev-rules.sh   # Linux, once
```

## How it works

```
app (egui UI)  ──commands──▶  worker thread  ──▶  DeviceManager
      ▲                           │                   │
      └────────events─────────────┘          drivers: logitech · razer
                                              corsair · steelseries
                                                       │
                                               gearwise-hid (hidapi)
```

| Crate | Purpose |
|---|---|
| `gearwise-core` | Vendor-neutral model: capabilities, state, settings, profiles, macros, device manager, demo devices, mock transport for tests |
| `gearwise-hid` | hidapi backend (pure-Rust on both platforms — no libudev or C toolchain needed) |
| `gearwise-logitech` / `-razer` / `-corsair` / `-steelseries` | Protocol drivers |
| `gearwise-input` | Macro playback (`uinput` / `SendInput`) and focused-application detection |
| `app` | The desktop UI |

The UI never blocks on device I/O: changes are applied optimistically and
confirmed by the worker. Onboard-memory writes are batched (a short delay
after the last change) so dragging a slider never hammers flash.

## Development

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p gearwise-logitech --example logitech_probe   # read-only dump of connected Logitech devices
cargo run -p gearwise-corsair --example corsair_probe
```

## Sponsors

Dhiva Labs is an independent, self-funded lab. [Sponsorships](https://github.com/sponsors/Dhiva-Labs) help bring this project and the rest of Dhiva Labs to every platform, iPhone and Mac included.

[![Sponsor Dhiva Labs](https://img.shields.io/badge/Sponsor-Dhiva%20Labs-ea4aaa?logo=githubsponsors&logoColor=white)](https://github.com/sponsors/Dhiva-Labs)

Monthly sponsors at $10 and up are listed here:

<!-- sponsors -->
*Be the first: [sponsor Dhiva Labs](https://github.com/sponsors/Dhiva-Labs).*
<!-- sponsors -->

## License

MIT
