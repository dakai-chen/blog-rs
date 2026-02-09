use std::sync::Arc;

use crate::config::AppConfig;
use crate::storage::db::DbPool;
use crate::template::TemplateEngine;

/// 应用程序的全局状态
pub struct AppState {
    /// 数据库连接池
    pub db: DbPool,
    /// 模板引擎
    pub template: TemplateEngine,
}

impl AppState {
    pub async fn from_config(config: &AppConfig) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            db: crate::storage::db::build_pool(&config.database).await?,
            template: crate::template::build_template(&config.theme)?,
        }))
    }
}
