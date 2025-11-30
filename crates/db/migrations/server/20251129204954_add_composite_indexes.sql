CREATE INDEX IF NOT EXISTS idx_events_time_range ON events(timestamp, end_timestamp);

CREATE INDEX IF NOT EXISTS idx_events_project_time ON events(project_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_events_app_time ON events(app_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_events_category_time ON events(category_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_events_entity_time ON events(entity_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_events_branch_time ON events(branch_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_events_language_time ON events(language_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_events_source_time ON events(source_id, timestamp);

DROP INDEX IF EXISTS idx_events_end;
DROP INDEX IF EXISTS idx_events_timestamp;

