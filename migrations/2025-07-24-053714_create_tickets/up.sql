-- Your SQL goes here
CREATE TABLE tickets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    period TEXT NOT NULL,                
    time TEXT NOT NULL,                  
    red1 INTEGER NOT NULL,
    red2 INTEGER NOT NULL,
    red3 INTEGER NOT NULL,
    red4 INTEGER NOT NULL,
    red5 INTEGER NOT NULL,
    red6 INTEGER NOT NULL,
    blue INTEGER NOT NULL,
    UNIQUE(period)                       
);
