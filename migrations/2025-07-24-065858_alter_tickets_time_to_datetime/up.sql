-- Change time field from TEXT to DATETIME
-- SQLite doesn't support ALTER COLUMN directly, so we need to recreate the table

-- Create new table with datetime field
CREATE TABLE tickets_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    period TEXT NOT NULL,                
    time DATETIME NOT NULL,              -- Changed from TEXT to DATETIME
    red1 INTEGER NOT NULL,
    red2 INTEGER NOT NULL,
    red3 INTEGER NOT NULL,
    red4 INTEGER NOT NULL,
    red5 INTEGER NOT NULL,
    red6 INTEGER NOT NULL,
    blue INTEGER NOT NULL,
    UNIQUE(period)
);

-- Copy data from old table to new table, converting time format
INSERT INTO tickets_new (id, period, time, red1, red2, red3, red4, red5, red6, blue)
SELECT id, period, datetime(time) as time, red1, red2, red3, red4, red5, red6, blue
FROM tickets;

-- Drop old table and rename new table
DROP TABLE tickets;
ALTER TABLE tickets_new RENAME TO tickets;
