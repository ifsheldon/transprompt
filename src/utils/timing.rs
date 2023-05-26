use chrono::{NaiveDate, NaiveDateTime};

pub type VirtualTime = NaiveDateTime;

pub fn create_virtual_time(year: u32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Option<VirtualTime> {
    NaiveDate::from_ymd_opt(year as i32, month, day)
        .and_then(|date| date.and_hms_opt(hour, min, sec))
}