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
    timestamp     TEXT NOT NULL,
    duration      INTEGER DEFAULT 0,
    category_id INTEGER NOT NULL,
    app_id      INTEGER NOT NULL,
    entity_id     INTEGER,
    project_id    INTEGER,
    branch_id   INTEGER,
    language_id      INTEGER,
    end_timestamp TEXT,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
    FOREIGN KEY (app_id) REFERENCES apps (id) ON DELETE CASCADE,
    FOREIGN KEY (language_id) REFERENCES languages (id) ON DELETE CASCADE,
    FOREIGN KEY (branch_id) REFERENCES branches (id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS afk_events
(
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    afk_start TEXT NOT NULL,
    afk_end   TEXT,
    duration  INTEGER
);

CREATE TABLE IF NOT EXISTS heartbeats
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id    INTEGER,
    entity_id     INTEGER,
    branch_id     INTEGER,
    language_id   INTEGER,
    app_id        INTEGER NOT NULL,
    timestamp     TEXT NOT NULL,
    is_write      BOOLEAN DEFAULT FALSE,
    lines         INTEGER,
    cursorpos     INTEGER,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
    FOREIGN KEY (entity_id) REFERENCES entities (id) ON DELETE CASCADE,
    FOREIGN KEY (branch_id) REFERENCES branches (id) ON DELETE CASCADE,
    FOREIGN KEY (language_id) REFERENCES languages (id) ON DELETE CASCADE,
    FOREIGN KEY (app_id) REFERENCES apps (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS categories
(
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE VIRTUAL TABLE IF NOT EXISTS projects_fts
USING fts5(
    name,
    content='projects',
    content_rowid='id',
    tokenize = 'porter unicode61'
);

CREATE TRIGGER IF NOT EXISTS projects_ai
AFTER INSERT ON projects
BEGIN
    INSERT INTO projects_fts(rowid, name) VALUES (new.id, new.name);
END;

CREATE TRIGGER IF NOT EXISTS projects_au
AFTER UPDATE ON projects
BEGIN
    UPDATE projects_fts SET name = new.name WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS projects_ad
AFTER DELETE ON projects
BEGIN
    DELETE FROM projects_fts WHERE rowid = old.id;
END;



