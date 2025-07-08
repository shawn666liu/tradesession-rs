use anyhow::anyhow;
use chrono::NaiveTime;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use tradesession::SessionManager;

fn to_pyerr(err: anyhow::Error) -> PyErr {
    PyErr::new::<PyException, _>(err.to_string())
}

#[gen_stub_pyclass]
#[pyclass]
pub struct SessionPP {
    session: tradesession::TradeSession,
}

#[gen_stub_pyclass]
#[pyclass]
pub struct SessionMgr {
    mgr: SessionManager,
}

#[gen_stub_pymethods]
#[pymethods]
impl SessionMgr {
    #[staticmethod]
    pub fn new_from_csv(csv_file_path: &str) -> PyResult<Self> {
        let mgr = SessionManager::new_from_csv(csv_file_path).map_err(to_pyerr)?;
        Ok(SessionMgr { mgr })
    }

    #[staticmethod]
    pub fn new_from_string(csv_content: &str) -> PyResult<Self> {
        let mgr = SessionManager::new_from_string(csv_content).map_err(to_pyerr)?;
        Ok(SessionMgr { mgr })
    }

    pub fn get_session(&self, product: &str) -> PyResult<SessionPP> {
        self.mgr
            .get_session(product)
            .map(|s| SessionPP { session: s.clone() })
            .ok_or_else(|| to_pyerr(anyhow!("Session for product '{}' not found", product)))
    }

    pub fn day_begin(&self, product: &str) -> PyResult<NaiveTime> {
        self.mgr
            .day_begin(product)
            .map(|tm| *tm)
            .ok_or_else(|| to_pyerr(anyhow!("Day begin for product '{}' not found", product)))
    }

    pub fn day_end(&self, product: &str) -> PyResult<NaiveTime> {
        self.mgr
            .day_end(product)
            .map(|tm| *tm)
            .ok_or_else(|| to_pyerr(anyhow!("Day end for product '{}' not found", product)))
    }

    pub fn in_session(
        &self,
        product: &str,
        ts: NaiveTime,
        include_begin: bool,
        include_end: bool,
    ) -> PyResult<bool> {
        let opt = self
            .mgr
            .in_session(product, &ts, include_begin, include_end);
        opt.ok_or_else(|| to_pyerr(anyhow!("Session for product '{}' not found", product)))
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl SessionPP {
    pub fn day_begin(&self) -> NaiveTime {
        self.session.day_begin().clone()
    }

    pub fn day_end(&self) -> NaiveTime {
        self.session.day_end().clone()
    }

    pub fn in_session(&self, ts: NaiveTime, include_begin: bool, include_end: bool) -> bool {
        self.session.in_session(&ts, include_begin, include_end)
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn tradesessionpy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SessionPP>()?;
    m.add_class::<SessionMgr>()?;
    Ok(())
}

// Define a function to gather stub information.
define_stub_info_gatherer!(stub_info);
