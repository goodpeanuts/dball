-- Revert time field from DATETIME back to TEXT
-- SQLite doesn't support ALTER COLUMN directly, so we need to recreate the table

-- Create table with TEXT time field (original structure)
CREATE TABLE tickets_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    period TEXT NOT NULL,                
    time TEXT NOT NULL,                  -- Reverted back to TEXT
    red1 INTEGER NOT NULL,
    red2 INTEGER NOT NULL,
    red3 INTEGER NOT NULL,
    red4 INTEGER NOT NULL,
    red5 INTEGER NOT NULL,
    red6 INTEGER NOT NULL,
    blue INTEGER NOT NULL,
    UNIQUE(period)
);

-- Copy data from current table to new table, converting datetime back to text
INSERT INTO tickets_new (id, period, time, red1, red2, red3, red4, red5, red6, blue)
SELECT id, period, strftime('%Y-%m-%d %H:%M:%S', time) as time, red1, red2, red3, red4, red5, red6, blue
FROM tickets;

-- Drop current table and rename new table
DROP TABLE tickets;
ALTER TABLE tickets_new RENAME TO tickets;
