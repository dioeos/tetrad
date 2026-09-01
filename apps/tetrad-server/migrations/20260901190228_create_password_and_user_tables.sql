CREATE TABLE users (
    id                  INTEGER PRIMARY KEY,
    external_id         TEXT NOT NULL UNIQUE,
    username            TEXT NOT NULL,
    normalized_username TEXT NOT NULL UNIQUE,
    created_at_ms       INTEGER NOT NULL,
    updated_at_ms       INTEGER NOT NULL
);

CREATE TABLE password_credentials (
    user_id       INTEGER PRIMARY KEY
        REFERENCES users(id) ON DELETE CASCADE,
    password_hash TEXT NOT NULL,
    updated_at_ms INTEGER NOT NULL
);

