pub mod triangle;
pub mod rectangle;
pub mod ellipse;
pub mod polygon;
pub mod quadratic;

use crate::shape::{Shape, ShapeType};
use crate::util::Rnd;
use rand::Rng;

use ellipse::{Ellipse, RotatedEllipse};
use polygon::Polygon;
use quadratic::Quadratic;
use rectangle::{Rectangle, RotatedRectangle};
use triangle::Triangle;

/// Create a random shape of the requested type. `Any` picks a type at random.
pub fn random_shape(t: ShapeType, w: i32, h: i32, rng: &mut Rnd) -> Box<dyn Shape> {
    match t {
        ShapeType::Any => {
            let pick = ShapeType::from_i32(rng.gen_range(1..=8));
            random_shape(pick, w, h, rng)
        }
        ShapeType::Triangle => Box::new(Triangle::new_random(w, h, rng)),
        ShapeType::Rectangle => Box::new(Rectangle::new_random(w, h, rng)),
        ShapeType::Ellipse => Box::new(Ellipse::new_random_ellipse(w, h, rng)),
        ShapeType::Circle => Box::new(Ellipse::new_random_circle(w, h, rng)),
        ShapeType::RotatedRectangle => Box::new(RotatedRectangle::new_random(w, h, rng)),
        ShapeType::Quadratic => Box::new(Quadratic::new_random(w, h, rng)),
        ShapeType::RotatedEllipse => Box::new(RotatedEllipse::new_random(w, h, rng)),
        ShapeType::Polygon => Box::new(Polygon::new_random(w, h, 4, false, rng)),
    }
}
