use std::borrow::Cow;
use std::net::IpAddr;
use std::sync::LazyLock;

use totp_rs::TOTP;

use crate::error::{AppError, AppErrorMeta};
use crate::jwt::admin::AdminJwtData;
use crate::model::bo::auth::{AdminAccessTokenBo, AdminBo, AdminLoginBo};
use crate::model::bo::failed_attempts::FailedAttemptsBanBo;
use crate::storage::db::DbConn;
use crate::util::jwt::JwtClaims;

static ADMIN_TOTP: LazyLock<TOTP> =
    LazyLock::new(|| TOTP::from_url(&crate::config::get().admin.totp_url).unwrap());

async fn is_banned(ip: IpAddr) -> Result<(), AppError> {
    if FailedAttemptsBanBo::is_banned(FailedAttemptsBanBo::SCENE_LOGIN, ip, "").await? {
        return Err(AppErrorMeta::BadRequest.with_message("登录尝试次数过多，请稍后再试"));
    }
    Ok(())
}

async fn try_ban(ip: IpAddr, db: &mut DbConn) -> Result<Cow<'static, str>, AppError> {
    let (remaining_times, is_banned) =
        FailedAttemptsBanBo::record_failed_with_ban(FailedAttemptsBanBo::SCENE_LOGIN, ip, "", db)
            .await?;

    if is_banned {
        Ok("登录尝试次数过多，请稍后再试".into())
    } else {
        Ok(format!("密码或口令错误！剩余尝试次数 {remaining_times} 次").into())
    }
}

/// 管理员登录认证
pub async fn login(
    client_ip: IpAddr,
    bo: &AdminLoginBo<'_>,
    db: &mut DbConn,
) -> Result<AdminAccessTokenBo, AppError> {
    is_banned(client_ip).await?;

    if crate::config::get().admin.password != bo.password {
        let message = try_ban(client_ip, db).await?;
        return Err(AppErrorMeta::BadRequest.with_message(message));
    }
    if ADMIN_TOTP.generate_current()? != bo.totp_code {
        let message = try_ban(client_ip, db).await?;
        return Err(AppErrorMeta::BadRequest.with_message(message));
    }
    AdminAccessTokenBo::generate()
}

/// 验证管理员令牌
pub async fn validate_admin_token(token: &str) -> Result<AdminBo, AppError> {
    let claims =
        JwtClaims::<AdminJwtData>::decode(token, crate::config::get().jwt.secret.as_bytes())
            .map_err(|e| {
                AppErrorMeta::AdminAccessTokenInvalid
                    .with_source(e)
                    .with_context(format!("token: {token}"))
            })?;

    if !claims.is_effective() {
        return Err(AppErrorMeta::AdminAccessTokenNotEffective.into_error());
    }
    if claims.is_expired() {
        return Err(AppErrorMeta::AdminAccessTokenExpired.into_error());
    }

    Ok(AdminBo {
        expires_at: claims.exp,
    })
}
