CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL DEFAULT '',
    updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- Default settings
INSERT OR IGNORE INTO settings (key, value) VALUES ('ai_provider', 'openai');
INSERT OR IGNORE INTO settings (key, value) VALUES ('ai_api_key', '');
INSERT OR IGNORE INTO settings (key, value) VALUES ('ai_model', 'gpt-4o');
INSERT OR IGNORE INTO settings (key, value) VALUES ('ai_base_url', '');
INSERT OR IGNORE INTO settings (key, value) VALUES ('language', 'en');
INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'system');
