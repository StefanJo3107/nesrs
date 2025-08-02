// use pyo3::{pymodule, PyResult, Python};
// use pyo3::types::PyModule;
// use pyo3::prelude::*;
use crate::api::emulator::Emulator;

pub mod emulator;

// #[pymodule]
// fn nesrs(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
//     m.add_class::<Emulator>()?;
//     Ok(())
// }