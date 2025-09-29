CREATE TABLE apps_new (
    id   BLOB(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    last_updated INTEGER
);
INSERT INTO apps_new (id, name, last_updated)
SELECT id, name, unixepoch() FROM apps;
DROP TABLE apps;
ALTER TABLE apps_new RENAME TO apps;

CREATE TABLE projects_new (
    id        BLOB(16) NOT NULL PRIMARY KEY,
    name      TEXT NOT NULL UNIQUE,
    root_path TEXT,
    last_updated INTEGER
);
INSERT INTO projects_new (id, name, root_path, last_updated)
SELECT id, name, root_path, unixepoch() FROM projects;
DROP TABLE projects;
ALTER TABLE projects_new RENAME TO projects;

CREATE TABLE languages_new (
    id   BLOB(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    last_updated INTEGER
);
INSERT INTO languages_new (id, name, last_updated)
SELECT id, name, unixepoch() FROM languages;
DROP TABLE languages;
ALTER TABLE languages_new RENAME TO languages;

CREATE TABLE categories_new (
    id   BLOB(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    last_updated INTEGER
);
INSERT INTO categories_new (id, name, last_updated)
SELECT id, name, unixepoch() FROM categories;
DROP TABLE categories;
ALTER TABLE categories_new RENAME TO categories;

CREATE TABLE sources_new (
    id   BLOB(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    last_updated INTEGER
);
INSERT INTO sources_new (id, name, last_updated)
SELECT id, name, unixepoch() FROM sources;
DROP TABLE sources;
ALTER TABLE sources_new RENAME TO sources;

CREATE TABLE branches_new (
    id         BLOB(16) NOT NULL PRIMARY KEY,
    project_id BLOB(16) NOT NULL,
    name       TEXT NOT NULL,
    last_updated INTEGER,
    UNIQUE (project_id, name),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);
INSERT INTO branches_new (id, project_id, name, last_updated)
SELECT id, project_id, name, unixepoch() FROM branches;
DROP TABLE branches;
ALTER TABLE branches_new RENAME TO branches;

CREATE TABLE entities_new (
    id         BLOB(16) NOT NULL PRIMARY KEY,
    project_id BLOB(16) NOT NULL,
    name       TEXT NOT NULL,
    type       TEXT,
    last_updated INTEGER,
    UNIQUE (project_id, name),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);
INSERT INTO entities_new (id, project_id, name, type, last_updated)
SELECT id, project_id, name, type, unixepoch() FROM entities;
DROP TABLE entities;
ALTER TABLE entities_new RENAME TO entities;

CREATE TABLE events_new (
    id            BLOB(16) NOT NULL PRIMARY KEY,
    timestamp     INTEGER NOT NULL,
    duration      INTEGER DEFAULT 0,
    category_id   BLOB(16) NOT NULL,
    app_id        BLOB(16) NOT NULL,
    entity_id     BLOB(16),
    project_id    BLOB(16),
    branch_id     BLOB(16),
    language_id   BLOB(16),
    source_id     BLOB(16) NOT NULL,
    end_timestamp INTEGER,
    FOREIGN KEY (project_id)  REFERENCES projects (id)  ON DELETE CASCADE,
    FOREIGN KEY (app_id)      REFERENCES apps (id)      ON DELETE CASCADE,
    FOREIGN KEY (language_id) REFERENCES languages (id) ON DELETE CASCADE,
    FOREIGN KEY (branch_id)   REFERENCES branches (id)  ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE CASCADE,
    FOREIGN KEY (source_id)   REFERENCES sources (id)   ON DELETE CASCADE
);
INSERT INTO events_new
SELECT id,
       CAST(strftime('%s', timestamp) AS INTEGER),
       duration,
       category_id,
       app_id,
       entity_id,
       project_id,
       branch_id,
       language_id,
       source_id,
       CASE
          WHEN end_timestamp IS NOT NULL
          THEN CAST(strftime('%s', end_timestamp) AS INTEGER)
          ELSE NULL
       END
FROM events;
DROP TABLE events;
ALTER TABLE events_new RENAME TO events;

CREATE TABLE afk_events_new (
    id        BLOB(16) NOT NULL PRIMARY KEY,
    afk_start TEXT NOT NULL,
    afk_end   TEXT,
    duration  INTEGER
);
INSERT INTO afk_events_new SELECT id, afk_start, afk_end, duration FROM afk_events;
DROP TABLE afk_events;
ALTER TABLE afk_events_new RENAME TO afk_events;

DROP INDEX IF EXISTS idx_branches_project;
DROP INDEX IF EXISTS idx_entities_project;
DROP INDEX IF EXISTS idx_entities_project;
DROP INDEX IF EXISTS idx_entities_project;
DROP INDEX IF EXISTS idx_events_branch;
DROP INDEX IF EXISTS idx_events_language;
DROP INDEX IF EXISTS idx_events_project;
DROP INDEX IF EXISTS idx_events_app;
DROP INDEX IF EXISTS idx_events_category;
DROP INDEX IF EXISTS idx_events_source;

CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
CREATE INDEX IF NOT EXISTS idx_events_end ON events(end_timestamp);


CREATE TRIGGER IF NOT EXISTS projects_ai
AFTER INSERT ON projects
BEGIN
    INSERT INTO projects_fts(rowid, name) VALUES (NULL, new.name);
    INSERT INTO projects_fts_map(docid, project_id)
    VALUES (last_insert_rowid(), new.id);
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
