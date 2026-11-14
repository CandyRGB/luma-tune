fn rgb(temp: f32) -> [f32; 3] {
    let t = (temp / 100.0).clamp(10.0, 400.0);
    let r = if t <= 66.0 {
        255.0
    } else {
        329.698_73 * (t - 60.0).powf(-0.133_204_76)
    };
    let g = if t <= 66.0 {
        99.470_8 * t.ln() - 161.119_57
    } else {
        288.122_17 * (t - 60.0).powf(-0.075_514_85)
    };
    let b = if t >= 66.0 {
        255.0
    } else if t <= 19.0 {
        0.0
    } else {
        138.517_73 * (t - 10.0).ln() - 305.044_8
    };
    [
        (r / 255.0).clamp(0.0, 1.0),
        (g / 255.0).clamp(0.0, 1.0),
        (b / 255.0).clamp(0.0, 1.0),
    ]
}

pub fn gains(temp: u32) -> [f32; 3] {
    let t = temp.clamp(1000, 12000) as f32;
    let c = rgb(t);
    let reference = rgb(6500.0);

    let cm = c[0].max(c[1]).max(c[2]).max(1e-4);
    let rm = reference[0].max(reference[1]).max(reference[2]).max(1e-4);

    let mut out = [
        (c[0] / cm) / (reference[0] / rm),
        (c[1] / cm) / (reference[1] / rm),
        (c[2] / cm) / (reference[2] / rm),
    ];
    let peak = out[0].max(out[1]).max(out[2]).max(1.0);
    for v in out.iter_mut() {
        *v /= peak;
    }
    out
}
