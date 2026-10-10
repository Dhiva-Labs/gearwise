//! Colour math for animated lighting previews. The device renders the real
//! effect; this only approximates it on screen.

use periph_core::model::{LightingEffect, Rgb, WaveDirection};

/// Colour an LED at horizontal position `pos` (0 = left, 1 = right) shows at
/// time `t` seconds. `salt` decorrelates LEDs for the reactive effect.
pub fn sample(effect: &LightingEffect, brightness: u8, t: f32, pos: f32, salt: u32) -> Rgb {
    let rgb = match *effect {
        LightingEffect::Off => Rgb::BLACK,
        LightingEffect::Static { color } => color,
        LightingEffect::Breathing { color, period_ms } => {
            let phase = t * 1000.0 / f32::from(period_ms.max(1));
            scale(color, 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos())
        }
        LightingEffect::ColorCycle { period_ms } => {
            Rgb::from_hue(360.0 * fract(t * 1000.0 / f32::from(period_ms.max(1))))
        }
        LightingEffect::Wave {
            period_ms,
            direction,
        } => {
            let travel = fract(t * 1000.0 / f32::from(period_ms.max(1)));
            let hue = match direction {
                WaveDirection::LeftToRight => pos - travel,
                WaveDirection::RightToLeft => pos + travel,
            };
            Rgb::from_hue(360.0 * fract(hue))
        }
        LightingEffect::Reactive { color, duration_ms } => {
            // Each LED is "pressed" once every 2.4 s at its own offset and fades over `duration_ms`.
            let cycle = 2.4;
            let offset = (salt.wrapping_mul(2_654_435_761) >> 8) as f32 / 16_777_216.0 * cycle;
            let since = fract((t + offset) / cycle) * cycle;
            let fade = f32::from(duration_ms.max(1)) / 1000.0;
            let lit = (1.0 - since / fade).clamp(0.0, 1.0);
            scale(color, 0.12 + 0.88 * lit)
        }
    };
    rgb.scaled(brightness)
}

fn scale(c: Rgb, k: f32) -> Rgb {
    let k = k.clamp(0.0, 1.0);
    Rgb::new(
        (f32::from(c.r) * k) as u8,
        (f32::from(c.g) * k) as u8,
        (f32::from(c.b) * k) as u8,
    )
}

fn fract(x: f32) -> f32 {
    x - x.floor()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Rgb = Rgb::new(255, 0, 0);

    #[test]
    fn off_is_black_and_static_is_its_colour() {
        assert_eq!(sample(&LightingEffect::Off, 100, 1.0, 0.5, 0), Rgb::BLACK);
        assert_eq!(
            sample(&LightingEffect::Static { color: RED }, 100, 3.3, 0.1, 0),
            RED
        );
    }

    #[test]
    fn brightness_scales_the_result() {
        let c = sample(&LightingEffect::Static { color: RED }, 50, 0.0, 0.0, 0);
        assert_eq!(c, Rgb::new(127, 0, 0));
        assert_eq!(
            sample(&LightingEffect::Static { color: RED }, 0, 0.0, 0.0, 0),
            Rgb::BLACK
        );
    }

    #[test]
    fn breathing_goes_dark_and_bright_over_one_period() {
        let e = LightingEffect::Breathing {
            color: RED,
            period_ms: 2000,
        };
        assert_eq!(sample(&e, 100, 0.0, 0.0, 0).r, 0);
        assert!(sample(&e, 100, 1.0, 0.0, 0).r >= 254);
        assert!(sample(&e, 100, 2.0, 0.0, 0).r <= 1);
    }

    #[test]
    fn colour_cycle_starts_red_and_is_periodic() {
        let e = LightingEffect::ColorCycle { period_ms: 4000 };
        assert_eq!(sample(&e, 100, 0.0, 0.3, 0), RED);
        assert_eq!(sample(&e, 100, 4.0, 0.3, 0), sample(&e, 100, 0.0, 0.3, 0));
        assert_ne!(sample(&e, 100, 1.0, 0.0, 0), RED);
    }

    #[test]
    fn wave_moves_with_direction() {
        let ltr = LightingEffect::Wave {
            period_ms: 4000,
            direction: WaveDirection::LeftToRight,
        };
        let rtl = LightingEffect::Wave {
            period_ms: 4000,
            direction: WaveDirection::RightToLeft,
        };
        // At t = 0 hue follows position; a quarter period later the pattern has shifted.
        assert_eq!(sample(&ltr, 100, 0.0, 0.0, 0), RED);
        assert_eq!(sample(&ltr, 100, 1.0, 0.25, 0), RED);
        assert_eq!(sample(&rtl, 100, 1.0, 0.75, 0), RED);
    }

    #[test]
    fn reactive_fades_after_a_press() {
        let e = LightingEffect::Reactive {
            color: RED,
            duration_ms: 500,
        };
        let samples: Vec<u8> = (0..240)
            .map(|i| sample(&e, 100, i as f32 * 0.01, 0.0, 7).r)
            .collect();
        assert!(samples.iter().any(|r| *r >= 250), "key lights up");
        assert!(samples.iter().any(|r| *r <= 40), "key rests dim");
    }
}
