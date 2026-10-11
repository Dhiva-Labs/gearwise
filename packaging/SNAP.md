# Publishing Gearwise to the Snap Store

## Why this snap needs a store review

Gearwise talks to mice and keyboards through their HID interfaces
(`/dev/hidraw*`) and plays macros through `/dev/uinput`. Under strict
confinement there is no interface a desktop snap can use for that without a
store declaration:

* `hidraw` only connects to slots provided by gadget snaps (Ubuntu Core);
  classic desktops have none.
* `raw-usb` covers `/dev/bus/usb` only, not hidraw — and claiming a HID
  interface through libusb would detach the kernel driver, which on a
  Logitech receiver kills the mouse's input.
* `uinput` plugs are `allow-installation: false` without a declaration.

So the snap declares a `custom-device` slot/plug pair for `/dev/hidraw*` plus
the `uinput` plug, and the first upload goes to **manual review**. Nothing
can be released until the store grants the declaration, and a revision in
review blocks later releases of the snap (see BranchPilot's packaging notes),
so do not upload unrelated builds while it waits.

Snap interfaces do not change Unix permissions: users still need the udev
rule. A confined snap cannot write `/etc/udev`, so the rule ships inside the
snap and the description gives the one-time copy command.

## Build

Docker's `FORWARD DROP` policy cuts LXD's bridge, so while Docker runs the
build container cannot reach the store. Allow the bridge first (sudo,
non-persistent), then build:

```bash
sudo iptables -I DOCKER-USER -i lxdbr0 -j ACCEPT
sudo iptables -I DOCKER-USER -o lxdbr0 -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
snapcraft pack --use-lxd
```

## Test locally

```bash
sudo snap install --dangerous ./gearwise_0.1.0_amd64.snap
sudo cp /snap/gearwise/current/udev/70-gearwise.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
sudo snap connect gearwise:hidraw-devices gearwise:hidraw
sudo snap connect gearwise:uinput
sudo snap connect gearwise:hardware-observe
snap run gearwise
```

## Upload

```bash
snapcraft register gearwise          # once
snapcraft upload gearwise_0.1.0_amd64.snap
snapcraft revisions gearwise         # shows the revision waiting for review
```

## Store request

Posted for revision 1 on 2026-10-11 as a comment on the revision's review
page (dashboard.snapcraft.io → gearwise → Revisions → #1 → "Add feedback for
this revision"), which reaches the reviewers directly. The same text works
on the forum (https://forum.snapcraft.io/c/store-requests/19) if they ask for it there.

Text:

Title: **Request: custom-device (hidraw) and uinput for gearwise**

> Hi! I'd like to request installation of the `custom-device` slot/plug pair
> and the `uinput` plug for the snap **gearwise**
> (https://snapcraft.io/gearwise, source: https://github.com/Dhiva-Labs/gearwise).
>
> **What it is:** Gearwise is an open-source (MIT) configuration app for
> gaming mice and keyboards from Logitech, Razer, Corsair and SteelSeries —
> an alternative to vendor tools such as G HUB, Synapse and iCUE, which
> don't exist on Linux. It sets DPI, polling rate, RGB lighting, button
> assignments and onboard profiles, and plays user-defined macros.
>
> **Why custom-device (hidraw):** every one of these devices is configured
> through vendor-specific HID reports (Logitech HID++ 2.0, Razer's 90-byte
> feature reports, Corsair Bragi, SteelSeries' protocol) on the device's
> hidraw node. The `hidraw` interface only works with gadget-provided slots,
> so it can't be used on classic desktops, and `raw-usb` would require
> detaching the kernel HID driver — which on wireless receivers disconnects
> the user's mouse. The snap's slot is limited to `/dev/hidraw*` with
> `subsystem: hidraw` udev tagging, and the app only ever opens the vendor
> control interface of supported devices (vendor IDs 046d, 1532, 1b1c,
> 1038), never keyboard input interfaces.
>
> **Why uinput:** buttons assigned to macros or keystrokes on devices
> without onboard memory are played back through a virtual input device,
> the same mechanism input-remapper and similar tools use.
>
> **Auto-connection:** not requested; the description tells users to
> connect `hidraw-devices`, `uinput` and `hardware-observe` manually and
> to install the shipped udev rule.
>
> Thanks!
