CREATE TABLE IF NOT EXISTS projects
(
    id        BLOB(16) NOT NULL PRIMARY KEY,
    name      TEXT NOT NULL UNIQUE,
    root_path TEXT
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS branches
(
    id BLOB(16) NOT NULL PRIMARY KEY,
    project_id BLOB(16) NOT NULL,
    name TEXT NOT NULL,
    UNIQUE (project_id, name)
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS languages
(
    id BLOB(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS apps
(
    id BLOB(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS entities
(
    id BLOB(16) NOT NULL PRIMARY KEY,
    project_id BLOB(16) NOT NULL,
    name TEXT NOT NULL,
    type TEXT,
    UNIQUE (project_id, name),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS events
(
    id            BLOB(16) NOT NULL PRIMARY KEY,
    timestamp     TEXT NOT NULL,
    duration      INTEGER DEFAULT 0,
    category_id   BLOB(16) NOT NULL,
    app_id        BLOB(16) NOT NULL,
    entity_id     BLOB(16),
    project_id    BLOB(16),
    branch_id     BLOB(16),
    language_id   BLOB(16),
    source_id     BLOB(16) NOT NULL,
    end_timestamp TEXT,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
    FOREIGN KEY (app_id) REFERENCES apps (id) ON DELETE CASCADE,
    FOREIGN KEY (language_id) REFERENCES languages (id) ON DELETE CASCADE,
    FOREIGN KEY (branch_id) REFERENCES branches (id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE CASCADE,
    FOREIGN KEY (source_id) REFERENCES sources (id) ON DELETE CASCADE
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS afk_events
(
    id        BLOB(16) NOT NULL PRIMARY KEY,
    afk_start TEXT NOT NULL,
    afk_end   TEXT,
    duration  INTEGER
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS categories
(
    id   BLOB(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS sources
(
    id   BLOB(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS projects_fts_map
(
    docid   INTEGER PRIMARY KEY,
    project_id BLOB(16) NOT NULL UNIQUE,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE IF NOT EXISTS projects_fts
USING fts5(
    name,
    tokenize = 'porter unicode61'
);

CREATE TRIGGER IF NOT EXISTS projects_ai
AFTER INSERT ON projects
BEGIN
    INSERT INTO projects_fts(rowid, name) VALUES (NULL, new.name);
    INSERT INTO projects_fts_map(docid, project_id) VALUES (last_insert_rowid(), new.id);
END;

CREATE TRIGGER IF NOT EXISTS projects_au
AFTER UPDATE ON projects
BEGIN
    UPDATE projects_fts
        SET name = new.name
    WHERE rowid = (SELECT docid FROM projects_fts_map WHERE project_id = old.id);
END;

CREATE TRIGGER IF NOT EXISTS projects_ad
AFTER DELETE ON projects
BEGIN
    DELETE FROM projects_fts
        WHERE rowid = (SELECT docid FROM projects_fts_map WHERE project_id = old.id);
    DELETE FROM projects_fts_map WHERE project_id = old.id;
END;

CREATE INDEX IF NOT EXISTS idx_branches_project ON branches(project_id);
CREATE INDEX IF NOT EXISTS idx_entities_project ON entities(project_id);

CREATE INDEX IF NOT EXISTS idx_events_project ON events(project_id);
CREATE INDEX IF NOT EXISTS idx_events_entity ON events(entity_id);
CREATE INDEX IF NOT EXISTS idx_events_branch ON events(branch_id);
CREATE INDEX IF NOT EXISTS idx_events_language ON events(language_id);
CREATE INDEX IF NOT EXISTS idx_events_app ON events(app_id);
CREATE INDEX IF NOT EXISTS idx_events_category ON events(timestamp);
CREATE INDEX IF NOT EXISTS idx_events_source ON events(source_id);



