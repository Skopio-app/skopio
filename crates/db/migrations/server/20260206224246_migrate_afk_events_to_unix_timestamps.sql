CREATE TABLE afk_events_new (
    id          BLOB(16) NOT NULL PRIMARY KEY,
    afk_start   INTEGER NOT NULL,
    afk_end     INTEGER,
    duration    INTEGER
);

INSERT INTO afk_events_new(id, afk_start, afk_end, duration)
SELECT
    id,
    CAST(strftime('%s', afk_start) AS INTEGER),
    CASE WHEN afk_end IS NOT NULL THEN CAST(strftime('%s', afk_end) AS INTEGER) ELSE NULL END,
    duration
FROM afk_events;

DROP TABLE afk_events;
ALTER TABLE afk_events_new RENAME TO afk_events;

CREATE INDEX IF NOT EXISTS idx_afk_start ON afk_events(afk_start);
CREATE INDEX IF NOT EXISTS idx_afk_end ON afk_events(afk_end);
