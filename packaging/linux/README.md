# Linux packaging

## udev rules

`70-periph-studio.rules` lets the user logged in at the console talk to their
Logitech (046d), Razer (1532), Corsair (1b1c) and SteelSeries (1038) devices
without root, and to `/dev/uinput` for macro playback. It works through
`TAG+="uaccess"`, which gives the active seat's user an ACL on the node; no
group membership is needed.

Install (asks for sudo/pkexec when not root, safe to re-run):

    ./install-udev-rules.sh

Then unplug and replug the device, or log out and back in, if it is still
refused. Remove with `./install-udev-rules.sh --uninstall`.

Notes:

- The `70-` prefix matters: `73-seat-late.rules` converts the `uaccess` tag
  into an ACL, so the tag must be set before it runs.
- The ACL only exists for the user at the physical console. SSH sessions and
  services need an explicit `GROUP=`/`MODE=` rule of their own.
- The rules cover USB devices (matched on `idVendor`) and Bluetooth devices
  (matched on the HID parent name `0005:<VID>:*`).
- `static_node=uinput` lets udev tag `/dev/uinput` before the module is
  loaded. If the node is still missing, run `sudo modprobe uinput` (add
  `uinput` to `/etc/modules-load.d/` to make it permanent).
- Debian/RPM packages should ship the file as
  `/usr/lib/udev/rules.d/70-periph-studio.rules` instead of running the script.
- Check the syntax with `udevadm verify 70-periph-studio.rules`.
