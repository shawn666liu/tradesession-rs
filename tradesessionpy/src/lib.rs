use anyhow::anyhow;
use chrono::NaiveTime;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::collections::{BTreeSet, HashMap};

use tradesession;

fn to_pyerr(err: anyhow::Error) -> PyErr {
    PyErr::new::<PyException, _>(err.to_string())
}

#[gen_stub_pyclass]
#[pyclass]
pub struct TradeSession {
    session: tradesession::TradeSession,
}

#[gen_stub_pyclass]
#[pyclass]
pub struct SessionMgr {
    mgr: tradesession::SessionManager,
}

#[gen_stub_pymethods]
#[pymethods]
impl TradeSession {
    #[staticmethod]
    pub fn new_commodity_session() -> Self {
        let session = tradesession::TradeSession::new_commodity_session();
        Self { session }
    }
    #[staticmethod]
    pub fn new_commodity_session_night() -> Self {
        let session = tradesession::TradeSession::new_commodity_session_night();
        Self { session }
    }
    #[staticmethod]
    pub fn new_stock_session() -> Self {
        let session = tradesession::TradeSession::new_stock_session();
        Self { session }
    }
    #[staticmethod]
    pub fn new_stock_index_session() -> Self {
        let session = tradesession::TradeSession::new_stock_index_session();
        Self { session }
    }
    #[staticmethod]
    pub fn new_full_session() -> Self {
        let session = tradesession::TradeSession::new_full_session();
        Self { session }
    }

    #[new]
    pub fn new(minutes: Vec<u32>) -> PyResult<Self> {
        if minutes.is_empty() {
            let session = tradesession::TradeSession::new();
            return Ok(Self { session });
        }
        let minutes: BTreeSet<u32> = minutes.into_iter().collect();
        let session = tradesession::TradeSession::new_from_minutes(&minutes);
        Ok(Self { session })
    }

    /// 该品种日线开始时间，9:00/9:15/9:30/21:00, 一般是集合竞价所在的时间
    pub fn day_begin(&self) -> NaiveTime {
        self.session.day_begin().clone()
    }

    ///该品种日线结束时间，商品15:00，股指曾经15:15，股指现在15:00
    pub fn day_end(&self) -> NaiveTime {
        self.session.day_end().clone()
    }

    /// 该品种早盘开始时间，9:00/9:15/9:30,非夜盘品种跟day_begin相同
    pub fn morning_begin(&self) -> NaiveTime {
        self.session.morning_begin().clone()
    }

    /// 是否有夜盘交易
    pub fn has_nigth(&self) -> bool {
        self.session.has_nigth()
    }

    #[pyo3(signature = (ts, include_begin, include_end=false))]
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

    pub fn minutes_list(&self) -> Vec<u32> {
        self.session.minutes_list().iter().cloned().collect()
    }

    pub fn add_slice(
        &mut self,
        start_hour: u32,
        start_minute: u32,
        end_hour: u32,
        end_minute: u32,
    ) -> PyResult<()> {
        self.session
            .add_slice(start_hour, start_minute, end_hour, end_minute)
            .map_err(to_pyerr)
    }

    pub fn post_fix(&mut self) {
        self.session.post_fix();
    }
    #[pyo3(name = "__str__")]
    pub fn to_string(&self) -> String {
        format!("{}", self.session)
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl SessionMgr {
    #[new]
    pub fn new() -> Self {
        let mgr = tradesession::SessionManager::new();
        Self { mgr }
    }

    /// 创建失败时会爆出异常
    #[staticmethod]
    pub fn new_from_csv(csv_file_path: &str) -> PyResult<Self> {
        let mgr = tradesession::SessionManager::new_from_csv(csv_file_path).map_err(to_pyerr)?;
        Ok(Self { mgr })
    }
    /// 创建失败时会爆出异常
    #[staticmethod]
    pub fn new_from_csv_content(csv_content: &str) -> PyResult<Self> {
        let mgr =
            tradesession::SessionManager::new_from_csv_content(csv_content).map_err(to_pyerr)?;
        Ok(Self { mgr })
    }

    /// 注意sessions列,(json里面有逗号,需要多重双引号)
    /// ag,SHFE,"[{""Begin"":""09:00:00"",""End"":""10:15:00""},{""Begin"":""10:30:00"",""End"":""11:30:00""},{""Begin"":""13:30:00"",""End"":""15:00:00""},{""Begin"":""21:00:00"",""End"":""02:30:00""}]"
    pub fn reload_csv_content(&mut self, csv_content: &str, merge: bool) -> PyResult<()> {
        self.mgr
            .reload_csv_content(csv_content, merge)
            .map_err(to_pyerr)
    }
    /// 注意sessions列,(json里面有逗号,需要多重双引号)
    /// ag,SHFE,"[{""Begin"":""09:00:00"",""End"":""10:15:00""},{""Begin"":""10:30:00"",""End"":""11:30:00""},{""Begin"":""13:30:00"",""End"":""15:00:00""},{""Begin"":""21:00:00"",""End"":""02:30:00""}]"
    pub fn reload_csv_file(&mut self, csv_file_path: &str, merge: bool) -> PyResult<()> {
        self.mgr
            .reload_csv_file(csv_file_path, merge)
            .map_err(to_pyerr)
    }
    pub fn has_session(&self, product: &str) -> bool {
        self.mgr.has_session(product)
    }

    /// 获取失败时会爆出异常
    pub fn get_session(&self, product: &str) -> PyResult<TradeSession> {
        self.mgr
            .get_session(product)
            .map(|s| TradeSession { session: s.clone() })
            .ok_or_else(|| to_pyerr(anyhow!("Session for product '{}' not found", product)))
    }
    /// 获取失败时会爆出异常
    pub fn day_begin(&self, product: &str) -> PyResult<NaiveTime> {
        self.mgr
            .day_begin(product)
            .map(|tm| *tm)
            .ok_or_else(|| to_pyerr(anyhow!("day begin for product '{}' not found", product)))
    }
    /// 获取失败时会爆出异常
    pub fn day_end(&self, product: &str) -> PyResult<NaiveTime> {
        self.mgr
            .day_end(product)
            .map(|tm| *tm)
            .ok_or_else(|| to_pyerr(anyhow!("day end for product '{}' not found", product)))
    }
    /// 获取失败时会爆出异常
    pub fn morning_begin(&self, product: &str) -> PyResult<NaiveTime> {
        self.mgr
            .morning_begin(product)
            .map(|tm| *tm)
            .ok_or_else(|| to_pyerr(anyhow!("morning_begin for product '{}' not found", product)))
    }
    /// 获取失败时会爆出异常
    #[pyo3(signature = (product, ts, include_begin, include_end=false))]
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
    #[getter]
    pub fn sessions_count(&self) -> usize {
        self.mgr.session_map().len()
    }
    pub fn session_map(&self) -> PyResult<HashMap<String, TradeSession>> {
        Ok(self
            .mgr
            .session_map()
            .iter()
            .map(|(k, v)| (k.clone(), TradeSession { session: v.clone() }))
            .collect())
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn tradesessionpy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<TradeSession>()?;
    m.add_class::<SessionMgr>()?;
    Ok(())
}

// Define a function to gather stub information.
define_stub_info_gatherer!(stub_info);
