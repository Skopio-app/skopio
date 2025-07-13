CREATE TABLE IF NOT EXISTS heartbeats
(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
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
    timestamp TEXT NOT NULL,
    duration INTEGER DEFAULT 0,
    category TEXT,
    app_name TEXT NOT NULL,
    entity_name TEXT,
    entity_type TEXT,
    project_name TEXT,
    project_path TEXT,
    branch_name TEXT,
    language_name TEXT,
    end_timestamp TEXT NOT NULL,
    synced BOOL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS afk_events
(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    afk_start TEXT NOT NULL,
    afk_end TEXT,
    duration INTEGER DEFAULT 0,
    synced BOOL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS goals
(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    target_seconds INTEGER NOT NULL,
    time_span TEXT NOT NULL,
    use_apps BOOLEAN NOT NULL DEFAULT 0,
    use_categories BOOLEAN NOT NULL DEFAULT 0,
    ignore_no_activity_days BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS goal_apps
(
    goal_id INTEGER NOT NULL,
    app TEXT NOT NULL,
    PRIMARY KEY (goal_id, app),
    FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE
);

CREATE TABLE goal_categories
(
    goal_id INTEGER NOT NULL,
    category TEXT NOT NULL,
    PRIMARY KEY (goal_id, category),
    FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS goal_excluded_days
(
    goal_id INTEGER NOT NULL,
    day TEXT NOT NULL,
    PRIMARY KEY (goal_id, day),
    FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS shown_goal_notifications
(
    goal_id INTEGER NOT NULL,
    time_span TEXT NOT NULL,
    period_key TEXT NOT NULL,
    shown_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (goal_id, time_span, period_key)
);

