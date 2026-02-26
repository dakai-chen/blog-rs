use serde::{Deserialize, Serialize};

/// 文章状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum ArticleStatus {
    /// 草稿
    Draft,
    /// 私密
    Private,
    /// 发布
    Published,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchArticleSort {
    /// 排序规则：published_at DESC NULLS FIRST, updated_at DESC
    ByPublishedAtDesc,
    /// 排序规则：updated_at DESC NULLS FIRST
    ByUpdatedAtDesc,
}
