pub fn sample(points: &[[f32; 2]], x: f32) -> f32 {
    if points.is_empty() {
        return x;
    }
    if points.len() == 1 {
        return points[0][1];
    }

    let pts = normalize(points);
    if x <= pts[0][0] {
        return pts[0][1];
    }
    if x >= pts[pts.len() - 1][0] {
        return pts[pts.len() - 1][1];
    }

    let m = slopes(&pts);
    let mut i = 0;
    while i + 1 < pts.len() && x > pts[i + 1][0] {
        i += 1;
    }

    let h = pts[i + 1][0] - pts[i][0];
    if h <= 1e-6 {
        return pts[i][1];
    }
    let t = (x - pts[i][0]) / h;
    let t2 = t * t;
    let t3 = t2 * t;

    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;

    h00 * pts[i][1] + h10 * h * m[i] + h01 * pts[i + 1][1] + h11 * h * m[i + 1]
}

fn normalize(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut pts: Vec<[f32; 2]> = points
        .iter()
        .map(|p| [p[0].clamp(0.0, 1.0), p[1].clamp(0.0, 1.0)])
        .collect();
    pts.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal));

    let mut out: Vec<[f32; 2]> = Vec::with_capacity(pts.len());
    for p in pts {
        match out.last() {
            Some(last) if (p[0] - last[0]).abs() < 1e-5 => {}
            _ => out.push(p),
        }
    }
    out
}

fn slopes(pts: &[[f32; 2]]) -> Vec<f32> {
    let n = pts.len();
    let mut h = Vec::with_capacity(n - 1);
    let mut d = Vec::with_capacity(n - 1);
    for i in 0..n - 1 {
        let hi = pts[i + 1][0] - pts[i][0];
        h.push(hi);
        d.push((pts[i + 1][1] - pts[i][1]) / hi.max(1e-6));
    }

    let mut m = vec![0.0f32; n];
    m[0] = d[0];
    m[n - 1] = d[n - 2];

    for i in 1..n - 1 {
        if d[i - 1] * d[i] <= 0.0 {
            m[i] = 0.0;
        } else {
            let w1 = 2.0 * h[i] + h[i - 1];
            let w2 = h[i] + 2.0 * h[i - 1];
            m[i] = (w1 + w2) / (w1 / d[i - 1] + w2 / d[i]);
        }
    }
    m
}
