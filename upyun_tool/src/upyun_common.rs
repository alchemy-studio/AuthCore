use chrono::prelude::*;
use std::time::SystemTime;

pub fn get_gmt_date() -> String {
    let date = Utc::now();
    let rfc2822_date_string = format!("{}", date.to_rfc2822());
    let gmt_date = format!("{}", rfc2822_date_string.replace("+0000", "GMT"));
    println!("GMT date : {}", gmt_date);
    gmt_date
}

pub fn get_sys_time_in_secs() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}
