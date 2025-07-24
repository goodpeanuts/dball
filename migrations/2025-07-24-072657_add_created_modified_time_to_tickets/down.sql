-- Remove created_time and modified_time columns from tickets table
-- Recreate table without timestamp fields

-- Create table without timestamp fields (original structure)
CREATE TABLE tickets_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    period TEXT NOT NULL,                
    time DATETIME NOT NULL,
    red1 INTEGER NOT NULL,
    red2 INTEGER NOT NULL,
    red3 INTEGER NOT NULL,
    red4 INTEGER NOT NULL,
    red5 INTEGER NOT NULL,
    red6 INTEGER NOT NULL,
    blue INTEGER NOT NULL,
    UNIQUE(period)
);

-- Copy data without timestamp fields
INSERT INTO tickets_new (id, period, time, red1, red2, red3, red4, red5, red6, blue)
SELECT id, period, time, red1, red2, red3, red4, red5, red6, blue
FROM tickets;

-- Drop current table and rename new table
DROP TABLE tickets;
ALTER TABLE tickets_new RENAME TO tickets;
