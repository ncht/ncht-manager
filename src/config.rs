use anyhow::Context as _;
use shuttle_secrets::SecretStore;

#[derive(Debug)]
pub struct Config {
    pub threshold_days: i64,
    pub active_category: String,
    pub archive_category: String,
}

impl Config {
    pub fn from_secret_store(secret_store: &SecretStore) -> anyhow::Result<Self> {
        let threshold_days: i64 = secret_store
            .get("THRESHOLD_DAYS")
            .context("THRESHOLD_DAYS")?
            .parse()?;
        let active_category = secret_store
            .get("ACTIVE_CATEGORY")
            .context("ACTIVE_CATEGORY")?;
        let archive_category = secret_store
            .get("ARCHIVE_CATEGORY")
            .context("ARCHIVE_CATEGORY")?;

        Ok(Self {
            threshold_days,
            active_category,
            archive_category,
        })
    }
}
