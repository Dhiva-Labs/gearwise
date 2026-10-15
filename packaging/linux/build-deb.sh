#!/bin/sh
# Assemble a binary .deb from a release build.
#
#   packaging/linux/build-deb.sh <path/to/gearwise> <version> <out-dir> [arch]
#
# arch is the Debian architecture of the binary (amd64, arm64); it defaults to
# the architecture of the machine the script runs on.
#
# Layout: binary in /usr/bin, desktop entry and hicolor icons, the udev rule
# in /usr/lib/udev/rules.d (so the logged-in user can open the devices and
# /dev/uinput), MIT copyright. postinst reloads udev so plugged-in devices
# are usable right after installing.
set -eu

bin=$1
version=$2
out=$3
here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
root=$(CDPATH='' cd -- "$here/../.." && pwd)
arch=${4:-$(dpkg --print-architecture)}
pkg=gearwise_${version}_${arch}
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
d=$stage/$pkg

install -Dm755 "$bin" "$d/usr/bin/gearwise"
install -Dm644 "$here/gearwise.desktop" "$d/usr/share/applications/gearwise.desktop"
for s in 16 32 48 64 128 256 512; do
    install -Dm644 "$root/packaging/icons/gearwise-$s.png" \
        "$d/usr/share/icons/hicolor/${s}x${s}/apps/gearwise.png"
done
install -Dm644 "$here/70-gearwise.rules" "$d/usr/lib/udev/rules.d/70-gearwise.rules"
install -Dm644 "$root/LICENSE" "$d/usr/share/doc/gearwise/copyright"
gzip -9cn "$root/CHANGELOG.md" > "$stage/changelog.gz"
install -Dm644 "$stage/changelog.gz" "$d/usr/share/doc/gearwise/changelog.gz"

size=$(du -sk "$d/usr" | cut -f1)
mkdir -p "$d/DEBIAN"
cat > "$d/DEBIAN/control" <<CTRL
Package: gearwise
Version: $version
Architecture: $arch
Maintainer: Dhiva Labs <dhivakar1010@gmail.com>
Installed-Size: $size
Depends: libc6 (>= 2.35), libgcc-s1, libgl1, libegl1, libxkbcommon0, libwayland-client0, libwayland-cursor0, libx11-6, libxcursor1, libxrandr2, libxi6, libxkbcommon-x11-0
Recommends: udev
Section: utils
Priority: optional
Homepage: https://github.com/Dhiva-Labs/gearwise
Description: Configure gaming mice and keyboards
 Gearwise configures mice and keyboards from Logitech, Razer, Corsair and
 SteelSeries: DPI stages, report rate, RGB lighting (zones and per-key),
 button assignments, onboard profiles, macros and per-application
 profiles, in one lightweight native app.
CTRL
cat > "$d/DEBIAN/postinst" <<'POST'
#!/bin/sh
set -e
if [ "$1" = configure ] && command -v udevadm >/dev/null 2>&1; then
    udevadm control --reload-rules || true
    udevadm trigger --subsystem-match=hidraw --subsystem-match=misc || true
fi
POST
cat > "$d/DEBIAN/postrm" <<'POST'
#!/bin/sh
set -e
if [ "$1" = remove ] && command -v udevadm >/dev/null 2>&1; then
    udevadm control --reload-rules || true
fi
POST
chmod 755 "$d/DEBIAN/postinst" "$d/DEBIAN/postrm"

mkdir -p "$out"
dpkg-deb --root-owner-group --build "$d" "$out/$pkg.deb" >/dev/null
echo "$out/$pkg.deb"
