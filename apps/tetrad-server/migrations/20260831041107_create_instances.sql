CREATE TABLE instances (
  singleton                 INTEGER PRIMARY KEY CHECK (singleton = 1),
  id                        TEXT NOT NULL UNIQUE,
  name                      TEXT NOT NULL,
  setup_completed_at_ms     INTEGER,
  created_at_ms             INTEGER NOT NULL
);
