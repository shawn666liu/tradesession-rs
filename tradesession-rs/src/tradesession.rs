use anyhow::{anyhow, Context, Result};
use encoding_rs_io::DecodeReaderBytes;
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    fs::File,
    io::Read,
    path::Path,
};

#[cfg(feature = "with-chrono")]
use chrono::Timelike;

use crate::jcswitch::*;

/// 4小时对应的总秒数
pub const SECS_IN_FOUR_HOURS: u32 = 4 * 60 * 60;

/// 每天的总秒数
pub const SECS_IN_ONE_DAY: u32 = 86400;

/// 将日内时间增加4小时后得到的时间，用于规避夜盘跨零点的问题
/// 即夜里20:00:00作为新交易日的0:00:00
/// 但不超过24:00:00，对其模86400
/// 以秒作为字段进行记录和比较
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub struct ShiftedTime(pub u32);

impl ShiftedTime {
    pub fn from_num_seconds_from_midnight(seconds: u32) -> Self {
        let secs = (seconds + SECS_IN_FOUR_HOURS) % SECS_IN_ONE_DAY;
        Self(secs)
    }

    /// Shift后的时间对应的秒数
    pub fn secs(&self) -> u32 {
        self.0
    }

    /// 原始时间对应的秒数
    pub fn origin_secs(&self) -> u32 {
        (self.0 + SECS_IN_ONE_DAY - SECS_IN_FOUR_HOURS) % SECS_IN_ONE_DAY
    }

    pub fn origin_time(&self) -> MyTimeType {
        let secs = self.origin_secs();
        let h = secs / (60 * 60);
        let m = secs % (60 * 60) / 60;
        let s = secs % 60;
        make_time(h, m, s)
    }

    pub fn shifted_time(&self) -> MyTimeType {
        let secs = self.0;
        let h = secs / (60 * 60);
        let m = secs % (60 * 60) / 60;
        let s = secs % 60;
        make_time(h, m, s)
    }

    /// 微调,慎用
    pub fn adjust(&mut self, secs: i8) {
        if secs > 0 {
            self.0 += secs as u32;
        } else {
            self.0 -= (-secs) as u32;
        }
    }
}

impl Display for ShiftedTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, sec {}, {}",
            self.origin_time(),
            self.0,
            self.shifted_time()
        )
    }
}
impl From<&MyTimeType> for ShiftedTime {
    fn from(t: &MyTimeType) -> Self {
        let sec = t.hour() as u32 * 3600
            + t.minute() as u32 * 60
            + t.second() as u32
            + SECS_IN_FOUR_HOURS;
        let sec = sec % SECS_IN_ONE_DAY;
        Self(sec)
    }
}

impl From<MyTimeType> for ShiftedTime {
    fn from(t: MyTimeType) -> Self {
        Self::from(&t)
    }
}

impl Into<MyTimeType> for ShiftedTime {
    fn into(self) -> MyTimeType {
        self.origin_time()
    }
}

impl Into<MyTimeType> for &ShiftedTime {
    fn into(self) -> MyTimeType {
        self.origin_time()
    }
}

/// 内部保存开始和结束时间的秒数,
/// 由于需要处理夜盘跨零点的问题, 所有时间比实际时间增加4小时,
/// 即夜里20:00:00作为新交易日的0:00:00
/// 但不超过24:00:00，对其模86400
#[derive(Clone, Debug, Copy)]
pub struct SessionSlice {
    begin: ShiftedTime,
    end: ShiftedTime,
}

impl SessionSlice {
    // /// increase 4 hours. 秒数加4小时, 再模86400, 即不超过1天, 变为内部数据
    // #[inline]
    // pub fn inc4hours(sec: u32) -> u32 {
    //     (sec + SECS_IN_FOUR_HOURS) % SECS_IN_ONE_DAY
    // }
    // /// decrease 4 hours. 秒数减少4小时, 再模86400, 不超过1天, 恢复到正常数据
    // #[inline]
    // pub fn dec4hours(sec: u32) -> u32 {
    //     // 直接减4小时可能为负数，先加一天再减
    //     (sec + SECS_IN_ONE_DAY - SECS_IN_FOUR_HOURS) % SECS_IN_ONE_DAY
    // }

    /// 注意： 输入数据必须已经加过4小时了
    fn internal_new(begin_sec: ShiftedTime, end_sec: ShiftedTime) -> Result<Self> {
        if begin_sec >= end_sec {
            return Err(anyhow!(
                "SessionSlice: begin must less than end, but {} > {}",
                begin_sec,
                end_sec
            ));
        }

        Ok(Self {
            begin: begin_sec,
            end: end_sec,
        })
    }

    /// 用Time构造
    pub fn new(begin: &MyTimeType, end: &MyTimeType) -> Result<Self> {
        Self::internal_new(ShiftedTime::from(begin), ShiftedTime::from(end))
    }

    /// 注意：超前4小时
    pub fn begin(&self) -> ShiftedTime {
        self.begin
    }
    /// 注意：超前4小时
    pub fn end(&self) -> ShiftedTime {
        self.end
    }

    /// 一个时间点, 在时段内吗? 一般应含开始(include_begin?), 是否含结束(include_end?)
    pub fn in_slice(&self, ts: &MyTimeType, include_begin: bool, include_end: bool) -> bool {
        let sec = ShiftedTime::from(ts);
        match (include_begin, include_end) {
            (true, true) => sec >= self.begin && sec <= self.end,
            (true, false) => sec >= self.begin && sec < self.end,
            (false, true) => sec > self.begin && sec <= self.end,
            (false, false) => sec > self.begin && sec < self.end,
        }
    }
    // /// 获取此时间片对应分钟(u32)的数组，含开始，不含结束
    // /// 注意：所有数值超前4小时
    // pub fn to_minustes_vec(&self) -> Vec<u32> {
    //     let mut v = Vec::<u32>::new();
    //     let mut start = self.begin_sec;
    //     while start < self.end_sec {
    //         v.push(start / 60);
    //         start += 60;
    //     }
    //     v
    // }
}

impl Display for SessionSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "raw({}~{}), act({}~{})",
            self.begin.shifted_time(),
            self.end.shifted_time(),
            self.begin.origin_time(),
            self.end.origin_time(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct TradeSession {
    slices: Vec<SessionSlice>,
    /// 该品种日线开始时间，9:00/9:15/9:30/21:00, 一般是集合竞价所在的时间
    day_begin: MyTimeType,
    ///该品种日线结束时间，商品15:00，股指曾经15:15，股指现在15:00
    day_end: MyTimeType,
}

impl TradeSession {
    pub fn new(slices: Vec<SessionSlice>) -> Self {
        let mut session = Self {
            slices,
            day_begin: make_time(9, 0, 0),
            day_end: make_time(15, 0, 0),
        };
        session.post_fix();
        session
    }

    /// 生成一个股票的交易时段
    pub fn new_stock_session() -> Self {
        let mut ss = TradeSession::new(vec![]);
        ss.add_slice(
            SessionSlice::new(&make_time(9, 30, 0), &make_time(11, 30, 0)).expect("no fail"),
        );
        ss.add_slice(
            SessionSlice::new(&make_time(13, 0, 0), &make_time(15, 0, 0)).expect("no fail"),
        );
        ss.post_fix();
        ss
    }

    /// 生成一个股指期货的交易时段
    pub fn new_stock_index_session() -> Self {
        let mut ss = TradeSession::new(vec![]);
        ss.add_slice(
            SessionSlice::new(&make_time(9, 15, 0), &make_time(11, 30, 0)).expect("no fail"),
        );
        ss.add_slice(
            SessionSlice::new(&make_time(13, 0, 0), &make_time(15, 0, 0)).expect("no fail"),
        );
        ss.post_fix();
        ss
    }

    /// 生成一个常规的商品期货交易时段(无夜盘)
    pub fn new_commodity_session() -> Self {
        let mut ss = TradeSession::new(vec![]);
        ss.add_slice(
            SessionSlice::new(&make_time(9, 0, 0), &make_time(10, 15, 0)).expect("no fail"),
        );
        ss.add_slice(
            SessionSlice::new(&make_time(10, 30, 0), &make_time(11, 30, 0)).expect("no fail"),
        );
        ss.add_slice(
            SessionSlice::new(&make_time(13, 30, 0), &make_time(15, 0, 0)).expect("no fail"),
        );
        ss.post_fix();
        ss
    }

    /// 生成一个常规的商品期货（不含金融期货）交易时段(含夜盘)
    pub fn new_commodity_session_night() -> Self {
        let mut ss = TradeSession::new(vec![]);
        // 添加夜盘 21:00 ~ 2:30
        ss.add_slice(
            SessionSlice::new(&make_time(21, 0, 0), &make_time(2, 30, 0)).expect("no fail"),
        );
        ss.add_slice(
            SessionSlice::new(&make_time(9, 0, 0), &make_time(10, 15, 0)).expect("no fail"),
        );
        ss.add_slice(
            SessionSlice::new(&make_time(10, 30, 0), &make_time(11, 30, 0)).expect("no fail"),
        );
        ss.add_slice(
            SessionSlice::new(&make_time(13, 30, 0), &make_time(15, 0, 0)).expect("no fail"),
        );
        ss.post_fix();
        ss
    }

    /// 生成一个涵盖商品股指国债股票等的全部交易时段(含夜盘)
    pub fn new_full_session() -> Self {
        let mut ss = TradeSession::new(vec![]);
        // 添加夜盘 21:00 ~ 2:30
        ss.add_slice(
            SessionSlice::new(&make_time(21, 0, 0), &make_time(2, 30, 0)).expect("no fail"),
        );
        ss.add_slice(
            SessionSlice::new(&make_time(9, 0, 0), &make_time(11, 30, 0)).expect("no fail"),
        );
        ss.add_slice(
            SessionSlice::new(&make_time(13, 30, 0), &make_time(15, 15, 0)).expect("no fail"),
        );
        ss.post_fix();
        ss
    }

    /// 注意： 所有数值比实际时间多4小时
    pub fn get_slices(&self) -> &Vec<SessionSlice> {
        &self.slices
    }
    /// 注意： 所有数值比实际时间多4小时
    pub fn get_slices_mut(&mut self) -> &mut Vec<SessionSlice> {
        &mut self.slices
    }
    /// 该品种日线开始时间，9:00/9:15/9:30/21:00, 一般是集合竞价所在的时间
    pub fn day_begin(&self) -> &MyTimeType {
        &self.day_begin
    }
    ///该品种日线结束时间，商品15:00，股指曾经15:15，股指现在15:00
    pub fn day_end(&self) -> &MyTimeType {
        &self.day_end
    }

    /// 一个时间点, 在时段内吗? 一般应含开始(include_begin?), 是否含结束(include_end?)
    pub fn in_session(&self, ts: &MyTimeType, include_begin: bool, include_end: bool) -> bool {
        let sec = ShiftedTime::from(ts);
        for slice in &self.slices {
            let found = match (include_begin, include_end) {
                (true, true) => sec >= slice.begin && sec <= slice.end,
                (true, false) => sec >= slice.begin && sec < slice.end,
                (false, true) => sec > slice.begin && sec <= slice.end,
                (false, false) => sec > slice.begin && sec < slice.end,
            };
            if found {
                return true;
            }
        }
        return false;
    }
    /// 所有add_slice调用完毕之后，应该调用post_fix进行整合
    pub fn add_slice(&mut self, slice: SessionSlice) -> &mut Self {
        self.slices.push(slice);
        self
    }

    /// 在所有Slice都加入之后，合并连续Slice，移除重叠等，并计算day_begin、day_end的值
    pub fn post_fix(&mut self) {
        if self.slices.is_empty() {
            return;
        }
        // BTree自动排序并移除重复的begin_sec
        let dict: BTreeMap<ShiftedTime, ShiftedTime> = self
            .slices
            .iter()
            .map(|slice| (slice.begin, slice.end))
            .collect();
        // todo: 重叠检测及移除，连续项合并等，暂时不做

        self.slices = dict
            .into_iter()
            .map(|(k, v)| SessionSlice::internal_new(k, v).expect("no fail"))
            .collect();

        let first = self.slices.first().expect("no fail");
        let last = self.slices.last().expect("no fail");
        self.day_begin = first.begin.into();
        self.day_end = last.end.into();
    }

    // /// 获取此时间片对应分钟(u32)的数组，含开始，不含结束
    // /// 注意：所有数值超前4小时
    // pub fn to_minustes_vec(&self) -> Vec<u32> {
    //     let mut v = Vec::<u32>::new();
    //     for slice in self.slices.iter() {
    //         v.append(&mut slice.to_minustes_vec())
    //     }
    //     v
    // }

    /// 将数据库中的Session字段转为Vec<SessionSlice>,
    /// [{"Begin":"09:00:00","End":"10:15:00"},{"Begin":"10:30:00","End":"11:30:00"},{"Begin":"13:30:00","End":"15:00:00"},{"Begin":"21:00:00","End":"02:30:00"}]
    pub fn parse_slices(json: &str) -> Result<Vec<SessionSlice>> {
        let mut res = Vec::<SessionSlice>::new();
        match serde_json::from_str(&json.to_lowercase())? {
            Value::Array(arr) => {
                for elem in arr {
                    // println!("{:?}", elem);
                    // println!("{} ~ {}", elem["Begin"], elem["End"]);
                    match (&elem["begin"], &elem["end"]) {
                        (Value::String(bb), Value::String(ee)) => {
                            #[cfg(feature = "with-jiff")]
                            {
                                let begin = MyTimeType::strptime("%H:%M:%S", bb)?;
                                let end = MyTimeType::strptime("%H:%M:%S", ee)?;
                                res.push(SessionSlice::new(&begin, &end)?);
                            }
                            #[cfg(feature = "with-chrono")]
                            {
                                let begin = MyTimeType::parse_from_str(bb, "%H:%M:%S")?;
                                let end = MyTimeType::parse_from_str(ee, "%H:%M:%S")?;
                                res.push(SessionSlice::new(&begin, &end)?);
                            }
                        }
                        _ => return Err(anyhow!("trade session解析错误: {}", elem)),
                    }
                }
            }
            _ => return Err(anyhow!("trade session字符串必须是Array类型")),
        }

        return Ok(res);
    }

    pub fn load_from_read<R: Read>(read: R) -> Result<HashMap<String, TradeSession>> {
        let mut hash: HashMap<String, Self> = Default::default();
        let mut rdr = csv::Reader::from_reader(read);
        for line in rdr.records() {
            let rec = line?;
            if rec.len() == 3 {
                let json = &rec[2];
                let slices = Self::parse_slices(json)?;
                let session = Self::new(slices);
                hash.insert(rec[0].into(), session);
            } else {
                return Err(anyhow!("bad format session {:#?}", rec));
            }
        }
        Ok(hash)
    }

    /// csv文件是直接从数据库表导出的,一共三列, product,exchange,sessions
    /// ag,SHFE,"[{""Begin"":""09:00:00"",""End"":""10:15:00""},{""Begin"":""10:30:00"",""End"":""11:30:00""},{""Begin"":""13:30:00"",""End"":""15:00:00""},{""Begin"":""21:00:00"",""End"":""02:30:00""}]"
    pub fn load_from_csv<P: AsRef<Path>>(path: P) -> Result<HashMap<String, TradeSession>> {
        let path = path.as_ref();
        let file = File::open(path).with_context(|| path.display().to_string())?;
        return Self::load_from_read(DecodeReaderBytes::new(file));
    }
}
impl Display for TradeSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "day_begin: {}, day_end: {}\n",
            self.day_begin, self.day_end
        )?;
        for (idx, sec) in self.slices.iter().enumerate() {
            write!(f, "{}: {}\n", idx, sec)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn _trade_session() {
        let stk = TradeSession::new_commodity_session_night();
        // let v = stk.to_minustes_vec();
        // println!("{:?}", v);
        assert!(!stk.in_session(&make_time(8, 59, 10), true, false));
        assert!(stk.in_session(&make_time(9, 59, 10), true, false));
        assert!(stk.in_session(&make_time(0, 59, 10), true, false));
        assert!(!stk.in_session(&make_time(20, 59, 10), true, false));

        // 国债???
        //assert!(stk.in_session(&make_time(15, 14, 59), true, false));
        // assert!(!stk.in_session(&make_time(15, 15, 0), true, false));
    }

    #[test]
    fn parse_session_json() -> Result<()> {
        let json = "[{\"Begin\":\"09:00:00\",\"end\":\"10:15:00\"},{\"Begin\":\"10:30:00\",\"End\":\"11:30:00\"},{\"Begin\":\"13:30:00\",\"End\":\"15:00:00\"},{\"Begin\":\"21:00:00\",\"End\":\"01:00:00\"}]";
        let slice_vec = TradeSession::parse_slices(json)?;
        assert_eq!(slice_vec.len(), 4);
        println!("parsed vec lenght is {}", slice_vec.len());
        for slice in &slice_vec {
            println!("{}", slice);
        }
        assert_eq!(slice_vec[3].begin(), ShiftedTime::from(make_time(21, 0, 0)));
        assert_eq!(slice_vec[3].end(), ShiftedTime::from(make_time(1, 0, 0)));

        // let session = TradeSession::new(slice_vec);

        Ok(())
    }
}
