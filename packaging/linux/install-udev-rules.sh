#!/bin/sh
# Install (or remove) the Periph Studio udev rules.
#
#   install-udev-rules.sh              install / update, reload udev
#   install-udev-rules.sh --uninstall  remove the rules, reload udev
#   install-udev-rules.sh --no-reload  copy only, skip the udevadm calls
#
# Needs root to write /etc/udev/rules.d; re-runs itself through sudo (when
# attached to a terminal) or pkexec (when started from a desktop launcher).
# Safe to run any number of times.
#
# UDEV_RULES_DIR overrides the destination directory (packagers, testing).

set -eu

RULES_NAME=70-periph-studio.rules
DEST_DIR=${UDEV_RULES_DIR:-/etc/udev/rules.d}

self_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
self=$self_dir/$(basename -- "$0")
src=$self_dir/$RULES_NAME
dest=$DEST_DIR/$RULES_NAME

action=install
reload=yes
for arg in "$@"; do
    case $arg in
        --uninstall) action=uninstall ;;
        --no-reload) reload=no ;;
        -h | --help)
            sed -n '2,12p' "$self" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "install-udev-rules: unknown option '$arg'" >&2
            exit 2
            ;;
    esac
done

die() {
    echo "install-udev-rules: $*" >&2
    exit 1
}

# --- Get root if we need it -------------------------------------------------
if [ "$(id -u)" -ne 0 ]; then
    if [ -d "$DEST_DIR" ] && [ -w "$DEST_DIR" ] && { [ ! -e "$dest" ] || [ -w "$dest" ]; }; then
        : # writable as the current user (e.g. UDEV_RULES_DIR in a scratch dir)
    elif [ "$DEST_DIR" != /etc/udev/rules.d ]; then
        die "$DEST_DIR is not writable by this user"
    elif [ -t 0 ] && command -v sudo >/dev/null 2>&1; then
        echo "Root is needed to write $DEST_DIR; re-running with sudo."
        exec sudo "$self" "$@"
    elif command -v pkexec >/dev/null 2>&1; then
        echo "Root is needed to write $DEST_DIR; re-running with pkexec."
        exec pkexec "$self" "$@"
    elif command -v sudo >/dev/null 2>&1; then
        echo "Root is needed to write $DEST_DIR; re-running with sudo."
        exec sudo "$self" "$@"
    else
        die "must run as root (neither sudo nor pkexec found)"
    fi
fi

# --- Do the work ------------------------------------------------------------
case $action in
    install)
        [ -r "$src" ] || die "cannot read $src"
        [ -d "$DEST_DIR" ] || mkdir -p -- "$DEST_DIR"
        if [ -f "$dest" ] && cmp -s -- "$src" "$dest"; then
            echo "Rules already up to date: $dest"
        else
            [ -f "$dest" ] && verb=Updated || verb=Installed
            tmp=$dest.tmp.$$
            trap 'rm -f -- "$tmp"' EXIT
            cp -- "$src" "$tmp"
            chmod 0644 -- "$tmp"
            mv -f -- "$tmp" "$dest"
            trap - EXIT
            echo "$verb $dest"
        fi
        ;;
    uninstall)
        if [ -e "$dest" ]; then
            rm -f -- "$dest"
            echo "Removed $dest"
        else
            echo "Nothing to remove: $dest does not exist"
        fi
        ;;
esac

if [ "$reload" = no ]; then
    echo "Skipped udev reload (--no-reload)."
elif ! command -v udevadm >/dev/null 2>&1; then
    echo "udevadm not found; skipped reload. Reboot or replug devices to apply."
elif [ "$DEST_DIR" != /etc/udev/rules.d ]; then
    echo "Custom destination; skipped udev reload."
else
    # Reload even when nothing changed: it is cheap and repairs the case where
    # the file was copied by hand without telling udev.
    if udevadm control --reload-rules && udevadm trigger --subsystem-match=hidraw --subsystem-match=misc; then
        echo "Reloaded udev rules and re-applied them to existing hidraw/uinput nodes."
    else
        echo "udevadm failed; the rules apply after a reboot or when devices are replugged." >&2
    fi
fi

if [ "$action" = install ]; then
    echo "Done. If a device is still inaccessible, unplug and replug it (or log out and in)."
else
    echo "Done. Access granted by the rules ends when the nodes are next recreated."
fi
