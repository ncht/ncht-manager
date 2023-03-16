use once_cell::sync::OnceCell;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub threshold_days: i64,
    pub active_category: String,
    pub archive_category: String,
}

static CONFIG: OnceCell<Config> = OnceCell::new();

pub fn config() -> &'static Config {
    CONFIG.get().unwrap()
}

pub fn init_config() {
    let threshold_days: i64 = env::var("THRESHOLD_DAYS").unwrap().parse().unwrap();
    let active_category: String = env::var("ACTIVE_CATEGORY").unwrap();
    let archive_category: String = env::var("ARCHIVE_CATEGORY").unwrap();

    CONFIG
        .set(Config {
            threshold_days,
            active_category,
            archive_category,
        })
        .unwrap();
}
