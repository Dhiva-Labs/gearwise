//! egui keys to HID keyboard usages, and keystroke capture helpers.

use std::time::Instant;

use egui::Key;
use periph_core::model::Modifiers;

/// HID usage (page 0x07) of a physical key, where one exists.
pub fn hid_usage(key: Key) -> Option<u8> {
    use Key::*;
    Some(match key {
        A => 0x04,
        B => 0x05,
        C => 0x06,
        D => 0x07,
        E => 0x08,
        F => 0x09,
        G => 0x0a,
        H => 0x0b,
        I => 0x0c,
        J => 0x0d,
        K => 0x0e,
        L => 0x0f,
        M => 0x10,
        N => 0x11,
        O => 0x12,
        P => 0x13,
        Q => 0x14,
        R => 0x15,
        S => 0x16,
        T => 0x17,
        U => 0x18,
        V => 0x19,
        W => 0x1a,
        X => 0x1b,
        Y => 0x1c,
        Z => 0x1d,
        Num1 | Exclamationmark => 0x1e,
        Num2 => 0x1f,
        Num3 => 0x20,
        Num4 => 0x21,
        Num5 => 0x22,
        Num6 => 0x23,
        Num7 => 0x24,
        Num8 => 0x25,
        Num9 => 0x26,
        Num0 => 0x27,
        Enter => 0x28,
        Escape => 0x29,
        Backspace => 0x2a,
        Tab => 0x2b,
        Space => 0x2c,
        Minus => 0x2d,
        Equals | Plus => 0x2e,
        OpenBracket | OpenCurlyBracket => 0x2f,
        CloseBracket | CloseCurlyBracket => 0x30,
        Backslash | Pipe => 0x31,
        Semicolon | Colon => 0x33,
        Quote => 0x34,
        Backtick => 0x35,
        Comma => 0x36,
        Period => 0x37,
        Slash | Questionmark => 0x38,
        F1 => 0x3a,
        F2 => 0x3b,
        F3 => 0x3c,
        F4 => 0x3d,
        F5 => 0x3e,
        F6 => 0x3f,
        F7 => 0x40,
        F8 => 0x41,
        F9 => 0x42,
        F10 => 0x43,
        F11 => 0x44,
        F12 => 0x45,
        Insert => 0x49,
        Home => 0x4a,
        PageUp => 0x4b,
        Delete => 0x4c,
        End => 0x4d,
        PageDown => 0x4e,
        ArrowRight => 0x4f,
        ArrowLeft => 0x50,
        ArrowDown => 0x51,
        ArrowUp => 0x52,
        IntlBackslash => 0x64,
        F13 => 0x68,
        F14 => 0x69,
        F15 => 0x6a,
        F16 => 0x6b,
        F17 => 0x6c,
        F18 => 0x6d,
        F19 => 0x6e,
        F20 => 0x6f,
        F21 => 0x70,
        F22 => 0x71,
        F23 => 0x72,
        F24 => 0x73,
        ControlLeft => 0xe0,
        ShiftLeft => 0xe1,
        AltLeft => 0xe2,
        SuperLeft => 0xe3,
        ControlRight => 0xe4,
        ShiftRight => 0xe5,
        AltRight => 0xe6,
        SuperRight => 0xe7,
        _ => return None,
    })
}

/// Modifier bit for a modifier usage (0xe0..=0xe7).
pub fn modifier_bit(usage: u8) -> Option<u8> {
    (0xe0..=0xe7)
        .contains(&usage)
        .then(|| 1u8 << (usage - 0xe0))
}

/// One physical key transition seen by the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEv {
    pub usage: u8,
    pub pressed: bool,
    pub at: Instant,
}

/// Remove key and text events from this frame's input (so widgets never see
/// them while a capture is active) and return them as HID transitions.
pub fn take_key_events(ctx: &egui::Context) -> Vec<KeyEv> {
    let now = Instant::now();
    let mut out = Vec::new();
    ctx.input_mut(|i| {
        i.events.retain(|e| match e {
            egui::Event::Key {
                key,
                physical_key,
                pressed,
                repeat,
                ..
            } => {
                if !*repeat
                    && let Some(usage) =
                        hid_usage(physical_key.unwrap_or(*key)).or_else(|| hid_usage(*key))
                {
                    out.push(KeyEv {
                        usage,
                        pressed: *pressed,
                        at: now,
                    });
                }
                false
            }
            egui::Event::Text(_) | egui::Event::Copy | egui::Event::Cut | egui::Event::Paste(_) => {
                false
            }
            _ => true,
        });
    });
    out
}

/// Turns a stream of key transitions into one keystroke: a key plus the
/// modifiers held with it, or a modifier tapped on its own.
#[derive(Default, Debug)]
pub struct Chord {
    held: u8,
    lone: Option<u8>,
}

impl Chord {
    pub fn feed(&mut self, ev: &KeyEv) -> Option<(u8, Modifiers)> {
        match (modifier_bit(ev.usage), ev.pressed) {
            (Some(bit), true) => {
                self.held |= bit;
                self.lone = Some(ev.usage);
                None
            }
            (Some(bit), false) => {
                self.held &= !bit;
                (self.lone.take() == Some(ev.usage)).then_some((ev.usage, Modifiers(self.held)))
            }
            (None, true) => {
                self.lone = None;
                Some((ev.usage, Modifiers(self.held)))
            }
            (None, false) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(usage: u8, pressed: bool) -> KeyEv {
        KeyEv {
            usage,
            pressed,
            at: Instant::now(),
        }
    }

    #[test]
    fn known_keys_map_to_hid_usages() {
        assert_eq!(hid_usage(Key::A), Some(0x04));
        assert_eq!(hid_usage(Key::Z), Some(0x1d));
        assert_eq!(hid_usage(Key::Num1), Some(0x1e));
        assert_eq!(hid_usage(Key::Num0), Some(0x27));
        assert_eq!(hid_usage(Key::Enter), Some(0x28));
        assert_eq!(hid_usage(Key::Escape), Some(0x29));
        assert_eq!(hid_usage(Key::F1), Some(0x3a));
        assert_eq!(hid_usage(Key::F12), Some(0x45));
        assert_eq!(hid_usage(Key::ArrowRight), Some(0x4f));
        assert_eq!(hid_usage(Key::ShiftLeft), Some(0xe1));
        assert_eq!(hid_usage(Key::SuperRight), Some(0xe7));
        assert_eq!(hid_usage(Key::Copy), None);
    }

    #[test]
    fn every_mapped_key_has_a_name_in_the_key_table() {
        for key in Key::ALL {
            if let Some(u) = hid_usage(*key)
                && u != 0x64
            {
                assert!(
                    periph_core::keys::name(u).is_some(),
                    "{key:?} -> 0x{u:02x} has no name"
                );
            }
        }
    }

    #[test]
    fn letters_and_digits_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for key in Key::ALL {
            let letter_or_digit = format!("{key:?}");
            if (letter_or_digit.len() == 1
                && letter_or_digit.chars().all(|c| c.is_ascii_uppercase()))
                || letter_or_digit.starts_with("Num")
            {
                assert!(
                    seen.insert(hid_usage(*key).unwrap()),
                    "duplicate usage for {key:?}"
                );
            }
        }
        assert_eq!(seen.len(), 36);
    }

    #[test]
    fn modifier_bits_follow_hid_layout() {
        assert_eq!(modifier_bit(0xe0), Some(Modifiers::LCTRL));
        assert_eq!(modifier_bit(0xe1), Some(Modifiers::LSHIFT));
        assert_eq!(modifier_bit(0xe7), Some(Modifiers::RGUI));
        assert_eq!(modifier_bit(0x04), None);
    }

    #[test]
    fn chord_captures_key_with_modifiers() {
        let mut c = Chord::default();
        assert_eq!(c.feed(&ev(0xe0, true)), None);
        assert_eq!(c.feed(&ev(0xe1, true)), None);
        let (u, m) = c.feed(&ev(0x04, true)).unwrap();
        assert_eq!(u, 0x04);
        assert_eq!(m.0, Modifiers::LCTRL | Modifiers::LSHIFT);
        assert_eq!(c.feed(&ev(0x04, false)), None);
    }

    #[test]
    fn chord_captures_lone_modifier_on_release() {
        let mut c = Chord::default();
        assert_eq!(c.feed(&ev(0xe1, true)), None);
        assert_eq!(c.feed(&ev(0xe1, false)), Some((0xe1, Modifiers(0))));
        // A modifier used in a combo is not captured on its own afterwards.
        c.feed(&ev(0xe0, true));
        c.feed(&ev(0x06, true));
        assert_eq!(c.feed(&ev(0xe0, false)), None);
    }
}
