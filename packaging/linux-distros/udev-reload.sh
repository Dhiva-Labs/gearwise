#!/bin/sh
# rpm %post/%postun and pacman post_install/post_remove: reload the udev
# rules and re-apply them to the hidraw and uinput nodes that already exist,
# so plugged-in devices work without a replug. Without a running udev (a
# container, a chroot) there is nothing to reload; the rule applies at boot.
if command -v udevadm > /dev/null 2>&1; then
  udevadm control --reload-rules 2> /dev/null || :
  udevadm trigger --subsystem-match=hidraw --subsystem-match=misc 2> /dev/null || :
fi
