# Changelog

## 0.1.1 — 2026-10-16

- Added: Linux arm64 and Windows arm64 builds next to x64:
  `gearwise-<version>-linux-aarch64.tar.gz`, `gearwise_<version>_arm64.deb` and
  `gearwise-<version>-windows-aarch64.zip`, each built and started natively on
  Arm in CI. The snap now builds for arm64 as well.
- Fixed: the strict snap now gets Wayland and GPU access (Mesa from the
  `gpu-2404` interface, the `opengl` plug and an absolute `WAYLAND_DISPLAY`
  socket path), and its desktop entry is picked up from `snap/gui`.
- Docs: PPA, release downloads and Snap Store status; README names the arm64
  downloads and carries build badges.

## 0.1.0 — 2026-10-11

First release.

- Configure mice and keyboards from Logitech, Razer, Corsair and
  SteelSeries in one lightweight native app for Linux and Windows.
- **Logitech** (HID++ 2.0, wired, Lightspeed / Unifying / Bolt receivers,
  Bluetooth): DPI stages, report rate up to 8 kHz, LED zones, per-key RGB,
  onboard profiles with button bindings, G-keys and diverted buttons for
  software actions, battery.
- **Razer** (34 models): DPI and stages, polling incl. HyperPolling, zone
  effects, per-key RGB, battery.
- **Corsair** (~45 models, Bragi and legacy): DPI, polling, zone and
  per-key colour, battery.
- **SteelSeries** (55 models): sensitivity presets, polling, colours,
  button mapping, save to device.
- Button assignments to mouse buttons, keystrokes with modifiers, media
  keys, DPI functions, profile cycle or macros.
- Macro recorder and editor; playback through uinput (Linux) and
  SendInput (Windows).
- Profiles record every change and re-apply it when a device connects.
- Model-accurate artwork for 27 popular mice and the Corsair K55 RGB PRO,
  with labelled buttons and live lighting previews.
- Demo devices to explore the app without hardware.
