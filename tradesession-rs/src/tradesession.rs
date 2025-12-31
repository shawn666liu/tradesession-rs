use anyhow::{Result, anyhow};
use serde_json::Value;
use std::{collections::BTreeSet, fmt::Display};

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
    /// 原始时间,尚未增加4小时
    pub fn new_from_time(hour: u32, minute: u32) -> Self {
        let seconds = hour * 3600 + minute * 60;
        Self::new_from_midnight_seconds(seconds)
    }
    /// 原始秒数,尚未增加4小时
    pub fn new_from_midnight_seconds(seconds: u32) -> Self {
        let secs = (seconds + SECS_IN_FOUR_HOURS) % SECS_IN_ONE_DAY;
        Self(secs)
    }
    /// seconds已经增加4小时
    pub fn new_from_shifted(seconds: u32) -> Self {
        Self(seconds)
    }

    /// Shift后的时间对应的秒数
    pub fn seconds(&self) -> u32 {
        self.0
    }

    /// 名义时间对应的秒数
    pub fn nominal_seconds(&self) -> u32 {
        (self.0 + SECS_IN_ONE_DAY - SECS_IN_FOUR_HOURS) % SECS_IN_ONE_DAY
    }

    /// 名义时间
    pub fn nominal_time(&self) -> MyTimeType {
        let secs = self.nominal_seconds();
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
    pub fn adjust(&mut self, secs: i32) {
        if secs > 0 {
            self.0 += secs as u32;
        } else {
            self.0 -= (-secs) as u32;
        }
    }
}

impl Display for ShiftedTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "with-chrono")]
        {
            write!(
                f,
                "{}, sec {}, {}",
                self.nominal_time().format("%R"),
                self.0,
                self.shifted_time().format("%R")
            )
        }
        #[cfg(feature = "with-jiff")]
        {
            write!(
                f,
                "{}, sec {}, {}",
                self.nominal_time().strftime("%H:%M"),
                self.0,
                self.shifted_time().strftime("%H:%M")
            )
        }
    }
}
impl From<&MyTimeType> for ShiftedTime {
    // 这里有一个严重的问题，使用2025-07-23 00:00:00和2025-07-23 00:00:00.500,
    // 计算出来的ShiftedTime秒数是一样的，500ms的差异被弄丢了
    // 但实际上，前者是上一个bar的结束，后者是新一个bar的开始
    // 因为切分k线时，使用左开右闭区间(]，整点时间是属于前一个周期的，比如收盘时15:00:00,它属于上一个bar
    // 比如商品期货，早上的第一个一分钟bar,
    // 如果不含集合竞价，它是[9:00:00～9:01:00], 第二个(9:01:00~9:02:00]
    // 如果包含集合竞价，它是[8:59:00～9:01:00], 第二个(9:01:00~9:02:00]
    fn from(t: &MyTimeType) -> Self {
        let mut sec = t.hour() as u32 * 3600
            + t.minute() as u32 * 60
            + t.second() as u32
            + SECS_IN_FOUR_HOURS;
        if t.nanosecond() > 0 {
            sec += 1;
        }
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
        self.nominal_time()
    }
}

impl Into<MyTimeType> for &ShiftedTime {
    fn into(self) -> MyTimeType {
        self.nominal_time()
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
    /// 用Time构造, begin必须小于end(除非开始时间大于20点且结束时间跨零点)
    pub fn new(begin: &MyTimeType, end: &MyTimeType) -> Result<Self> {
        Self::new_from_shifted(ShiftedTime::from(begin), ShiftedTime::from(end))
    }

    /// 注意： 输入数据必须已经加过4小时了, begin必须小于end
    pub fn new_from_shifted(begin_sec: ShiftedTime, end_sec: ShiftedTime) -> Result<Self> {
        if begin_sec >= end_sec {
            return Err(anyhow!(
                "SessionSlice: begin must less than end, but got begin: {}, end: {}",
                begin_sec,
                end_sec
            ));
        }
        Ok(Self {
            begin: begin_sec,
            end: end_sec,
        })
    }

    /// 原始时间，尚未增加4小时, start时间必须小于end(除非开始时间大于20点且结束时间跨零点)
    pub fn new_from_time(
        start_hour: u32,
        start_minute: u32,
        end_hour: u32,
        end_minute: u32,
    ) -> Result<Self> {
        let begin_sec = ShiftedTime::new_from_time(start_hour, start_minute);
        let end_sec = ShiftedTime::new_from_time(end_hour, end_minute);
        Self::new_from_shifted(begin_sec, end_sec)
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

    /// 是否为夜盘交易， 所有夜盘的开始时间都是21:00
    pub fn is_night(&self) -> bool {
        // 21:00前移4小时为1:00, 数值应是3600秒
        self.begin.seconds() == 3600
    }

    /// 获取此时间片对应分钟(最大不超过1440,u16足够)的数组，
    /// 含开始，不含结束， 注意：所有数值超前4小时
    pub fn minutes_list(&self) -> BTreeSet<u16> {
        let start_minute = (self.begin.seconds() / 60) as u16;
        let end_minute = (self.end.seconds() / 60) as u16;
        // 注意：end_minute不包含在内
        (start_minute..end_minute).collect()
    }
}

impl Display for SessionSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "with-chrono")]
        {
            write!(
                f,
                "raw({}~{}), act({}~{})",
                self.begin.shifted_time().format("%R"),
                self.end.shifted_time().format("%R"),
                self.begin.nominal_time().format("%R"),
                self.end.nominal_time().format("%R"),
            )
        }
        #[cfg(feature = "with-jiff")]
        {
            write!(
                f,
                "raw({}~{}), act({}~{})",
                self.begin.shifted_time().strftime("%H:%M"),
                self.end.shifted_time().strftime("%H:%M"),
                self.begin.nominal_time().strftime("%H:%M"),
                self.end.nominal_time().strftime("%H:%M"),
            )
        }
    }
}

#[derive(Clone, Debug)]
pub struct TradeSession {
    slices: Vec<SessionSlice>,
    /// 该品种日线开始时间，9:00/9:15/9:30/21:00, 一般是集合竞价所在的时间
    day_begin: MyTimeType,
    ///该品种日线结束时间，商品15:00，股指曾经15:15，股指现在15:00
    day_end: MyTimeType,
    /// 该品种早盘开始时间，9:00/9:15/9:30,非夜盘品种跟day_begin相同
    morning_begin: MyTimeType,
}

impl TradeSession {
    pub fn new() -> Self {
        let day_begin = make_time(9, 0, 0);
        let day_end = make_time(15, 0, 0);
        let morning_begin = day_begin.clone();
        Self {
            slices: vec![],
            day_begin,
            day_end,
            morning_begin,
        }
    }
    pub fn new_from_slices(slices: &Vec<SessionSlice>) -> Self {
        let mut session = Self::new();
        session.slices.extend(slices);
        session.post_fix();
        session
    }

    pub fn new_from_minutes<I, T>(minutes: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<u16> + Copy,
    {
        let mut session = TradeSession::new();
        session.load_from_minutes(minutes);
        session
    }

    /// 生成一个股票的交易时段
    pub fn new_stock_session() -> Self {
        let mut ss = TradeSession::new();
        ss.add_slice(9, 30, 11, 30).expect("no fail");
        ss.add_slice(13, 0, 15, 0).expect("no fail");
        ss.post_fix();
        ss
    }

    /// 生成一个股指期货的交易时段, 现在股指期货跟股票一样
    pub fn new_stock_index_session() -> Self {
        Self::new_stock_session()
    }

    /// 生成一个国债期货的交易时段, 比金融期货多15分钟
    pub fn new_bond_session() -> Self {
        let mut ss = TradeSession::new();
        ss.add_slice(9, 30, 11, 30).expect("no fail");
        ss.add_slice(13, 0, 15, 15).expect("no fail");
        ss.post_fix();
        ss
    }

    /// 生成一个常规的商品期货交易时段(无夜盘)
    pub fn new_commodity_session() -> Self {
        let mut ss = TradeSession::new();
        ss.add_slice(9, 0, 10, 15).expect("no fail");
        ss.add_slice(10, 30, 11, 30).expect("no fail");
        ss.add_slice(13, 30, 15, 0).expect("no fail");
        ss.post_fix();
        ss
    }

    /// 生成一个常规的商品期货（不含金融期货）交易时段(含夜盘)
    pub fn new_commodity_session_night() -> Self {
        let mut ss = TradeSession::new();
        // 添加夜盘 21:00 ~ 2:30
        ss.add_slice(21, 0, 2, 30).expect("no fail");
        ss.add_slice(9, 0, 10, 15).expect("no fail");
        ss.add_slice(10, 30, 11, 30).expect("no fail");
        ss.add_slice(13, 30, 15, 0).expect("no fail");
        ss.post_fix();
        ss
    }

    /// 生成一个涵盖商品股指国债股票等的全部交易时段(含夜盘)
    pub fn new_full_session() -> Self {
        let mut ss = TradeSession::new();
        ss.add_slice(21, 0, 2, 30).expect("no fail");
        ss.add_slice(9, 0, 11, 30).expect("no fail");
        ss.add_slice(13, 0, 15, 15).expect("no fail");
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
    /// 该品种早盘开始时间，9:00/9:15/9:30,非夜盘品种跟day_begin相同
    pub fn morning_begin(&self) -> &MyTimeType {
        &self.morning_begin
    }
    /// 是否有夜盘交易
    pub fn has_night(&self) -> bool {
        self.slices.iter().any(|slice| slice.is_night())
    }

    /// 一个时间点, 在时段内吗? 一般应含开始(include_begin?), 是否含结束(include_end?)
    pub fn in_session(&self, ts: &MyTimeType, include_begin: bool, include_end: bool) -> bool {
        let sec = ShiftedTime::from(ts);
        for slice in &self.slices {
            // 由于每一次调用slice.in_slice(&ts,...)内部都需要转换ts到sec,
            // 所以这里复制代码逻辑，仅转换ts到sec一次
            // let found = slice.in_slice(&ts, include_begin, include_end);
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

    /// start, end之间任意时间点落在session中吗?
    pub fn any_in_session(
        &self,
        start: &MyTimeType,
        end: &MyTimeType,
        include_begin_end: bool,
    ) -> bool {
        let start = ShiftedTime::from(start);
        let end = ShiftedTime::from(end);
        self.slices.iter().any(|slice| {
            if include_begin_end {
                start <= slice.end && end >= slice.begin
            } else {
                start < slice.end && end > slice.begin
            }
        })
    }

    /// 所有add_slice调用完毕之后，应该调用post_fix进行整合
    pub fn add_slice_directly(&mut self, slice: SessionSlice) -> &mut Self {
        self.slices.push(slice);
        self
    }

    /// 输入的是原始时间，尚未平移, 注意：结束时间应大于开始时间(除非开始时间大于20点且结束时间跨零点)
    /// 所有add_slice调用完毕之后，应该调用post_fix进行整合
    pub fn add_slice(
        &mut self,
        start_hour: u32,
        start_minute: u32,
        end_hour: u32,
        end_minute: u32,
    ) -> Result<()> {
        self.slices.push(SessionSlice::new_from_time(
            start_hour,
            start_minute,
            end_hour,
            end_minute,
        )?);
        Ok(())
    }

    /// 这里假定slice已经处理过了，是正确的
    pub fn fix_day_begin_end(&mut self) {
        if self.slices.is_empty() {
            return;
        }
        // 早盘开始时间一般是9:00/9:15/9:30, 所以寻找开始时间在[9:00, 10:00)之间的第一个slice, 其开始时间是morning_begin
        // 如果手动添加了其他时段，比如早上8点我们要连接CTP接口等，那么把开始时间放宽到6点到11点

        let first = self.slices.first().expect("no fail");
        let last = self.slices.last().expect("no fail");
        self.day_begin = first.begin.into();
        self.day_end = last.end.into();

        // 6:00 shift后(6+4)*3600 = 36000, 11:00 shift后54000
        let morning = self.slices.iter().find(|slice| {
            let secs = slice.begin.seconds();
            secs >= 36000 && secs < 54000
        });
        if let Some(slice) = morning {
            self.morning_begin = slice.begin.into();
        } else {
            self.morning_begin = self.day_begin.clone();
        }
    }

    /// 在所有Slice都加入之后，使用minutes方式重算，合并并移除重叠等，计算day_begin、day_end的值
    pub fn post_fix(&mut self) {
        if self.slices.is_empty() {
            return;
        }
        let minutes = self.minutes_list();
        self.internal_load_minutes(&minutes);
        self.fix_day_begin_end();
    }

    /// 获取此时间片对应分钟(u32)的数组，含开始，不含结束
    /// 注意：所有数值超前4小时
    /// 应用场景1：校验所有add_slice，自动移除重迭，自动排序，参看post_fix
    /// 应用场景2：比如仅交易了5个品种，要检查这些品种开市时间段有行情，用以求这些Session的并集
    pub fn minutes_list(&self) -> BTreeSet<u16> {
        self.slices
            .iter()
            .flat_map(|slice| slice.minutes_list())
            .collect()
    }
    pub fn load_from_minutes<I, T>(&mut self, minutes: I)
    where
        I: IntoIterator<Item = T>,
        T: Into<u16> + Copy,
    {
        let minutes: BTreeSet<u16> = minutes.into_iter().map(|t| t.into()).collect();
        self.internal_load_minutes(&minutes);
        self.fix_day_begin_end();
    }

    fn internal_load_minutes(&mut self, minutes: &BTreeSet<u16>) {
        self.slices.clear();
        if minutes.is_empty() {
            return;
        }

        let mut current_start = None;
        let mut prev_minute = None;

        for &minute in minutes {
            match (current_start, prev_minute) {
                (None, _) => {
                    current_start = Some(minute);
                    prev_minute = Some(minute);
                }
                (Some(_), Some(prev)) if minute == prev + 1 => {
                    prev_minute = Some(minute);
                }
                (Some(start), Some(prev)) => {
                    // 中间不连续时，slice结束
                    self.slices.push(SessionSlice {
                        begin: ShiftedTime(start as u32 * 60),
                        end: ShiftedTime(prev as u32 * 60 + 60),
                    });
                    current_start = Some(minute);
                    prev_minute = Some(minute);
                }
                (Some(_), None) => {
                    //impossible case, but to satisfy the match
                    unreachable!("current_start should not be Some without prev_minute");
                }
            }
        }

        // 添加最后一个块（此时 current_start 和 prev_minute 必然都有值）
        if let (Some(start), Some(end)) = (current_start, prev_minute) {
            self.slices.push(SessionSlice {
                begin: ShiftedTime(start as u32 * 60),
                end: ShiftedTime(end as u32 * 60 + 60),
            });
        }
    }
}

impl Display for TradeSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "with-chrono")]
        {
            write!(
                f,
                "day_begin:{}, morning_begin:{}, day_end:{}",
                self.day_begin.format("%R"),
                self.morning_begin.format("%R"),
                self.day_end.format("%R")
            )?;
        }
        #[cfg(feature = "with-jiff")]
        {
            write!(
                f,
                "day_begin:{}, morning_begin:{}, day_end:{}",
                self.day_begin.strftime("%H:%M"),
                self.morning_begin.strftime("%H:%M"),
                self.day_end.strftime("%H:%M")
            )?;
        }
        for (idx, sec) in self.slices.iter().enumerate() {
            write!(f, "\n{}: {}", idx + 1, sec)?;
        }
        Ok(())
    }
}

/// 将数据库中的Session字段转为Vec<SessionSlice>,
/// [{"Begin":"09:00:00","End":"10:15:00"},{"Begin":"10:30:00","End":"11:30:00"},{"Begin":"13:30:00","End":"15:00:00"},{"Begin":"21:00:00","End":"02:30:00"}]
pub fn parse_json_slices(json: &str) -> Result<Vec<SessionSlice>> {
    let mut res = Vec::<SessionSlice>::new();
    match serde_json::from_str(&json.to_lowercase())? {
        Value::Array(arr) => {
            for elem in arr {
                // println!("{:?}", elem);
                // println!("{} ~ {}", elem["Begin"], elem["End"]);
                match (&elem["begin"], &elem["end"]) {
                    (Value::String(bb), Value::String(ee)) => {
                        let begin = parse_time(bb, "%H:%M:%S")?;
                        let end = parse_time(ee, "%H:%M:%S")?;
                        res.push(SessionSlice::new(&begin, &end)?);
                    }
                    _ => return Err(anyhow!("trade session解析错误: {}", elem)),
                }
            }
        }
        _ => return Err(anyhow!("trade session字符串必须是Array类型")),
    }

    return Ok(res);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn slice_to_minutes() -> anyhow::Result<()> {
        let slice = SessionSlice::new_from_time(9, 0, 9, 5)?;
        let mut minutes = slice.minutes_list();
        println!("slice minutes: {:?}", minutes);
        assert_eq!(minutes.len(), 5);
        assert!(minutes.contains(&780)); // 9:00 is 540 + 240 minutes
        assert!(minutes.contains(&781)); // 9:01 is 541 + 240 minutes
        assert!(minutes.contains(&782)); // 9:02 is 542 + 240 minutes
        assert!(minutes.contains(&783)); // 9:03 is 543 + 240 minutes
        assert!(minutes.contains(&784)); // 9:04 is 544 + 240 minutes
        assert!(!minutes.contains(&785)); // 9:05 is not included

        minutes.insert(840); // 10:00 is 780 + 60 minutes
        minutes.insert(841); // 10:01 is 781 + 60 minutes

        let session = TradeSession::new_from_minutes(minutes.clone());
        let minutes2 = session.minutes_list();
        println!("slice minutes2: {:?}", minutes2);

        assert_eq!(minutes == minutes2, true);
        assert_eq!(session.slices.len(), 2);
        for slice in &session.slices {
            println!("{}", slice);
        }
        Ok(())
    }

    #[test]
    fn _trade_session() -> anyhow::Result<()> {
        let stk = TradeSession::new_commodity_session_night();
        assert_eq!(stk.slices.len(), 4);
        for slice in &stk.slices {
            println!("{}", slice);
        }
        assert!(!stk.in_session(&make_time(8, 59, 10), true, false));
        assert!(stk.in_session(&make_time(9, 59, 10), true, false));
        assert!(stk.in_session(&make_time(0, 59, 10), true, false));
        assert!(!stk.in_session(&make_time(20, 59, 10), true, false));

        // 国债???
        //assert!(stk.in_session(&make_time(15, 14, 59), true, false));
        // assert!(!stk.in_session(&make_time(15, 15, 0), true, false));

        let slice = SessionSlice::new_from_time(21, 0, 2, 30)?;
        assert!(slice.is_night());
        assert_eq!(slice.begin(), ShiftedTime::from(make_time(21, 0, 0)));
        assert_eq!(slice.end(), ShiftedTime::from(make_time(2, 30, 0)));

        let slice = SessionSlice::new_from_time(9, 0, 10, 15)?;
        assert!(!slice.is_night());
        Ok(())
    }

    #[test]
    fn parse_session_json() -> Result<()> {
        let json = "[{\"Begin\":\"09:00:00\",\"end\":\"10:15:00\"},{\"Begin\":\"10:30:00\",\"End\":\"11:30:00\"},{\"Begin\":\"13:30:00\",\"End\":\"15:00:00\"},{\"Begin\":\"21:00:00\",\"End\":\"01:00:00\"}]";
        let slice_vec = parse_json_slices(json)?;
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
    #[test]
    fn fix_fail() {
        let nanos_since_midnight_start = 82800000000000;
        let start = time_from_midnight_nanos(nanos_since_midnight_start);

        // 这个86400秒本来会失败，因为达到或超过24小时了，应该是零点而不是24点
        // 所以time_from_midnight_nanos()内部进行了取模，这样就不会失败了
        let nanos_since_midnight_end = 86400000000000;
        let end = time_from_midnight_nanos(nanos_since_midnight_end);
        println!("start {}, end {}", start, end);
    }
}
