CREATE TABLE user_profiles (
  id              INTEGER PRIMARY KEY,
  torii_user_id   TEXT NOT NULL UNIQUE,
  external_id     TEXT NOT NULL UNIQUE,
  display_name    TEXT NOT NULL,
  created_at_ms   INTEGER NOT NULL,
  updated_at_ms   INTEGER NOT NULL,

  FOREIGN KEY (torii_user_id) REFERENCES users(id) ON DELETE CASCADE
);
