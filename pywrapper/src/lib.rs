use pyo3::prelude::*;
use rust_ethernet_ip::{EipClient, PlcValue, RoutePath, UdtData};
use tokio::runtime::Runtime;

#[pyclass]
pub struct PyEipClient {
    client: Option<EipClient>,
    rt: Runtime,
}

#[pymethods]
impl PyEipClient {
    #[new]
    pub fn new() -> Self {
        Self {
            client: None,
            rt: Runtime::new().unwrap(),
        }
    }

    pub fn connect(&mut self, address: &str) -> PyResult<bool> {
        let client = self.rt.block_on(EipClient::new(address));
        match client {
            Ok(c) => {
                self.client = Some(c);
                Ok(true)
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    pub fn connect_with_route(
        &mut self,
        address: &str,
        route_path: &PyRoutePath,
    ) -> PyResult<bool> {
        let route = route_path.inner.clone();
        let client = self.rt.block_on(EipClient::with_route_path(address, route));
        match client {
            Ok(c) => {
                self.client = Some(c);
                Ok(true)
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    pub fn set_route_path(&mut self, route_path: &PyRoutePath) -> PyResult<()> {
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Not connected"))?;
        let route = route_path.inner.clone();
        client.set_route_path(route);
        Ok(())
    }

    pub fn read_dint(&mut self, tag: &str) -> PyResult<i32> {
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Not connected"))?;
        let value = self.rt.block_on(client.read_tag(tag));
        match value {
            Ok(PlcValue::Dint(v)) => Ok(v),
            Ok(_) => Err(pyo3::exceptions::PyTypeError::new_err("Tag is not a DINT")),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    pub fn write_dint(&mut self, tag: &str, value: i32) -> PyResult<()> {
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Not connected"))?;
        let result = self
            .rt
            .block_on(client.write_tag(tag, PlcValue::Dint(value)));
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    pub fn read_udt_data(&mut self, tag: &str) -> PyResult<PyUdtData> {
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Not connected"))?;
        let value = self.rt.block_on(client.read_tag(tag));
        match value {
            Ok(PlcValue::Udt(udt_data)) => Ok(PyUdtData { inner: udt_data }),
            Ok(_) => Err(pyo3::exceptions::PyTypeError::new_err("Tag is not a UDT")),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    pub fn write_udt_data(&mut self, tag: &str, udt_data: &PyUdtData) -> PyResult<()> {
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Not connected"))?;
        let result = self
            .rt
            .block_on(client.write_tag(tag, PlcValue::Udt(udt_data.inner.clone())));
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyRoutePath {
    inner: RoutePath,
}

#[pymethods]
impl PyRoutePath {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: RoutePath::new(),
        }
    }

    pub fn add_slot(&mut self, slot: u8) -> PyResult<()> {
        self.inner = self.inner.clone().add_slot(slot);
        Ok(())
    }

    pub fn add_port(&mut self, port: u8) -> PyResult<()> {
        self.inner = self.inner.clone().add_port(port);
        Ok(())
    }

    pub fn add_address(&mut self, address: String) -> PyResult<()> {
        self.inner = self.inner.clone().add_address(address);
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.inner.slots.is_empty()
            && self.inner.ports.is_empty()
            && self.inner.addresses.is_empty()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyUdtData {
    inner: UdtData,
}

#[pymethods]
impl PyUdtData {
    #[new]
    pub fn new(symbol_id: i32, data: Vec<u8>) -> Self {
        Self {
            inner: UdtData { symbol_id, data },
        }
    }

    #[getter]
    pub fn symbol_id(&self) -> i32 {
        self.inner.symbol_id
    }

    #[setter]
    pub fn set_symbol_id(&mut self, symbol_id: i32) {
        self.inner.symbol_id = symbol_id;
    }

    #[getter]
    pub fn data(&self) -> Vec<u8> {
        self.inner.data.clone()
    }

    #[setter]
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.inner.data = data;
    }
}

#[pymodule]
fn _rust_ethernet_ip(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEipClient>()?;
    m.add_class::<PyRoutePath>()?;
    m.add_class::<PyUdtData>()?;
    Ok(())
}
