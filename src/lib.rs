mod image;
use image::{assemble, load_tile, FenrirImage, FenrirTile};
mod tiff;
mod viewer;
use tiff::{FenrirTiff, FenrirTiffWriter};

use pyo3::prelude::*;

#[pyfunction]
fn hello() -> &'static str {
    "Fenrir is alive!"
}

#[pymodule]
fn fenrir(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<FenrirImage>()?;
    m.add_class::<FenrirTile>()?;
    m.add_class::<FenrirTiff>()?;
    m.add_class::<FenrirTiffWriter>()?;
    m.add_function(wrap_pyfunction!(hello, m)?)?;
    m.add_function(wrap_pyfunction!(load_tile, m)?)?;
    m.add_function(wrap_pyfunction!(assemble, m)?)?;
    Ok(())
}
