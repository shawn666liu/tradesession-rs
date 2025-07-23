use anyhow::{Result, anyhow};
use tradesession::{SessionManager, TradeSession};

use tradesession::jcswitch::{time_from_midnight_nanos, time_to_midnight_nanos};

pub struct SessionPP {
    session: tradesession::TradeSession,
}
pub struct SessionMgr {
    mgr: SessionManager,
}

pub fn new_session() -> Box<SessionPP> {
    Box::new(SessionPP {
        session: TradeSession::new(),
    })
}

pub fn new_from_minutes(minutes: Vec<u16>) -> Box<SessionPP> {
    let session = TradeSession::new_from_minutes(minutes);
    Box::new(SessionPP { session })
}

pub fn new_mgr() -> Box<SessionMgr> {
    Box::new(SessionMgr {
        mgr: SessionManager::new(),
    })
}

pub fn new_from_csv(csv_file_path: &str) -> Result<Box<SessionMgr>> {
    let mgr = SessionManager::new_from_csv(csv_file_path)?;
    Ok(Box::new(SessionMgr { mgr }))
}

pub fn new_from_csv_content(csv_content: &str) -> Result<Box<SessionMgr>> {
    let mgr = SessionManager::new_from_csv_content(csv_content)?;
    Ok(Box::new(SessionMgr { mgr }))
}

pub fn new_commodity_session() -> Box<SessionPP> {
    let session = TradeSession::new_commodity_session();
    Box::new(SessionPP { session })
}

pub fn new_commodity_session_night() -> Box<SessionPP> {
    let session = TradeSession::new_commodity_session_night();
    Box::new(SessionPP { session })
}

pub fn new_stock_session() -> Box<SessionPP> {
    let session = TradeSession::new_stock_session();
    Box::new(SessionPP { session })
}

pub fn new_stock_index_session() -> Box<SessionPP> {
    let session = TradeSession::new_stock_index_session();
    Box::new(SessionPP { session })
}

pub fn new_bond_session() -> Box<SessionPP> {
    let session = TradeSession::new_bond_session();
    Box::new(SessionPP { session })
}

pub fn new_full_session() -> Box<SessionPP> {
    let session = TradeSession::new_full_session();
    Box::new(SessionPP { session })
}

impl SessionPP {
    pub fn day_begin(&self) -> i64 {
        time_to_midnight_nanos(self.session.day_begin())
    }
    pub fn day_end(&self) -> i64 {
        time_to_midnight_nanos(self.session.day_end())
    }
    pub fn morning_begin(&self) -> i64 {
        time_to_midnight_nanos(self.session.morning_begin())
    }
    pub fn has_nigth(&self) -> bool {
        self.session.has_nigth()
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

    pub fn any_in_session(
        &self,
        nanos_since_midnight_start: i64,
        nanos_since_midnight_end: i64,
        include_begin_end: bool,
    ) -> bool {
        let start = time_from_midnight_nanos(nanos_since_midnight_start);
        let end = time_from_midnight_nanos(nanos_since_midnight_end);
        self.session.any_in_session(&start, &end, include_begin_end)
    }
    pub fn minutes_list(&self) -> Vec<u16> {
        self.session.minutes_list().iter().map(|tm| *tm).collect()
    }
    pub fn to_string(&self) -> String {
        format!("{}", self.session)
    }
    pub fn add_slice(
        &mut self,
        start_hour: u32,
        start_minute: u32,
        end_hour: u32,
        end_minute: u32,
    ) -> Result<()> {
        self.session
            .add_slice(start_hour, start_minute, end_hour, end_minute)
    }
    pub fn post_fix(&mut self) {
        self.session.post_fix();
    }
}

impl SessionMgr {
    pub fn reload_csv_content(&mut self, csv_content: &str, merge: bool) -> Result<()> {
        self.mgr.reload_csv_content(csv_content, merge)
    }
    pub fn reload_csv_file(&mut self, csv_file_path: &str, merge: bool) -> Result<()> {
        self.mgr.reload_csv_file(csv_file_path, merge)
    }

    pub fn has_session(&self, product: &str) -> bool {
        self.mgr.has_session(product)
    }

    pub fn get_session(&self, product: &str) -> Result<Box<SessionPP>> {
        self.mgr
            .get_session(product)
            .map(|s| Box::new(SessionPP { session: s.clone() }))
            .ok_or_else(|| anyhow!("Session for product '{}' not found", product))
    }

    pub fn day_begin(&self, product: &str) -> Result<i64> {
        self.mgr
            .day_begin(product)
            .map(|tm| time_to_midnight_nanos(tm))
            .ok_or_else(|| anyhow!("Day begin for product '{}' not found", product))
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

    pub fn any_in_session(
        &self,
        product: &str,
        nanos_since_midnight_start: i64,
        nanos_since_midnight_end: i64,
        include_begin_end: bool,
    ) -> Result<bool> {
        let start = time_from_midnight_nanos(nanos_since_midnight_start);
        let end = time_from_midnight_nanos(nanos_since_midnight_end);
        match self
            .mgr
            .any_in_session(product, &start, &end, include_begin_end)
        {
            Some(b) => Ok(b),
            None => Err(anyhow!("Session check for product '{}' not found", product)),
        }
    }
    pub fn sessions_count(&self) -> usize {
        self.mgr.session_map().len()
    }
    pub fn session_map_keys(&self) -> Vec<String> {
        self.mgr
            .session_map()
            .iter()
            .map(|(k, _)| k.clone())
            .collect()
    }
}

#[cxx::bridge(namespace = "sessionpp")]
mod ffi {
    extern "Rust" {
        type SessionPP;
        type SessionMgr;

        /// c++, new_session(),创建空session
        fn new_session() -> Box<SessionPP>;
        fn new_from_minutes(minutes: Vec<u16>) -> Box<SessionPP>;
        fn new_mgr() -> Box<SessionMgr>;
        /// 创建失败时会爆出异常
        fn new_from_csv(csv_file_path: &str) -> Result<Box<SessionMgr>>;
        /// 创建失败时会爆出异常
        fn new_from_csv_content(csv_content: &str) -> Result<Box<SessionMgr>>;
        /// 生成一个常规的商品期货（不含金融期货）交易时段(不含夜盘)
        fn new_commodity_session() -> Box<SessionPP>;
        /// 生成一个常规的商品期货（不含金融期货）交易时段(含夜盘)
        fn new_commodity_session_night() -> Box<SessionPP>;
        fn new_stock_session() -> Box<SessionPP>;
        fn new_stock_index_session() -> Box<SessionPP>;
        /// 生成一个国债期货的交易时段, 比金融期货多15分钟
        fn new_bond_session() -> Box<SessionPP>;
        /// 生成一个涵盖商品股指国债股票等的全部交易时段(含夜盘)
        fn new_full_session() -> Box<SessionPP>;

        //////////////////////////////////////////////////////////////////////

        /// 该品种日线开始时间，9:00/9:15/9:30/21:00, 一般是集合竞价所在的时间
        fn day_begin(self: &SessionPP) -> i64;
        ///该品种日线结束时间，商品15:00，股指曾经15:15，股指现在15:00
        fn day_end(self: &SessionPP) -> i64;
        /// 该品种早盘开始时间，9:00/9:15/9:30,非夜盘品种跟day_begin相同
        fn morning_begin(self: &SessionPP) -> i64;
        /// 是否有夜盘交易
        pub fn has_nigth(self: &SessionPP) -> bool;
        /// 获取此时间片对应分钟(u32)的数组，含开始，不含结束
        /// 注意：所有数值超前4小时
        /// 应用场景1：校验所有add_slice，自动移除重迭，自动排序，参看post_fix
        /// 应用场景2：比如仅交易了5个品种，要检查这些品种开市时间段有行情，用以求这些Session的并集
        fn minutes_list(self: &SessionPP) -> Vec<u16>;
        fn to_string(self: &SessionPP) -> String;
        /// 某个时间点落在session中吗?
        fn in_session(
            self: &SessionPP,
            nanos_since_midnight: i64,
            include_begin: bool,
            include_end: bool,
        ) -> bool;
        /// start,end之间任意时间点落在session中吗?
        fn any_in_session(
            self: &SessionPP,
            nanos_since_midnight_start: i64,
            nanos_since_midnight_end: i64,
            include_begin_end: bool,
        ) -> bool;
        fn add_slice(
            self: &mut SessionPP,
            start_hour: u32,
            start_minute: u32,
            end_hour: u32,
            end_minute: u32,
        ) -> Result<()>;
        fn post_fix(self: &mut SessionPP);

        /////////////////////////////////////////////////////

        /// csv文件是直接从数据库表导出的,一共三列, product,exchange,sessions
        /// 注意sessions列,(json里面有逗号,需要多重双引号)
        /// ag,SHFE,"[{""Begin"":""09:00:00"",""End"":""10:15:00""},{""Begin"":""10:30:00"",""End"":""11:30:00""},{""Begin"":""13:30:00"",""End"":""15:00:00""},{""Begin"":""21:00:00"",""End"":""02:30:00""}]"
        /// 如果csv文件只有两列, 则第一列为产品名, 第二列为json字符串
        /// 如果csv文件有三列, 则第一列为产品名, 第二列为交易所名, 第三列为json字符串
        fn reload_csv_content(self: &mut SessionMgr, csv_content: &str, merge: bool) -> Result<()>;
        /// 注意sessions列,(json里面有逗号,需要多重双引号)
        /// ag,SHFE,"[{""Begin"":""09:00:00"",""End"":""10:15:00""},{""Begin"":""10:30:00"",""End"":""11:30:00""},{""Begin"":""13:30:00"",""End"":""15:00:00""},{""Begin"":""21:00:00"",""End"":""02:30:00""}]"
        fn reload_csv_file(self: &mut SessionMgr, csv_file_path: &str, merge: bool) -> Result<()>;
        fn has_session(self: &SessionMgr, product: &str) -> bool;
        /// 获取失败时会爆出异常
        fn get_session(self: &SessionMgr, product: &str) -> Result<Box<SessionPP>>;
        /// 获取失败时会爆出异常
        fn day_begin(self: &SessionMgr, product: &str) -> Result<i64>;
        /// 获取失败时会爆出异常
        fn day_end(self: &SessionMgr, product: &str) -> Result<i64>;
        /// 获取失败时会爆出异常
        fn in_session(
            self: &SessionMgr,
            product: &str,
            nanos_since_midnight: i64,
            include_begin: bool,
            include_end: bool,
        ) -> Result<bool>;
        /// 获取失败时会爆出异常
        fn any_in_session(
            self: &SessionMgr,
            product: &str,
            nanos_since_midnight_start: i64,
            nanos_since_midnight_end: i64,
            include_begin_end: bool,
        ) -> Result<bool>;
        fn sessions_count(self: &SessionMgr) -> usize;
        /// cxx crate 目前不支持返回字典，所以只返回keys
        fn session_map_keys(self: &SessionMgr) -> Vec<String>;
    }
}
