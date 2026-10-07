CREATE TABLE instances (
  id                        INTEGER PRIMARY KEY CHECK (id = 1),
  uuid                      TEXT NOT NULL UNIQUE,
  name                      TEXT NOT NULL,
  setup_completed_at_ms     INTEGER,
  created_at_ms             INTEGER NOT NULL,
  updated_at_ms             INTEGER NOT NULL
);
