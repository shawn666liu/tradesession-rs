use anyhow::anyhow;
use chrono::NaiveTime;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use tradesession::{SessionManager, TradeSession};

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
    #[new]
    pub fn new() -> Self {
        let mgr = SessionManager::new();
        SessionMgr { mgr }
    }

    /// 创建失败时会爆出异常
    #[staticmethod]
    pub fn new_from_csv(csv_file_path: &str) -> PyResult<Self> {
        let mgr = SessionManager::new_from_csv(csv_file_path).map_err(to_pyerr)?;
        Ok(SessionMgr { mgr })
    }
    /// 创建失败时会爆出异常
    #[staticmethod]
    pub fn new_from_csv_content(csv_content: &str) -> PyResult<Self> {
        let mgr = SessionManager::new_from_csv_content(csv_content).map_err(to_pyerr)?;
        Ok(SessionMgr { mgr })
    }

    #[staticmethod]
    pub fn new_commodity_session() -> SessionPP {
        let session = TradeSession::new_commodity_session();
        SessionPP { session }
    }
    #[staticmethod]
    pub fn new_commodity_session_night() -> SessionPP {
        let session = TradeSession::new_commodity_session_night();
        SessionPP { session }
    }
    #[staticmethod]
    pub fn new_stock_session() -> SessionPP {
        let session = TradeSession::new_stock_session();
        SessionPP { session }
    }
    #[staticmethod]
    pub fn new_stock_index_session() -> SessionPP {
        let session = TradeSession::new_stock_index_session();
        SessionPP { session }
    }
    #[staticmethod]
    pub fn new_full_session() -> SessionPP {
        let session = TradeSession::new_full_session();
        SessionPP { session }
    }
    /// ag,SHFE,[{"Begin":"09:00:00","End":"10:15:00"},{"Begin":"10:30:00","End":"11:30:00"},{"Begin":"13:30:00","End":"15:00:00"},{"Begin":"21:00:00","End":"02:30:00"}]
    pub fn reload_csv_contend(&mut self, csv_content: &str, merge: bool) -> PyResult<()> {
        self.mgr
            .reload_csv_contend(csv_content, merge)
            .map_err(to_pyerr)
    }
    /// ag,SHFE,[{"Begin":"09:00:00","End":"10:15:00"},{"Begin":"10:30:00","End":"11:30:00"},{"Begin":"13:30:00","End":"15:00:00"},{"Begin":"21:00:00","End":"02:30:00"}]
    pub fn reload_csv_file(&mut self, csv_file_path: &str, merge: bool) -> PyResult<()> {
        self.mgr
            .reload_csv_file(csv_file_path, merge)
            .map_err(to_pyerr)
    }
    pub fn has_session(&self, product: &str) -> bool {
        self.mgr.has_session(product)
    }

    /// 获取失败时会爆出异常
    pub fn get_session(&self, product: &str) -> PyResult<SessionPP> {
        self.mgr
            .get_session(product)
            .map(|s| SessionPP { session: s.clone() })
            .ok_or_else(|| to_pyerr(anyhow!("Session for product '{}' not found", product)))
    }
    /// 获取失败时会爆出异常
    pub fn day_begin(&self, product: &str) -> PyResult<NaiveTime> {
        self.mgr
            .day_begin(product)
            .map(|tm| *tm)
            .ok_or_else(|| to_pyerr(anyhow!("Day begin for product '{}' not found", product)))
    }
    /// 获取失败时会爆出异常
    pub fn day_end(&self, product: &str) -> PyResult<NaiveTime> {
        self.mgr
            .day_end(product)
            .map(|tm| *tm)
            .ok_or_else(|| to_pyerr(anyhow!("Day end for product '{}' not found", product)))
    }
    /// 获取失败时会爆出异常
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

    /// start, end之间任意时间点落在session中吗?
    pub fn any_in_session(
        &self,
        product: &str,
        start: NaiveTime,
        end: NaiveTime,
        include_begin_end: bool,
    ) -> PyResult<bool> {
        let opt = self
            .mgr
            .any_in_session(product, &start, &end, include_begin_end);
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
    /// start, end之间任意时间点落在session中吗?
    pub fn any_in_session(
        &self,
        start: NaiveTime,
        end: NaiveTime,
        include_begin_end: bool,
    ) -> bool {
        self.session.any_in_session(&start, &end, include_begin_end)
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
