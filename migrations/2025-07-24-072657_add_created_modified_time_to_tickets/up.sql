-- Add created_time and modified_time columns to tickets table
-- SQLite doesn't support adding columns with non-constant defaults, so we recreate the table

-- Create new table with timestamp fields
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
    created_time DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_time DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(period)
);

-- Copy existing data with current timestamp for created_time and modified_time
INSERT INTO tickets_new (id, period, time, red1, red2, red3, red4, red5, red6, blue, created_time, modified_time)
SELECT id, period, time, red1, red2, red3, red4, red5, red6, blue, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
FROM tickets;

-- Drop old table and rename new table
DROP TABLE tickets;
ALTER TABLE tickets_new RENAME TO tickets;
