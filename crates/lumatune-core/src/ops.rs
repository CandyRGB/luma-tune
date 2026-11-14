use crate::params::Levels;

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0).max(1e-6)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn input_levels(x: f32, l: &Levels) -> f32 {
    let span = (l.in_white - l.in_black).abs().max(1e-4);
    ((x - l.in_black) / span).clamp(0.0, 1.0)
}

pub fn output_levels(x: f32, l: &Levels) -> f32 {
    (l.out_black + (l.out_white - l.out_black) * x).clamp(0.0, 1.0)
}

pub fn brightness(x: f32, b: f32) -> f32 {
    x + b
}

pub fn contrast(x: f32, c: f32) -> f32 {
    (x - 0.5) * c + 0.5
}

pub fn shadows(x: f32, amount: f32) -> f32 {
    x + amount * x * (1.0 - smoothstep(0.0, 0.7, x))
}

pub fn highlights(x: f32, amount: f32) -> f32 {
    x - amount * smoothstep(0.4, 1.0, x)
}

pub fn gamma(x: f32, g: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    x.powf(1.0 / g.max(1e-3))
}
