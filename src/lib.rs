mod image;
use image::FenrirImage;
mod viewer;

use pyo3::prelude::*;

#[pyfunction]
fn hello() -> &'static str {
    "Fenrir is alive!"
}

#[pymodule]
fn fenrir(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<FenrirImage>()?;
    m.add_function(wrap_pyfunction!(hello, m)?)?;
    Ok(())
}
