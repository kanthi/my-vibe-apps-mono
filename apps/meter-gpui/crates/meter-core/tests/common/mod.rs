#![allow(dead_code)]
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::Path;

pub fn d(s: &str) -> DateTime<Utc> {
    meter_core::util::parse_date(s).unwrap()
}

pub fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

pub fn sqlite(root: &Path, rel: &str, sql: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    Connection::open(p).unwrap().execute_batch(sql).unwrap();
}
