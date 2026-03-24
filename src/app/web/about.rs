use std::sync::Arc;

use boluo::BoxError;
use boluo::data::Extension;
use boluo::response::{Html, IntoResponse};

use crate::context::auth::AdminFromCookie;
use crate::context::db::DbPoolConnection;
use crate::error::AppErrorMeta;
use crate::model::bo::article::GetArticleBo;
use crate::model::bo::visitor::VisitorBo;
use crate::model::vo::about::AboutVo;
use crate::state::AppState;
use crate::template::render::PageContext;

#[boluo::route("/about", method = ["GET"])]
pub async fn about(
    admin: Option<AdminFromCookie>,
    visitor: VisitorBo,
    Extension(state): Extension<Arc<AppState>>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    if let Some(about_article_id) = &crate::config::get().article.about_article_id {
        let params = GetArticleBo {
            article_id: about_article_id.into(),
        };
        let Some(article) =
            crate::service::article::get_article(admin.as_deref(), &visitor, &params, &mut db)
                .await?
        else {
            return Err(AppErrorMeta::NotFound.with_message("文章不存在").into());
        };
        let vo = AboutVo::from(article);
        let context = PageContext::new(vo).admin(admin.map(Into::into));
        Ok(Html(state.template.typed_render(&context)))
    } else {
        let vo = AboutVo::default();
        let context = PageContext::new(vo).admin(admin.map(Into::into));
        Ok(Html(state.template.typed_render(&context)))
    }
}
