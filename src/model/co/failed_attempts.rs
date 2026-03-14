use std::borrow::Cow;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::storage::cache::{CacheData, CacheIdGenerator};

#[derive(Debug, Clone)]
pub struct FailedAttemptsBanCoIdGen<'a> {
    /// 场景
    pub scene: &'a str,
    /// IP
    pub ip: IpAddr,
    /// 目标ID
    pub target_id: &'a str,
}

impl CacheIdGenerator for FailedAttemptsBanCoIdGen<'_> {
    fn generate_id(&self) -> Cow<'_, str> {
        format!("{}:{}:{}", self.scene, self.ip, self.target_id).into()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FailedAttemptsBanCo;

impl CacheData for FailedAttemptsBanCo {
    fn kind() -> &'static str {
        "failed_attempts_ban"
    }
}
