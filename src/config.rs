use anyhow::Context as _;
use shuttle_secrets::SecretStore;

#[derive(Debug)]
pub struct Config {
    pub threshold_days: i64,
    pub active_category: String,
    pub archive_category: String,
    pub certified_member_roles: Vec<String>,
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
        let certified_member_roles = secret_store
            .get("CERTIFIED_MEMBER_ROLES")
            .context("CERTIFIED_MEMBER_ROLES")?
            .split(",")
            .map(|s| s.trim().to_owned())
            .collect();

        Ok(Self {
            threshold_days,
            active_category,
            archive_category,
            certified_member_roles,
        })
    }
}
