// https://github.com/rust-lang/rust/issues/63063
// https://stackoverflow.com/questions/27454761/what-is-a-crate-attribute-and-where-do-i-add-it
// https://www.worthe-it.co.za/blog/2017-01-15-aliasing-traits-in-rust.html
// - [rust - What is a crate attribute and where do I add it? - Stack Overflow](https://stackoverflow.com/questions/27454761/what-is-a-crate-attribute-and-where-do-i-add-it)
// - [Tracking issue for RFC 2515, “Permit impl Trait in type aliases” · Issue #63063 · rust-lang/rust · GitHub](https://github.com/rust-lang/rust/issues/63063)

use chrono::{NaiveDateTime};
use uuid::Uuid;

// use crate::common::HtyErr;

pub mod db;
pub mod jwt;
pub mod logger;
pub mod pagination;
pub mod common;
pub mod test_scaffold;
pub mod upyun;
pub mod web;
pub mod wx;
pub mod redis_util;
pub mod cert;
pub mod models;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate diesel;
extern crate crypto;
extern crate dotenv;
extern crate ring;

pub fn uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn pass_or_panic<T>(res: anyhow::Result<T>) {
    match res {
        Ok(_) => (),
        Err(e) => {
            panic!("panic!!! -> {:?}", e)
        }
    }
}

pub fn pass_or_panic2<T>(res: anyhow::Result<T>) {
    match res {
        Ok(_) => (),
        Err(e) => {
            panic!("panic!!! -> {:?}", e)
        }
    }
}

pub fn n_hour_later(now: &NaiveDateTime, n: i64) -> anyhow::Result<NaiveDateTime> {
    chrono::Duration::try_hours(n)
        .map(|duration| *now + duration)
        .ok_or_else(|| anyhow::anyhow!("Invalid number of hours: {}", n))
}

pub fn remove_quote(str: &String) -> String {
    str.replace("\"", "")
}