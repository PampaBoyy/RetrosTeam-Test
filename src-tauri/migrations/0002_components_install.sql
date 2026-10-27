-- Migration 2: installation details needed by the component manager.
-- Paths are stored RELATIVE to the data root so the whole folder can be moved between PCs.
ALTER TABLE components ADD COLUMN exe_path TEXT;
ALTER TABLE components ADD COLUMN sha256 TEXT;
ALTER TABLE components ADD COLUMN installed_at TEXT;
