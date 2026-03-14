/// 失败次数统计
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FailedAttemptsPo {
    /// 场景
    pub scene: String,
    /// IP
    pub ip: String,
    /// 目标ID
    pub target_id: String,
    /// 失败次数统计
    pub count: u32,
    /// 创建时间
    pub created_at: i64,
    /// 过期时间
    pub expires_at: i64,
}
