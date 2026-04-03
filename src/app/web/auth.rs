use std::sync::Arc;

use boluo::BoxError;
use boluo::data::{Extension, Form};
use boluo::response::{Html, IntoResponse, Redirect};

use crate::context::auth::{AdminCleanCookie, AdminFromCookie, AdminWriteCookie};
use crate::context::db::DbPoolConnection;
use crate::context::ip::ClientIP;
use crate::model::dto::web::auth::AdminLoginSubmitDto;
use crate::model::vo::auth::{AdminLoginVo, AdminLogoutVo};
use crate::state::AppState;
use crate::template::render::PageContext;
use crate::validator::Validation;

#[boluo::route("/login", method = ["GET"])]
pub async fn login(
    admin: Option<AdminFromCookie>,
    Extension(state): Extension<Arc<AppState>>,
) -> Result<impl IntoResponse, BoxError> {
    if admin.is_some() {
        return Ok(Redirect::to("/")?.into_response()?);
    }
    let context = PageContext::new(AdminLoginVo).admin(admin.map(Into::into));
    Html(state.template.typed_render(&context)).into_response()
}

#[boluo::route("/login", method = ["POST"])]
pub async fn login_submit(
    ClientIP(ip): ClientIP,
    Form(params): Form<AdminLoginSubmitDto>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let token = crate::service::auth::login(ip, &params.into(), &mut db).await?;
    Ok((AdminWriteCookie::from(token), Redirect::to("/")))
}

#[boluo::route("/logout", method = ["GET"])]
pub async fn logout(
    AdminFromCookie(admin): AdminFromCookie,
    Extension(state): Extension<Arc<AppState>>,
) -> Result<impl IntoResponse, BoxError> {
    let context = PageContext::new(AdminLogoutVo).admin(admin);
    Ok(Html(state.template.typed_render(&context)))
}

#[boluo::route("/logout", method = ["POST"])]
pub async fn logout_submit() -> Result<impl IntoResponse, BoxError> {
    Ok((AdminCleanCookie, Redirect::to("/")))
}
