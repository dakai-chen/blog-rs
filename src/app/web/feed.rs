use std::sync::Arc;

use boluo::BoxError;
use boluo::data::Extension;
use boluo::http::header::CONTENT_TYPE;
use boluo::response::IntoResponse;

use crate::context::db::DbPoolConnection;
use crate::model::bo::article::SearchArticleBo;
use crate::model::common::article::SearchArticleSort;
use crate::model::vo::feed::{AtomVo, RssVo};
use crate::state::AppState;
use crate::template::render::PageContext;

#[boluo::route("/rss.xml", method = ["GET"])]
pub async fn rss(
    Extension(state): Extension<Arc<AppState>>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    let params = SearchArticleBo {
        full_text: None,
        status: None,
        published_at_ge: None,
        published_at_lt: None,
        page: Some(1),
        size: crate::config::get().article.feed.page_size,
        sort: None,
    };
    let list = crate::service::article::search_article(None, &params, &mut db).await?;
    let vo = RssVo::from(list);
    let context = PageContext::new(vo).admin(None);
    let headers = [(CONTENT_TYPE, "application/xml; charset=utf-8")];
    Ok((headers, state.template.typed_render(&context)))
}

#[boluo::route("/atom.xml", method = ["GET"])]
pub async fn atom(
    Extension(state): Extension<Arc<AppState>>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    let params = SearchArticleBo {
        full_text: None,
        status: None,
        published_at_ge: None,
        published_at_lt: None,
        page: Some(1),
        size: crate::config::get().article.feed.page_size,
        sort: Some(SearchArticleSort::ByUpdatedAtDesc),
    };
    let list = crate::service::article::search_article(None, &params, &mut db).await?;
    let vo = AtomVo::from(list);
    let context = PageContext::new(vo).admin(None);
    let headers = [(CONTENT_TYPE, "application/xml; charset=utf-8")];
    Ok((headers, state.template.typed_render(&context)))
}
