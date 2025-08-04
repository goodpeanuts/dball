-- Insert test spot data for periods 2025084 and 2025086
-- These are example lottery number combinations for testing

-- Period 2025084 data (5 records)
INSERT INTO spot (period, red1, red2, red3, red4, red5, red6, blue, magnification, prize_status, created_time, modified_time) VALUES
('2025084', 2, 6, 7, 13, 16, 28, 11, 1, NULL, datetime('now'), datetime('now')),
('2025084', 4, 13, 15, 18, 22, 28, 16, 1, NULL, datetime('now'), datetime('now')),
('2025084', 9, 13, 15, 18, 19, 24, 16, 1, NULL, datetime('now'), datetime('now')),
('2025084', 3, 9, 20, 25, 26, 32, 8, 1, NULL, datetime('now'), datetime('now')),
('2025084', 12, 13, 22, 26, 29, 30, 10, 1, NULL, datetime('now'), datetime('now')),

-- Period 2025086 data (5 records)
('2025086', 8, 9, 19, 24, 26, 31, 3, 1, NULL, datetime('now'), datetime('now')),
('2025086', 3, 8, 10, 13, 23, 32, 13, 1, NULL, datetime('now'), datetime('now')),
('2025086', 9, 24, 27, 29, 31, 33, 9, 1, NULL, datetime('now'), datetime('now')),
('2025086', 4, 13, 20, 24, 26, 28, 15, 1, NULL, datetime('now'), datetime('now')),
('2025086', 8, 9, 10, 14, 18, 24, 14, 1, NULL, datetime('now'), datetime('now'));
