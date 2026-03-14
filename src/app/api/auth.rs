use boluo::BoxError;
use boluo::data::Json;
use boluo::response::IntoResponse;

use crate::context::db::DbPoolConnection;
use crate::context::ip::ClientIP;
use crate::model::dto::api::auth::AdminAccessTokenDto;
use crate::model::dto::api::auth::AdminLoginDto;
use crate::validator::Validation;

#[boluo::route("/auth/login", method = "POST")]
pub async fn login(
    ClientIP(ip): ClientIP,
    Json(params): Json<AdminLoginDto>,
    DbPoolConnection(mut db): DbPoolConnection,
) -> Result<impl IntoResponse, BoxError> {
    params.validate(&())?;
    let token = crate::service::auth::login(ip, &params.into(), &mut db).await?;
    Ok(crate::response::ok(AdminAccessTokenDto::from(token)))
}
