// Generic scanline polygon fill (non-zero winding) and curve flattening.
// Produces disjoint, left-to-right spans per row -- the invariant the scoring
// code relies on (no pixel counted twice).

use crate::scanline::Scanline;

/// Fill an arbitrary (possibly concave / self-intersecting) polygon given as a
/// closed ring of float points. Uses non-zero winding and samples pixel centers.
pub fn fill_polygon(pts: &[(f64, f64)], w: i32, h: i32) -> Vec<Scanline> {
    let mut lines = Vec::new();
    let n = pts.len();
    if n < 3 {
        return lines;
    }
    // Vertical extent.
    let mut ymin = f64::INFINITY;
    let mut ymax = f64::NEG_INFINITY;
    for &(_, py) in pts {
        if py < ymin { ymin = py; }
        if py > ymax { ymax = py; }
    }
    let y0 = (ymin.floor() as i32).max(0);
    let y1 = (ymax.ceil() as i32).min(h - 1);

    let mut xs: Vec<(f64, i32)> = Vec::with_capacity(8);
    for y in y0..=y1 {
        let yc = y as f64 + 0.5;
        xs.clear();
        for i in 0..n {
            let (ax, ay) = pts[i];
            let (bx, by) = pts[(i + 1) % n];
            // Does edge cross the horizontal line y = yc?
            let crosses = (ay <= yc && by > yc) || (by <= yc && ay > yc);
            if !crosses {
                continue;
            }
            let t = (yc - ay) / (by - ay);
            let x = ax + t * (bx - ax);
            let dir = if by > ay { 1 } else { -1 };
            xs.push((x, dir));
        }
        if xs.len() < 2 {
            continue;
        }
        xs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut wind = 0i32;
        let mut start = 0.0f64;
        for &(x, dir) in &xs {
            let prev = wind;
            wind += dir;
            if prev == 0 && wind != 0 {
                start = x;
            } else if prev != 0 && wind == 0 {
                let x1 = (start - 0.5).ceil() as i32;
                let x2 = (x - 0.5).floor() as i32;
                let x1 = x1.max(0);
                let x2 = x2.min(w - 1);
                if x2 >= x1 {
                    lines.push(Scanline { y, x1, x2, alpha: 0xffff });
                }
            }
        }
    }
    lines
}

/// Flatten a quadratic Bezier into `n` sample points (inclusive of both ends).
pub fn flatten_quadratic(
    x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, n: usize,
) -> Vec<(f64, f64)> {
    let mut pts = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let mt = 1.0 - t;
        let x = mt * mt * x1 + 2.0 * mt * t * x2 + t * t * x3;
        let y = mt * mt * y1 + 2.0 * mt * t * y2 + t * t * y3;
        pts.push((x, y));
    }
    pts
}

/// Build a closed stroke-outline polygon around a polyline of `pts` with the
/// given stroke width (round-ish via per-vertex averaged normals).
pub fn stroke_outline(pts: &[(f64, f64)], width: f64) -> Vec<(f64, f64)> {
    let n = pts.len();
    if n < 2 {
        return Vec::new();
    }
    let hw = (width / 2.0).max(0.5);
    // Per-vertex normal = average of adjacent segment normals.
    let mut normals = Vec::with_capacity(n);
    for i in 0..n {
        let (px, py) = if i == 0 { pts[0] } else { pts[i - 1] };
        let (qx, qy) = if i == n - 1 { pts[n - 1] } else { pts[i + 1] };
        let dx = qx - px;
        let dy = qy - py;
        let len = (dx * dx + dy * dy).sqrt().max(1e-9);
        // Left normal of direction (dx,dy) is (-dy, dx).
        normals.push((-dy / len, dx / len));
    }
    let mut left = Vec::with_capacity(n);
    let mut right = Vec::with_capacity(n);
    for i in 0..n {
        let (x, y) = pts[i];
        let (nx, ny) = normals[i];
        left.push((x + nx * hw, y + ny * hw));
        right.push((x - nx * hw, y - ny * hw));
    }
    // Outline: forward along left, backward along right.
    let mut outline = Vec::with_capacity(n * 2);
    outline.extend_from_slice(&left);
    for i in (0..n).rev() {
        outline.push(right[i]);
    }
    outline
}
