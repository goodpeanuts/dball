-- Add deprecated field to spot table
ALTER TABLE spot ADD COLUMN deprecated BOOLEAN NOT NULL DEFAULT FALSE;

-- Migrate existing deprecated spots (prize_status = -1) to use the new field
UPDATE spot SET deprecated = TRUE WHERE prize_status = -1;

-- Reset prize_status to NULL for deprecated spots
UPDATE spot SET prize_status = NULL WHERE deprecated = TRUE;
