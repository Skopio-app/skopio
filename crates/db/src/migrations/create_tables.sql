CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    root_path TEXT,
    metadata TEXT
);

CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    duration INTEGER DEFAULT 0,
    activity_type TEXT NOT NULL,
    app_name TEXT NOT NULL,
    file_name TEXT,
    project_id INTEGER,
    branch_name TEXT,
    language TEXT,
    metadata TEXT,
    status TEXT DEFAULT 'ongoing',
    end_timestamp TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS afk_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    afk_start TEXT NOT NULL,
    afk_end TEXT,
    duration INTEGER
);

CREATE TABLE IF NOT EXISTS goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    target_duration INTEGER NOT NULL,
    frequency TEXT NOT NULL,
    exclude_days TEXT,
    progress INTEGER DEFAULT 0,
    metadata TEXT
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

-- Table: event_tags (many-to-many relationship between events and tags)
CREATE TABLE IF NOT EXISTS event_tags (
    event_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (event_id, tag_id),
    FOREIGN KEY (event_id) REFERENCES events(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

CREATE TABLE IF NOT EXISTS yearly_summaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    year INTEGER NOT NULL,
    total_active_time INTEGER NOT NULL,
    total_afk_time INTEGER NOT NULL,
    most_active_app TEXT,
    most_active_project TEXT,
    metadata TEXT,
    last_updated TEXT NOT NULL
)
