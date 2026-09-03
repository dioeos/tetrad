CREATE TABLE user_profiles_new (
  id              INTEGER PRIMARY KEY,
  torii_user_id   TEXT NOT NULL UNIQUE,
  external_id     TEXT NOT NULL UNIQUE,
  display_name    TEXT,
  created_at_ms   INTEGER NOT NULL,
  updated_at_ms   INTEGER NOT NULL,

  FOREIGN KEY (torii_user_id) REFERENCES users(id) ON DELETE CASCADE
);

INSERT INTO user_profiles_new (
  id,
  torii_user_id,
  external_id,
  display_name,
  created_at_ms,
  updated_at_ms
)
SELECT
  id,
  torii_user_id,
  external_id,
  display_name,
  created_at_ms,
  updated_at_ms
FROM user_profiles;

DROP TABLE user_profiles;

ALTER TABLE user_profiles_new RENAME TO user_profiles;
