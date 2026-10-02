// The Shape abstraction: every primitive can rasterize itself to spans,
// mutate randomly (for hill climbing), clone, and emit SVG.

use crate::scanline::Scanline;
use crate::util::Rnd;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeType {
    Any = 0,
    Triangle = 1,
    Rectangle = 2,
    Ellipse = 3,
    Circle = 4,
    RotatedRectangle = 5,
    Quadratic = 6,
    RotatedEllipse = 7,
    Polygon = 8,
}

impl ShapeType {
    pub fn from_i32(v: i32) -> ShapeType {
        match v {
            1 => ShapeType::Triangle,
            2 => ShapeType::Rectangle,
            3 => ShapeType::Ellipse,
            4 => ShapeType::Circle,
            5 => ShapeType::RotatedRectangle,
            6 => ShapeType::Quadratic,
            7 => ShapeType::RotatedEllipse,
            8 => ShapeType::Polygon,
            _ => ShapeType::Any,
        }
    }
}

pub trait Shape: Send {
    fn rasterize(&self, w: i32, h: i32) -> Vec<Scanline>;
    fn mutate(&mut self, w: i32, h: i32, rng: &mut Rnd);
    fn clone_box(&self) -> Box<dyn Shape>;
    fn svg(&self, attrs: &str) -> String;
}

impl Clone for Box<dyn Shape> {
    fn clone(&self) -> Box<dyn Shape> {
        self.clone_box()
    }
}
