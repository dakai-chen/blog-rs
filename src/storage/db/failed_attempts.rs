use crate::model::po::failed_attempts::FailedAttemptsPo;
use crate::storage::db::DbConn;
use crate::util::time::UnixTimestampSecs;

pub async fn incr_count(po: &FailedAttemptsPo, db: &mut DbConn) -> anyhow::Result<u32> {
    sqlx::query_scalar(
        "
        INSERT INTO failed_attempts (
            `scene`,
            `ip`,
            `target_id`,
            `count`,
            `created_at`,
            `expires_at`
        ) VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT (`scene`, `ip`, `target_id`) DO UPDATE SET
            `count` = `count` + 1
        RETURNING
            `count`
        ",
    )
    .bind(&po.scene)
    .bind(&po.ip)
    .bind(&po.target_id)
    .bind(&po.count)
    .bind(&po.created_at)
    .bind(&po.expires_at)
    .fetch_one(db)
    .await
    .map_err(From::from)
}

pub async fn remove_single_expired(
    scene: &str,
    ip: &str,
    target_id: &str,
    db: &mut DbConn,
) -> anyhow::Result<u64> {
    sqlx::query("DELETE FROM failed_attempts WHERE scene = ? AND ip = ? AND target_id = ? AND expires_at < ?")
        .bind(scene)
        .bind(ip)
        .bind(target_id)
        .bind(UnixTimestampSecs::now().as_i64())
        .execute(db)
        .await
        .map(|res| res.rows_affected())
        .map_err(From::from)
}

pub async fn remove_all_expired(limit: u64, db: &mut DbConn) -> anyhow::Result<u64> {
    sqlx::query(
        "
        DELETE FROM failed_attempts WHERE rowid IN (
            SELECT rowid FROM failed_attempts WHERE expires_at < ? ORDER BY expires_at LIMIT ?
        )
        ",
    )
    .bind(UnixTimestampSecs::now().as_i64())
    .bind(i64::try_from(limit)?)
    .execute(db)
    .await
    .map(|res| res.rows_affected())
    .map_err(From::from)
}
