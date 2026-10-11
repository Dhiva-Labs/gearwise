use super::*;

#[test]
fn colour_edits_only_touch_effects_that_have_a_colour() {
    let red = Rgb::new(255, 0, 0);
    assert_eq!(
        with_color(LightingEffect::Static { color: Rgb::BLACK }, red),
        LightingEffect::Static { color: red }
    );
    assert_eq!(
        with_color(
            LightingEffect::Breathing {
                color: Rgb::BLACK,
                period_ms: 900
            },
            red
        ),
        LightingEffect::Breathing {
            color: red,
            period_ms: 900
        }
    );
    assert_eq!(
        with_color(LightingEffect::ColorCycle { period_ms: 5 }, red),
        LightingEffect::ColorCycle { period_ms: 5 }
    );
}

#[test]
fn period_edits_keep_other_fields() {
    let c = Rgb::new(1, 2, 3);
    assert_eq!(
        with_period(
            LightingEffect::Reactive {
                color: c,
                duration_ms: 1
            },
            700
        ),
        LightingEffect::Reactive {
            color: c,
            duration_ms: 700
        }
    );
    assert_eq!(
        with_period(
            LightingEffect::Wave {
                period_ms: 1,
                direction: WaveDirection::RightToLeft
            },
            9000
        ),
        LightingEffect::Wave {
            period_ms: 9000,
            direction: WaveDirection::RightToLeft
        }
    );
    assert_eq!(with_period(LightingEffect::Off, 10), LightingEffect::Off);
}

#[test]
fn direction_applies_to_waves_only() {
    let w = LightingEffect::Wave {
        period_ms: 3000,
        direction: WaveDirection::LeftToRight,
    };
    assert_eq!(
        with_direction(w, WaveDirection::RightToLeft),
        LightingEffect::Wave {
            period_ms: 3000,
            direction: WaveDirection::RightToLeft
        }
    );
    assert_eq!(
        with_direction(LightingEffect::Off, WaveDirection::RightToLeft),
        LightingEffect::Off
    );
}

#[test]
fn label_ink_contrasts_with_the_key() {
    assert_eq!(text_on(Color32::WHITE), Color32::from_rgb(0x12, 0x15, 0x1b));
    assert_eq!(
        text_on(Color32::from_rgb(10, 10, 40)),
        Color32::from_rgb(0xf2, 0xf4, 0xf8)
    );
}
