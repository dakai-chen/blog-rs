CREATE TABLE IF NOT EXISTS failed_attempts (
    scene           TEXT    NOT NULL,
    ip              TEXT    NOT NULL,
    target_id       TEXT    NOT NULL,
    count           INTEGER NOT NULL,
    created_at      INTEGER NOT NULL,
    expires_at      INTEGER NOT NULL,
    PRIMARY KEY (scene, ip, target_id)
);

CREATE INDEX IF NOT EXISTS idx_expires_at ON failed_attempts (expires_at);
