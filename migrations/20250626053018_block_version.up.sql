ALTER TABLE blocks ADD COLUMN version INTEGER DEFAULT 2;

UPDATE blocks
SET version = 1
WHERE version IS NULL;
