use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

pub fn naive_to_local(naive: NaiveDateTime) -> DateTime<Local> {
    Local
        .from_local_datetime(&naive)
        .single()
        .unwrap_or_else(|| panic!("Ambiguous or invalid local datetime: {}", naive))
}
