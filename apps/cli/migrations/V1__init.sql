CREATE TABLE IF NOT EXISTS heartbeats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    project_path TEXT NOT NULL,
    branch TEXT,
    entity_name TEXT NOT NULL,
    entity_type TEXT,
    language TEXT,
    app TEXT NOT NULL,
    is_write BOOLEAN DEFAULT FALSE,
    lines INTEGER,
    cursorpos INTEGER,
    synced BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    category TEXT NOT NULL,
    app TEXT NOT NULL,
    entity_name TEXT,
    entity_type TEXT,
    duration INTEGER NOT NULL,
    project_path TEXT NOT NULL,
    branch TEXT,
    language TEXT,
    end_timestamp INTEGER,
    synced BOOLEAN DEFAULT FALSE
)
