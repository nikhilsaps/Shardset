// Color: non-premultiplied RGBA with integer channels, matching the Go `Color` struct.

#[derive(Clone, Copy, Debug, Default)]
pub struct Color {
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub a: i32,
}

impl Color {
    pub fn new(r: i32, g: i32, b: i32, a: i32) -> Self {
        Color { r, g, b, a }
    }

    /// Parse a hex color like "#rgb", "#rgba", "#rrggbb", "#rrggbbaa".
    pub fn from_hex(x: &str) -> Color {
        let x = x.trim().trim_start_matches('#');
        let (mut r, mut g, mut b, mut a) = (0i32, 0i32, 0i32, 255i32);
        let parse = |s: &str| i32::from_str_radix(s, 16).unwrap_or(0);
        match x.len() {
            3 => {
                r = parse(&x[0..1]); g = parse(&x[1..2]); b = parse(&x[2..3]);
                r = (r << 4) | r; g = (g << 4) | g; b = (b << 4) | b;
            }
            4 => {
                r = parse(&x[0..1]); g = parse(&x[1..2]); b = parse(&x[2..3]); a = parse(&x[3..4]);
                r = (r << 4) | r; g = (g << 4) | g; b = (b << 4) | b; a = (a << 4) | a;
            }
            6 => { r = parse(&x[0..2]); g = parse(&x[2..4]); b = parse(&x[4..6]); }
            8 => { r = parse(&x[0..2]); g = parse(&x[2..4]); b = parse(&x[4..6]); a = parse(&x[6..8]); }
            _ => {}
        }
        Color { r, g, b, a }
    }

    /// Equivalent of Go's color.NRGBA{r,g,b,a}.RGBA(): returns 16-bit premultiplied channels.
    #[inline]
    pub fn premultiplied(&self) -> (u32, u32, u32, u32) {
        let (cr, cg, cb, ca) = (self.r as u32, self.g as u32, self.b as u32, self.a as u32);
        let mut r = cr; r |= r << 8; r *= ca; r /= 0xff;
        let mut g = cg; g |= g << 8; g *= ca; g /= 0xff;
        let mut b = cb; b |= b << 8; b *= ca; b /= 0xff;
        let mut a = ca; a |= a << 8;
        (r, g, b, a)
    }
}
