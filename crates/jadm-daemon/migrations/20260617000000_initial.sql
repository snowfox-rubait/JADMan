-- Initial schema
CREATE TABLE IF NOT EXISTS downloads (
    id            TEXT PRIMARY KEY,
    url           TEXT NOT NULL,
    filename      TEXT,
    size          INTEGER,
    downloaded    INTEGER DEFAULT 0,
    status        TEXT DEFAULT 'queued',
    category      TEXT,
    folder        TEXT,
    resumable     BOOLEAN DEFAULT 0,
    connections   INTEGER DEFAULT 8,
    engine        TEXT DEFAULT 'aria2c',
    mime_type     TEXT,
    cookies       TEXT,
    netscape_cookies TEXT,
    user_agent    TEXT,
    error         TEXT,
    added_at      TEXT NOT NULL,
    completed_at  TEXT,
    last_tried_at TEXT,
    write_subs    BOOLEAN DEFAULT 0,
    embed_thumbnail BOOLEAN DEFAULT 0,
    embed_chapters  BOOLEAN DEFAULT 0,
    ghost_mode    BOOLEAN DEFAULT 0
);

CREATE TABLE IF NOT EXISTS scheduler_rules (
    id            TEXT PRIMARY KEY,
    download_id   TEXT NOT NULL,
    trigger_type  TEXT NOT NULL,
    trigger_data  TEXT NOT NULL,
    created_at    TEXT NOT NULL,
    fired_at      TEXT,
    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
