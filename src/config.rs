use once_cell::sync::OnceCell;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub threshold_days: i64,
    pub active_category: String,
    pub archive_category: String,
    #[cfg(feature = "chatgpt")]
    pub chatgpt_token: String,
}

static CONFIG: OnceCell<Config> = OnceCell::new();

pub fn config() -> &'static Config {
    CONFIG.get().unwrap()
}

pub fn init_config() {
    let threshold_days: i64 = env::var("THRESHOLD_DAYS").unwrap().parse().unwrap();
    let active_category: String = env::var("ACTIVE_CATEGORY").unwrap();
    let archive_category: String = env::var("ARCHIVE_CATEGORY").unwrap();

    #[cfg(feature = "chatgpt")]
    let chatgpt_token: String = env::var("CHATGPT_TOKEN").unwrap();

    CONFIG
        .set(Config {
            threshold_days,
            active_category,
            archive_category,
            #[cfg(feature = "chatgpt")]
            chatgpt_token,
        })
        .unwrap();
}
