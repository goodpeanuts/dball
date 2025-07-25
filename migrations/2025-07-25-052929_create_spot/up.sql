-- Create spot table for lottery drawing results
CREATE TABLE spot (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    period TEXT NOT NULL,
    red1 INTEGER NOT NULL,
    red2 INTEGER NOT NULL,
    red3 INTEGER NOT NULL,
    red4 INTEGER NOT NULL,
    red5 INTEGER NOT NULL,
    red6 INTEGER NOT NULL,
    blue INTEGER NOT NULL,
    magnification INTEGER NOT NULL,
    prize_status INTEGER NULL,
    created_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert initial data
INSERT INTO spot (period, red1, red2, red3, red4, red5, red6, blue, magnification, prize_status) VALUES
(2025084, 2, 6, 7, 13, 16, 28, 11, 1, NULL),
(2025084, 4, 13, 15, 18, 22, 28, 16, 1, NULL),
(2025084, 9, 13, 15, 18, 19, 24, 16, 1, NULL),
(2025084, 3, 9, 20, 25, 26, 32, 8, 1, NULL),
(2025084, 12, 13, 22, 26, 29, 30, 10, 1, NULL);
