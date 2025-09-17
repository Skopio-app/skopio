DROP INDEX IF EXISTS idx_events_category;
CREATE INDEX IF NOT EXISTS idx_events_category ON events(category_id);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
CREATE INDEX IF NOT EXISTS idx_events_end ON events(end_timestamp);
CREATE INDEX IF NOT EXISTS idx_events_time ON events(end_timestamp, timestamp);
