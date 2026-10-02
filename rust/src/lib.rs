// PyO3 bindings: expose the Rust engine as the `shardset_engine` Python module.
// Python decodes/resizes the image (Pillow) and passes raw RGBA bytes in; the
// Rust side does all the heavy, multicore optimization.

mod bitmap;
mod color;
mod core_ops;
mod model;
mod raster;
mod scanline;
mod shape;
mod shapes;
mod state;
mod util;
mod worker;

use bitmap::Bitmap;
use color::Color;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use shape::ShapeType;

/// The image-reproduction model, driven step by step from Python.
#[pyclass]
struct Model {
    inner: model::Model,
}

#[pymethods]
impl Model {
    /// Create a model from raw RGBA bytes (width*height*4). `bg` is an optional
    /// hex color; if omitted, the image's average color is used. `workers`
    /// defaults to the number of logical CPUs.
    #[new]
    #[pyo3(signature = (width, height, rgba, bg=None, workers=None))]
    fn new(
        width: i32,
        height: i32,
        rgba: Vec<u8>,
        bg: Option<String>,
        workers: Option<usize>,
    ) -> PyResult<Model> {
        let expected = (width * height * 4) as usize;
        if rgba.len() != expected {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "rgba length {} != width*height*4 ({})",
                rgba.len(),
                expected
            )));
        }
        let target = Bitmap::from_rgba(width, height, rgba);
        let background = match bg {
            Some(s) if !s.trim().is_empty() => Color::from_hex(&s),
            _ => target.average_color(),
        };
        let nw = workers.unwrap_or_else(|| {
            std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
        });
        Ok(Model {
            inner: model::Model::new(target, background, nw.max(1)),
        })
    }

    /// Add one shard (plus `repeat` extra refined shapes). `mode` is 0..8.
    /// Returns the number of candidate shapes evaluated. Releases the GIL so the
    /// Rust worker threads run on all cores and the Python UI stays responsive.
    fn step(&mut self, py: Python<'_>, mode: i32, alpha: i32, repeat: i32) -> i64 {
        let inner = &mut self.inner;
        py.allow_threads(move || inner.step(ShapeType::from_i32(mode), alpha, repeat))
    }

    /// Current reconstruction as raw RGBA bytes (width*height*4).
    fn rgba_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new_bound(py, &self.inner.current.pix)
    }

    /// Current reconstruction as an SVG document (resolution-independent).
    fn svg(&self) -> String {
        self.inner.svg()
    }

    #[getter]
    fn score(&self) -> f64 {
        self.inner.score
    }
    #[getter]
    fn shape_count(&self) -> usize {
        self.inner.shapes.len()
    }
    #[getter]
    fn width(&self) -> i32 {
        self.inner.w
    }
    #[getter]
    fn height(&self) -> i32 {
        self.inner.h
    }
    #[getter]
    fn workers(&self) -> usize {
        self.inner.num_workers
    }
    #[getter]
    fn background_hex(&self) -> String {
        let c = self.inner.background;
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    }
}

#[pymodule]
fn shardset_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Model>()?;
    m.add("__doc__", "Shardset image reproduction engine (Rust core, multicore).")?;
    Ok(())
}
