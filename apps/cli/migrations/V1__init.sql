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
    source TEXT NOT NULL,
    end_timestamp INTEGER,
    synced BOOLEAN DEFAULT FALSE
)
