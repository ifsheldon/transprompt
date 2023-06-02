use chrono::{NaiveDate, NaiveDateTime};

pub type VirtualTime = NaiveDateTime;

pub fn create_virtual_time(year: u32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Option<VirtualTime> {
    NaiveDate::from_ymd_opt(year as i32, month, day)
        .and_then(|date| date.and_hms_opt(hour, min, sec))
}

#[cfg(test)]
mod tests {
    use super::create_virtual_time;

    #[test]
    fn test_virtual_time() {
        let before = create_virtual_time(2006, 7, 8, 9, 10, 11).unwrap();
        let after = create_virtual_time(2006, 8, 8, 9, 10, 11).unwrap();
        let duration = after - before;
        assert_eq!(duration.num_days(), 31);
    }
}