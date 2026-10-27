-- Migration 1: base schema. Applied inside a transaction by db::migrate.
CREATE TABLE platforms (
  id   TEXT PRIMARY KEY,           -- stable id, e.g. 'snes', 'ps1', 'windows'
  name TEXT NOT NULL
);

CREATE TABLE games (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  title         TEXT NOT NULL,
  platform_id   TEXT REFERENCES platforms(id),
  file_path     TEXT NOT NULL,     -- path as chosen by the user (reference, not copy)
  file_storage  TEXT NOT NULL DEFAULT 'reference'
                CHECK (file_storage IN ('reference','copied','moved')),
  version       TEXT,
  favorite      INTEGER NOT NULL DEFAULT 0 CHECK (favorite IN (0,1)),
  added_at      TEXT NOT NULL DEFAULT (datetime('now')),
  last_played_at TEXT,
  playtime_seconds INTEGER NOT NULL DEFAULT 0,
  UNIQUE (file_path)
);
CREATE INDEX idx_games_title ON games(title);
CREATE INDEX idx_games_platform ON games(platform_id);

CREATE TABLE components (
  id               TEXT PRIMARY KEY,   -- stable catalog id
  display_name     TEXT NOT NULL,
  kind             TEXT NOT NULL CHECK (kind IN ('emulator','core','tool','port','recompilation')),
  installed_version TEXT,
  install_path     TEXT,
  status           TEXT NOT NULL DEFAULT 'not_installed',
  integrity        TEXT NOT NULL DEFAULT 'unverified',
  source_url       TEXT,
  license          TEXT
);
