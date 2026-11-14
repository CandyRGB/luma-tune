use serde::{Deserialize, Serialize};

use crate::{curve, ops, white};

pub type Ramp = [u16; 768];

pub const BRIGHTNESS_RANGE: f32 = 0.12;
pub const TONE_RANGE: f32 = 0.3;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Levels {
    pub in_black: f32,
    pub in_white: f32,
    pub out_black: f32,
    pub out_white: f32,
}

impl Default for Levels {
    fn default() -> Self {
        Self {
            in_black: 0.0,
            in_white: 1.0,
            out_black: 0.0,
            out_white: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterParams {
    pub levels: Levels,
    #[serde(default)]
    pub brightness: f32,
    #[serde(default = "one")]
    pub contrast: f32,
    #[serde(default)]
    pub shadows: f32,
    #[serde(default)]
    pub highlights: f32,
    #[serde(default = "one")]
    pub gamma: f32,
    #[serde(default = "neutral_temp")]
    pub temperature: u32,
    #[serde(default)]
    pub tint: f32,
    #[serde(default = "unity_gains")]
    pub gains: [f32; 3],
    #[serde(default)]
    pub curve: Vec<[f32; 2]>,
}

fn one() -> f32 {
    1.0
}

fn neutral_temp() -> u32 {
    6500
}

fn unity_gains() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            levels: Levels::default(),
            brightness: 0.0,
            contrast: 1.0,
            shadows: 0.0,
            highlights: 0.0,
            gamma: 1.0,
            temperature: 6500,
            tint: 0.0,
            gains: [1.0, 1.0, 1.0],
            curve: Vec::new(),
        }
    }
}

impl FilterParams {
    pub fn is_neutral(&self) -> bool {
        *self == Self::default()
    }

    pub fn channel_gains(&self) -> [f32; 3] {
        let w = white::gains(self.temperature);
        let tint = self.tint.clamp(-1.0, 1.0);
        let green = 1.0 - 0.25 * tint;
        let edge = 1.0 + 0.12 * tint;
        [
            w[0] * self.gains[0] * edge,
            w[1] * self.gains[1] * green,
            w[2] * self.gains[2] * edge,
        ]
    }

    pub fn evaluate(&self, x: f32) -> [f32; 3] {
        let g = self.channel_gains();
        let brightness = self.brightness.clamp(-BRIGHTNESS_RANGE, BRIGHTNESS_RANGE);
        let shadows = self.shadows.clamp(-TONE_RANGE, TONE_RANGE);
        let highlights = self.highlights.clamp(-TONE_RANGE, TONE_RANGE);
        let mut out = [0.0f32; 3];
        for (c, slot) in out.iter_mut().enumerate() {
            let mut v = ops::input_levels(x, &self.levels);
            v = ops::brightness(v, brightness);
            v = ops::contrast(v, self.contrast);
            v = ops::shadows(v, shadows);
            v = ops::highlights(v, highlights);
            v = ops::gamma(v, self.gamma);
            v *= g[c];
            v = curve::sample(&self.curve, v);
            v = ops::output_levels(v, &self.levels);
            *slot = v.clamp(0.0, 1.0);
        }
        out
    }

    pub fn to_ramp(&self) -> Ramp {
        let mut ramp: Ramp = [0; 768];
        for i in 0..256 {
            let v = self.evaluate(i as f32 / 255.0);
            for c in 0..3 {
                ramp[c * 256 + i] = (v[c] * 65535.0).round() as u16;
            }
        }
        ramp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_produce_identity() {
        let ramp = FilterParams::default().to_ramp();
        for i in 0..256 {
            let expected = (i * 257) as u16;
            for c in 0..3 {
                assert_eq!(ramp[c * 256 + i], expected, "channel {c} index {i}");
            }
        }
    }

    #[test]
    fn gamma_matches_power_curve() {
        let p = FilterParams {
            gamma: 2.2,
            ..Default::default()
        };
        for i in [16u32, 64, 128, 200] {
            let x = i as f32 / 255.0;
            let expected = x.powf(1.0 / 2.2);
            let got = p.evaluate(x)[1];
            assert!((got - expected).abs() < 1e-5, "index {i}: {got} vs {expected}");
        }
    }

    #[test]
    fn ramp_stays_monotonic() {
        let p = FilterParams {
            gamma: 2.0,
            contrast: 1.15,
            brightness: 0.04,
            temperature: 7200,
            ..Default::default()
        };
        let ramp = p.to_ramp();
        for c in 0..3 {
            for i in 1..256 {
                assert!(
                    ramp[c * 256 + i] >= ramp[c * 256 + i - 1],
                    "channel {c} drops at {i}"
                );
            }
        }
    }

    #[test]
    fn neutral_temperature_is_unity() {
        let g = white::gains(6500);
        for v in g {
            assert!((v - 1.0).abs() < 1e-4, "{g:?}");
        }
    }

    #[test]
    fn warm_temperature_cuts_blue() {
        let g = white::gains(3000);
        assert!(g[0] > g[1] && g[1] > g[2], "{g:?}");
        assert!((g[0] - 1.0).abs() < 1e-4, "{g:?}");
    }

    #[test]
    fn cool_temperature_cuts_red() {
        let g = white::gains(9000);
        assert!(g[2] > g[1] && g[1] > g[0], "{g:?}");
    }

    #[test]
    fn curve_interpolation_is_monotonic_and_exact_at_nodes() {
        let pts = [[0.0, 0.0], [0.25, 0.18], [0.5, 0.5], [0.75, 0.84], [1.0, 1.0]];
        let mut previous = -1.0;
        for i in 0..=100 {
            let x = i as f32 / 100.0;
            let y = curve::sample(&pts, x);
            assert!(y >= previous, "not monotonic at {x}");
            previous = y;
        }
        for p in pts {
            assert!((curve::sample(&pts, p[0]) - p[1]).abs() < 1e-4, "{p:?}");
        }
    }

    #[test]
    fn shadows_lifts_dark_without_touching_black() {
        let p = FilterParams {
            shadows: 0.3,
            ..Default::default()
        };
        assert_eq!(p.evaluate(0.0)[1], 0.0);
        assert!(p.evaluate(0.25)[1] > 0.25);
        assert!((p.evaluate(1.0)[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn highlights_compresses_top_without_touching_black() {
        let p = FilterParams {
            highlights: 0.3,
            ..Default::default()
        };
        assert_eq!(p.evaluate(0.0)[1], 0.0);
        assert!(p.evaluate(0.9)[1] < 0.9);
        assert!((p.evaluate(1.0)[1] - 0.7).abs() < 1e-5);
    }

    #[test]
    fn tone_sliders_stay_monotonic_at_limits() {
        for shadows in [-TONE_RANGE, -0.15, 0.0, 0.15, TONE_RANGE] {
            for highlights in [-TONE_RANGE, 0.0, TONE_RANGE] {
                let p = FilterParams {
                    shadows,
                    highlights,
                    ..Default::default()
                };
                let ramp = p.to_ramp();
                for i in 1..256 {
                    assert!(
                        ramp[256 + i] >= ramp[256 + i - 1],
                        "shadows={shadows} highlights={highlights} 在 {i} 处回头"
                    );
                }
            }
        }
    }

    #[test]
    fn brightness_is_clamped_to_usable_range() {
        let wide = FilterParams {
            brightness: 0.5,
            ..Default::default()
        };
        let clamped = FilterParams {
            brightness: super::BRIGHTNESS_RANGE,
            ..Default::default()
        };
        assert_eq!(wide.to_ramp(), clamped.to_ramp());
    }

    #[test]
    fn levels_remap_input_range() {
        let p = FilterParams {
            levels: Levels {
                in_black: 0.1,
                in_white: 0.9,
                out_black: 0.0,
                out_white: 1.0,
            },
            ..Default::default()
        };
        assert!(p.evaluate(0.05)[1] == 0.0);
        assert!((p.evaluate(0.5)[1] - 0.5).abs() < 1e-5);
        assert!(p.evaluate(0.95)[1] == 1.0);
    }
}
