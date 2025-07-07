use anyhow::{Result, anyhow};
use tradesession::SessionManager;

use tradesession::jcswitch::{time_from_midnight_nanos, time_to_midnight_nanos};

pub struct SessionPP {
    session: tradesession::TradeSession,
}
pub struct SessionMgr {
    mgr: SessionManager,
}

pub fn new_from_csv(csv_file_path: &str) -> Result<Box<SessionMgr>> {
    let mgr = SessionManager::new_from_csv(csv_file_path)?;
    Ok(Box::new(SessionMgr { mgr }))
}

pub fn new_from_string(csv_content: &str) -> Result<Box<SessionMgr>> {
    let mgr = SessionManager::new_from_string(csv_content)?;
    Ok(Box::new(SessionMgr { mgr }))
}

impl SessionMgr {
    pub fn get_session(&self, product: &str) -> Result<Box<SessionPP>> {
        match self.mgr.get_session(product) {
            Some(s) => Ok(Box::new(SessionPP { session: s.clone() })),
            None => Err(anyhow!("Session for product '{}' not found", product)),
        }
    }

    pub fn day_begin(&self, product: &str) -> Result<i64> {
        match self.mgr.day_begin(product) {
            Some(tm) => Ok(time_to_midnight_nanos(tm)),
            None => Err(anyhow!("Day begin for product '{}' not found", product)),
        }
    }

    pub fn day_end(&self, product: &str) -> Result<i64> {
        match self.mgr.day_end(product) {
            Some(tm) => Ok(time_to_midnight_nanos(tm)),
            None => Err(anyhow!("Day end for product '{}' not found", product)),
        }
    }

    pub fn in_session(
        &self,
        product: &str,
        nanos_since_midnight: i64,
        include_begin: bool,
        include_end: bool,
    ) -> Result<bool> {
        let ts = time_from_midnight_nanos(nanos_since_midnight);
        match self
            .mgr
            .in_session(product, &ts, include_begin, include_end)
        {
            Some(b) => Ok(b),
            None => Err(anyhow!("Session check for product '{}' not found", product)),
        }
    }
}

impl SessionPP {
    pub fn day_begin(&self) -> i64 {
        time_to_midnight_nanos(self.session.day_begin())
    }

    pub fn day_end(&self) -> i64 {
        time_to_midnight_nanos(self.session.day_end())
    }

    pub fn in_session(
        &self,
        nanos_since_midnight: i64,
        include_begin: bool,
        include_end: bool,
    ) -> bool {
        let ts = time_from_midnight_nanos(nanos_since_midnight);
        self.session.in_session(&ts, include_begin, include_end)
    }
}

#[cxx::bridge(namespace = "tradesessionpp")]
mod ffi {

    extern "Rust" {
        type SessionPP;
        type SessionMgr;

        fn new_from_csv(csv_file_path: &str) -> Result<Box<SessionMgr>>;
        fn new_from_string(csv_content: &str) -> Result<Box<SessionMgr>>;

        fn day_begin(self: &SessionPP) -> i64;
        fn day_end(self: &SessionPP) -> i64;
        fn in_session(
            self: &SessionPP,
            nanos_since_midnight: i64,
            include_begin: bool,
            include_end: bool,
        ) -> bool;

        fn get_session(self: &SessionMgr, product: &str) -> Result<Box<SessionPP>>;
        fn day_begin(self: &SessionMgr, product: &str) -> Result<i64>;
        fn day_end(self: &SessionMgr, product: &str) -> Result<i64>;
        fn in_session(
            self: &SessionMgr,
            product: &str,
            nanos_since_midnight: i64,
            include_begin: bool,
            include_end: bool,
        ) -> Result<bool>;
    }
}
