use anyhow::{Context, Result, anyhow};
use encoding_rs_io::DecodeReaderBytes;
use std::fs::File;
use std::path::Path;
use std::{collections::HashMap, io::Read};

use crate::jcswitch::MyTimeType;

use super::tradesession::*;

/// 如果csv文件只有两列, 则第一列为产品名, 第二列为json字符串
/// 如果csv文件有三列, 则第一列为产品名, 第二列为交易所名, 第三列为json字符串
pub fn load_from_read<R: Read>(read: R) -> Result<HashMap<String, TradeSession>> {
    let mut hash = HashMap::new();
    let mut rdr = csv::Reader::from_reader(read);

    for result in rdr.records() {
        let record = result?;

        let (key_idx, json_idx) = match record.len() {
            2 => (0, 1),
            3 => (0, 2),
            _ => {
                return Err(anyhow!(
                    "bad format session: expected 2 or 3 fields, got {}, {:#?}",
                    record.len(),
                    record
                ));
            }
        };

        let json = &record[json_idx];
        let slices = parse_json_slices(json)?;
        let session = TradeSession::new_from_slices(&slices);

        hash.insert(record[key_idx].to_string(), session);
    }

    Ok(hash)
}

/// csv文件是直接从数据库表导出的,一共三列, product,exchange,sessions
/// 注意sessions列,(json里面有逗号,需要多重双引号)
/// ag,SHFE,"[{""Begin"":""09:00:00"",""End"":""10:15:00""},{""Begin"":""10:30:00"",""End"":""11:30:00""},{""Begin"":""13:30:00"",""End"":""15:00:00""},{""Begin"":""21:00:00"",""End"":""02:30:00""}]"
/// 如果csv文件只有两列, 则第一列为产品名, 第二列为json字符串
/// 如果csv文件有三列, 则第一列为产品名, 第二列为交易所名, 第三列为json字符串
pub fn load_from_csv<P: AsRef<Path>>(csv_file_path: P) -> Result<HashMap<String, TradeSession>> {
    let path = csv_file_path.as_ref();
    if !path.exists() {
        return Err(anyhow!("file not found `{}`", path.to_string_lossy()));
    }
    let file = File::open(path).with_context(|| path.display().to_string())?;
    return load_from_read(DecodeReaderBytes::new(file));
}

///从csv文件内容加载，参数为csv文件字符串
pub fn load_from_csv_content(csv_content: &str) -> Result<HashMap<String, TradeSession>> {
    return load_from_read(csv_content.as_bytes());
}

/// product vs json_session, when these two columns loaded from database
pub fn load_from_json_map(
    prd_vs_json: &HashMap<String, String>,
) -> Result<HashMap<String, TradeSession>> {
    let mut res_map: HashMap<String, TradeSession> = HashMap::new();
    for (k, v) in prd_vs_json {
        let res_vec: Vec<SessionSlice> = parse_json_slices(v)?;
        let session = TradeSession::new_from_slices(&res_vec);
        res_map.insert(k.to_string(), session);
    }
    Ok(res_map)
}

pub struct SessionManager {
    sessions: HashMap<String, TradeSession>,
}
impl SessionManager {
    /// 静态函数,生成一个股票交易时段
    pub fn new_stock_session() -> TradeSession {
        TradeSession::new_stock_session()
    }

    /// 静态函数,生成一个常规的股指期货交易时段
    pub fn new_stock_index_session() -> TradeSession {
        TradeSession::new_stock_index_session()
    }

    /// 静态函数,生成一个常规的商品期货交易时段(无夜盘)
    pub fn new_commodity_session() -> TradeSession {
        TradeSession::new_commodity_session()
    }
    /// 静态函数,生成一个常规的商品期货（不含金融期货）交易时段(含夜盘)
    pub fn new_commodity_session_night() -> TradeSession {
        TradeSession::new_commodity_session_night()
    }

    ////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
    pub fn new_from_map(session_map: HashMap<String, TradeSession>) -> Self {
        Self {
            sessions: session_map,
        }
    }
    /// csv file path
    pub fn new_from_csv<P: AsRef<Path>>(csv_file: P) -> Result<Self> {
        let sessions = load_from_csv(csv_file)?;
        Ok(Self { sessions })
    }
    pub fn new_from_csv_content(csv_content: &str) -> Result<Self> {
        let sessions = load_from_csv_content(csv_content)?;
        Ok(Self { sessions })
    }
    pub fn new_from_read<R: Read>(read: R) -> Result<Self> {
        let sessions = load_from_read(read)?;
        Ok(Self { sessions })
    }
    /// product vs json_session, when these two columns loaded from database
    pub fn new_from_json_map(prd_vs_json: &HashMap<String, String>) -> Result<Self> {
        let sessions = load_from_json_map(prd_vs_json)?;
        Ok(Self { sessions })
    }

    pub fn reload_csv_content(&mut self, csv_content: &str, merge: bool) -> Result<()> {
        let sessions = load_from_csv_content(csv_content)?;
        if merge {
            self.sessions.extend(sessions);
        } else {
            self.sessions = sessions;
        }
        Ok(())
    }
    pub fn reload_csv_file<P: AsRef<Path>>(&mut self, csv_file_path: P, merge: bool) -> Result<()> {
        let sessions = load_from_csv(csv_file_path)?;
        if merge {
            self.sessions.extend(sessions);
        } else {
            self.sessions = sessions;
        }
        Ok(())
    }

    pub fn session_map(&self) -> &HashMap<String, TradeSession> {
        &self.sessions
    }

    pub fn has_session(&self, product: &str) -> bool {
        self.sessions.contains_key(product)
    }

    pub fn get_session(&self, product: &str) -> Option<&TradeSession> {
        self.sessions.get(product)
    }

    pub fn add_session(&mut self, product: &str, session: TradeSession) {
        self.sessions.insert(product.to_string(), session);
    }

    /// 该品种日线开始时间，9:00/9:15/9:30/21:00, 一般是集合竞价所在的时间
    pub fn day_begin(&self, product: &str) -> Option<&MyTimeType> {
        self.sessions.get(product).map(|s| s.day_begin())
    }
    ///该品种日线结束时间，商品15:00，股指曾经15:15，股指现在15:00
    pub fn day_end(&self, product: &str) -> Option<&MyTimeType> {
        self.sessions.get(product).map(|s| s.day_end())
    }
    pub fn morning_begin(&self, product: &str) -> Option<&MyTimeType> {
        self.sessions.get(product).map(|s| s.morning_begin())
    }

    /// 一个时间点, 在时段内吗? 一般应含开始(include_begin?), 是否含结束(include_end?)
    pub fn in_session(
        &self,
        product: &str,
        ts: &MyTimeType,
        include_begin: bool,
        include_end: bool,
    ) -> Option<bool> {
        self.sessions
            .get(product)
            .map(|s| s.in_session(ts, include_begin, include_end))
    }

    pub fn any_in_session(
        &self,
        product: &str,
        start: &MyTimeType,
        end: &MyTimeType,
        include_begin_end: bool,
    ) -> Option<bool> {
        self.sessions
            .get(product)
            .map(|s| s.any_in_session(start, end, include_begin_end))
    }
}

#[cfg(test)]
mod tests {

    use crate::jcswitch::make_time;

    use super::*;

    #[test]
    fn tryload() -> anyhow::Result<()> {
        let csv_str = include_str!("../tradesession.csv");
        let map_ = load_from_csv_content(csv_str)?;
        for (k, v) in map_.iter() {
            println!("{}, {}", k, v);
        }

        let s_mgr = SessionManager::new_from_csv_content(csv_str)?;
        let session = s_mgr.get_session("ag").unwrap();
        println!("ag session: {}", session);
        let day_begin = s_mgr.day_begin("ag").unwrap();
        println!("ag day begin: {}", day_begin);
        let day_end = s_mgr.day_end("ag").unwrap();
        println!("ag day end: {}", day_end);
        s_mgr
            .in_session("ag", &make_time(9, 0, 0), true, false)
            .map(|in_session| println!("ag in session at 09:00:00: {}", in_session));
        s_mgr
            .in_session("ag", &make_time(1, 15, 0), true, false)
            .map(|in_session| println!("ag in session at 1:15:00: {}", in_session));
        s_mgr
            .in_session("ag", &make_time(16, 0, 0), true, false)
            .map(|in_session| println!("ag in session at 16:00:00: {}", in_session));
        Ok(())
    }
}
