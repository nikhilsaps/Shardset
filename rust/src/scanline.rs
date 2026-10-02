// A horizontal run of pixels on row Y from X1..=X2 with 16-bit coverage alpha.

#[derive(Clone, Copy, Debug)]
pub struct Scanline {
    pub y: i32,
    pub x1: i32,
    pub x2: i32,
    pub alpha: u32,
}

/// Clip scanlines to the image bounds, dropping fully-offscreen ones.
pub fn crop_scanlines(mut lines: Vec<Scanline>, w: i32, h: i32) -> Vec<Scanline> {
    let mut i = 0usize;
    for k in 0..lines.len() {
        let mut line = lines[k];
        if line.y < 0 || line.y >= h { continue; }
        if line.x1 >= w { continue; }
        if line.x2 < 0 { continue; }
        line.x1 = line.x1.clamp(0, w - 1);
        line.x2 = line.x2.clamp(0, w - 1);
        if line.x1 > line.x2 { continue; }
        lines[i] = line;
        i += 1;
    }
    lines.truncate(i);
    lines
}
