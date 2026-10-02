// Pixel-level operations: optimal color, span blending, and RMSE scoring.
// Direct ports of primitive/core.go.

use crate::bitmap::Bitmap;
use crate::color::Color;
use crate::scanline::Scanline;

/// Compute the optimal average color for a shape covering `lines`, given alpha.
pub fn compute_color(target: &Bitmap, current: &Bitmap, lines: &[Scanline], alpha: i32) -> Color {
    let (mut rsum, mut gsum, mut bsum, mut count): (i64, i64, i64, i64) = (0, 0, 0, 0);
    let a = 0x101i64 * 255 / alpha as i64;
    for line in lines {
        let mut i = target.offset(line.x1, line.y);
        for _x in line.x1..=line.x2 {
            let tr = target.pix[i] as i64;
            let tg = target.pix[i + 1] as i64;
            let tb = target.pix[i + 2] as i64;
            let cr = current.pix[i] as i64;
            let cg = current.pix[i + 1] as i64;
            let cb = current.pix[i + 2] as i64;
            i += 4;
            rsum += (tr - cr) * a + cr * 0x101;
            gsum += (tg - cg) * a + cg * 0x101;
            bsum += (tb - cb) * a + cb * 0x101;
            count += 1;
        }
    }
    if count == 0 {
        return Color::default();
    }
    let r = ((rsum / count) >> 8).clamp(0, 255) as i32;
    let g = ((gsum / count) >> 8).clamp(0, 255) as i32;
    let b = ((bsum / count) >> 8).clamp(0, 255) as i32;
    Color::new(r, g, b, alpha)
}

/// Copy the spans covered by `lines` from src into dst.
pub fn copy_lines(dst: &mut Bitmap, src: &Bitmap, lines: &[Scanline]) {
    for line in lines {
        let a = dst.offset(line.x1, line.y);
        let b = a + ((line.x2 - line.x1 + 1) * 4) as usize;
        dst.pix[a..b].copy_from_slice(&src.pix[a..b]);
    }
}

/// Alpha-blend color c onto `im` over the spans in `lines`.
pub fn draw_lines(im: &mut Bitmap, c: Color, lines: &[Scanline]) {
    const M: u32 = 0xffff;
    let (sr, sg, sb, sa) = c.premultiplied();
    for line in lines {
        let ma = line.alpha;
        let a = (M - sa * ma / M) * 0x101;
        let mut i = im.offset(line.x1, line.y);
        for _x in line.x1..=line.x2 {
            let dr = im.pix[i] as u32;
            let dg = im.pix[i + 1] as u32;
            let db = im.pix[i + 2] as u32;
            let da = im.pix[i + 3] as u32;
            im.pix[i] = (((dr * a + sr * ma) / M) >> 8) as u8;
            im.pix[i + 1] = (((dg * a + sg * ma) / M) >> 8) as u8;
            im.pix[i + 2] = (((db * a + sb * ma) / M) >> 8) as u8;
            im.pix[i + 3] = (((da * a + sa * ma) / M) >> 8) as u8;
            i += 4;
        }
    }
}

/// Full RMSE difference between two images, normalized to 0..1.
pub fn difference_full(a: &Bitmap, b: &Bitmap) -> f64 {
    let (w, h) = (a.w, a.h);
    let mut total: u64 = 0;
    for y in 0..h {
        let mut i = a.offset(0, y);
        for _x in 0..w {
            let dr = a.pix[i] as i64 - b.pix[i] as i64;
            let dg = a.pix[i + 1] as i64 - b.pix[i + 1] as i64;
            let db = a.pix[i + 2] as i64 - b.pix[i + 2] as i64;
            let da = a.pix[i + 3] as i64 - b.pix[i + 3] as i64;
            i += 4;
            total += (dr * dr + dg * dg + db * db + da * da) as u64;
        }
    }
    (total as f64 / (w * h * 4) as f64).sqrt() / 255.0
}

/// Incremental RMSE: adjust `score` for only the pixels touched by `lines`.
pub fn difference_partial(
    target: &Bitmap,
    before: &Bitmap,
    after: &Bitmap,
    score: f64,
    lines: &[Scanline],
) -> f64 {
    let (w, h) = (target.w, target.h);
    let mut total: i64 = ((score * 255.0).powi(2) * (w * h * 4) as f64) as i64;
    for line in lines {
        let mut i = target.offset(line.x1, line.y);
        for _x in line.x1..=line.x2 {
            let tr = target.pix[i] as i64;
            let tg = target.pix[i + 1] as i64;
            let tb = target.pix[i + 2] as i64;
            let ta = target.pix[i + 3] as i64;
            let br = before.pix[i] as i64;
            let bg = before.pix[i + 1] as i64;
            let bb = before.pix[i + 2] as i64;
            let ba = before.pix[i + 3] as i64;
            let ar = after.pix[i] as i64;
            let ag = after.pix[i + 1] as i64;
            let ab = after.pix[i + 2] as i64;
            let aa = after.pix[i + 3] as i64;
            i += 4;
            let (dr1, dg1, db1, da1) = (tr - br, tg - bg, tb - bb, ta - ba);
            let (dr2, dg2, db2, da2) = (tr - ar, tg - ag, tb - ab, ta - aa);
            total -= dr1 * dr1 + dg1 * dg1 + db1 * db1 + da1 * da1;
            total += dr2 * dr2 + dg2 * dg2 + db2 * db2 + da2 * da2;
        }
    }
    if total < 0 { total = 0; }
    (total as f64 / (w * h * 4) as f64).sqrt() / 255.0
}
