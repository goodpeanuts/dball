-- Restore deprecated spots to use prize_status = -1
UPDATE spot SET prize_status = -1 WHERE deprecated = TRUE;

-- Remove deprecated field
ALTER TABLE spot DROP COLUMN deprecated;
