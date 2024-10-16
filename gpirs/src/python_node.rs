use std::collections::HashMap;

use crate::port::PortValue;
use pyo3::prelude::*;

#[pyclass]
pub struct GpiNode {
    #[pyo3(get)]
    pub inputs: Vec<PortValue>,

    #[pyo3(get)]
    pub out: PortValue,

    #[pyo3(get, set)]
    pub config: HashMap<String, String>,
}
#[pyclass]
pub enum GpiPortType {
    Integer,
    Array,
    String,
}

///// A Python module for interfacing with GPI
#[pymodule]
pub fn gpirs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GpiNode>()?;
    m.add_class::<GpiPortType>()?;
    Ok(())
}
