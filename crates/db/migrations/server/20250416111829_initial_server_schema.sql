CREATE TABLE IF NOT EXISTS projects
(
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    name      TEXT NOT NULL UNIQUE,
    root_path TEXT
);

CREATE TABLE IF NOT EXISTS branches
(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS languages
(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS apps
(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS entities
(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    type TEXT,
    UNIQUE (project_id, name),
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS events
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp     DATETIME NOT NULL,
    duration      INTEGER DEFAULT 0,
    activity_type TEXT     NOT NULL,
    app_id      INTEGER     NOT NULL,
    entity_id     INTEGER,
    project_id    INTEGER,
    branch_id   INTEGER,
    language_id      INTEGER,
    end_timestamp DATETIME,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
    FOREIGN KEY (app_id) REFERENCES apps (id) ON DELETE CASCADE,
    FOREIGN KEY (language_id) REFERENCES languages (id) ON DELETE CASCADE,
    FOREIGN KEY (branch_id) REFERENCES branches (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS afk_events
(
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    afk_start DATETIME NOT NULL,
    afk_end   DATETIME,
    duration  INTEGER
);

CREATE TABLE IF NOT EXISTS goals
(
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL,
    target_duration INTEGER NOT NULL,
    frequency       TEXT    NOT NULL,
    exclude_days    TEXT,
    progress        INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS heartbeats
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id    INTEGER,
    entity_id     INTEGER,
    branch_id     INTEGER,
    language_id   INTEGER,
    app_id        INTEGER NOT NULL,
    timestamp     DATETIME NOT NULL,
    is_write      BOOLEAN DEFAULT FALSE,
    lines         INTEGER,
    cursorpos     INTEGER,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
    FOREIGN KEY (entity_id) REFERENCES entities (id) ON DELETE CASCADE,
    FOREIGN KEY (branch_id) REFERENCES branches (id) ON DELETE CASCADE,
    FOREIGN KEY (language_id) REFERENCES languages (id) ON DELETE CASCADE,
    FOREIGN KEY (app_id) REFERENCES apps (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tags
(
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

-- Table: event_tags (many-to-many relationship between events and tags)
CREATE TABLE IF NOT EXISTS event_tags
(
    event_id INTEGER NOT NULL,
    tag_id   INTEGER NOT NULL,
    PRIMARY KEY (event_id, tag_id),
    FOREIGN KEY (event_id) REFERENCES events (id),
    FOREIGN KEY (tag_id) REFERENCES tags (id)
);

CREATE TABLE IF NOT EXISTS yearly_summaries
(
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    year                 INTEGER NOT NULL UNIQUE,
    total_active_time    INTEGER NOT NULL,
    total_afk_time       INTEGER NOT NULL,
    most_active_app      TEXT,
    most_active_project  TEXT,
    most_active_language TEXT,
    metadata             TEXT,
    last_updated         TEXT    NOT NULL
);



