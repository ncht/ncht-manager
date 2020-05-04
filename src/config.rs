use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref THRESHOLD_DAYS: i64 = env::var("THRESHOLD_DAYS").unwrap().parse().unwrap();
    pub static ref ACTIVE_CATEGORY: String = env::var("ACTIVE_CATEGORY").unwrap();
    pub static ref ARCHIVE_CATEGORY: String = env::var("ARCHIVE_CATEGORY").unwrap();
}
