CREATE TABLE IF NOT EXISTS article (
    id                  TEXT    PRIMARY KEY,
    title               TEXT    NOT NULL,
    excerpt             TEXT    NOT NULL,
    markdown_content    TEXT    NOT NULL,
    plain_content       TEXT    NOT NULL,
    render_content      TEXT    NOT NULL,
    render_version      TEXT    NOT NULL,
    password            TEXT,
    status              TEXT    NOT NULL,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL,
    published_at        INTEGER
);

CREATE INDEX IF NOT EXISTS idx_published_at ON article (published_at DESC);
CREATE INDEX IF NOT EXISTS idx_status_published_at ON article (status, published_at DESC);
CREATE INDEX IF NOT EXISTS idx_updated_at ON article (updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_status_updated_at ON article (status, updated_at DESC);

CREATE VIRTUAL TABLE IF NOT EXISTS article_fts USING fts5 (
    id UNINDEXED,
    title,
    excerpt,
    plain_content,
    content='article',
    tokenize='simple'
);

CREATE TRIGGER IF NOT EXISTS article_fts_insert AFTER INSERT
ON article
BEGIN
    INSERT INTO article_fts (rowid, id, title, excerpt, plain_content)
    VALUES (NEW.rowid, NEW.id, NEW.title, NEW.excerpt, NEW.plain_content);
END;

CREATE TRIGGER IF NOT EXISTS article_fts_delete AFTER DELETE
ON article
BEGIN
    DELETE FROM article_fts WHERE id = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS article_fts_update AFTER UPDATE
ON article
BEGIN
    INSERT OR REPLACE INTO article_fts (rowid, id, title, excerpt, plain_content)
    VALUES (NEW.rowid, NEW.id, NEW.title, NEW.excerpt, NEW.plain_content);
END;

CREATE VIRTUAL TABLE IF NOT EXISTS article_fts_public USING fts5 (
    id UNINDEXED,
    title,
    excerpt,
    plain_content,
    content='article',
    tokenize='simple'
);

CREATE TRIGGER IF NOT EXISTS article_fts_public_insert AFTER INSERT
ON article
BEGIN
    INSERT INTO article_fts_public (rowid, id, title, excerpt, plain_content)
    SELECT NEW.rowid, NEW.id, NEW.title, NEW.excerpt, NEW.plain_content
    WHERE NEW.status = 'Published' AND NEW.password IS NULL;
END;

CREATE TRIGGER IF NOT EXISTS article_fts_public_delete AFTER DELETE
ON article
BEGIN
    DELETE FROM article_fts_public WHERE id = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS article_fts_public_update AFTER UPDATE
ON article
BEGIN
    INSERT OR REPLACE INTO article_fts_public (rowid, id, title, excerpt, plain_content)
    SELECT NEW.rowid, NEW.id, NEW.title, NEW.excerpt, NEW.plain_content
    WHERE NEW.status = 'Published' AND NEW.password IS NULL;
END;
