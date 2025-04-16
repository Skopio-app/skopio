CREATE TABLE IF NOT EXISTS heartbeats
(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME,
    project_name TEXT,
    project_path TEXT,
    entity_name TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    branch_name TEXT,
    language_name TEXT,
    app_name TEXT NOT NULL,
    is_write BOOL,
    lines INTEGER,
    cursorpos INTEGER,
    synced BOOl DEFAULT 0
);

CREATE TABLE IF NOT EXISTS events
(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME,
    duration INTEGER DEFAULT 0,
    activity_type TEXT,
    app_name TEXT NOT NULL,
    entity_name TEXT,
    entity_type TEXT,
    project_name TEXT,
    project_path TEXT,
    branch_name TEXT,
    language_name TEXT,
    end_timestamp DATETIME,
    synced BOOL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS afk_events
(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    afk_start DATETIME,
    afk_end DATETIME,
    duration INTEGER DEFAULT 0,
    synced BOOL DEFAULT 0
);


