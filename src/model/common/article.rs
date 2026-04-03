use serde::{Deserialize, Serialize};

/// 文章状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum ArticleStatus {
    /// 草稿
    Draft,
    /// 私密：列表不显示，详情 404
    Private,
    /// 隐藏：列表不显示，详情可访问
    Hidden,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArticleAccessibility {
    /// 可见
    Visible,
    /// 不可见
    Invisible,
    /// 需要密码
    NeedPassword,
}

#[derive(Debug, Clone)]
pub enum ArticleContentControl<T> {
    NeedPassword,
    Public(T),
}
