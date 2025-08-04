-- Remove the test spot data inserted in up.sql
-- Delete records by their unique combinations of period and numbers

-- Delete Period 2025084 records
DELETE FROM spot WHERE period = '2025084' AND red1 = 2 AND red2 = 6 AND red3 = 7 AND red4 = 13 AND red5 = 16 AND red6 = 28 AND blue = 11;
DELETE FROM spot WHERE period = '2025084' AND red1 = 4 AND red2 = 13 AND red3 = 15 AND red4 = 18 AND red5 = 22 AND red6 = 28 AND blue = 16;
DELETE FROM spot WHERE period = '2025084' AND red1 = 9 AND red2 = 13 AND red3 = 15 AND red4 = 18 AND red5 = 19 AND red6 = 24 AND blue = 16;
DELETE FROM spot WHERE period = '2025084' AND red1 = 3 AND red2 = 9 AND red3 = 20 AND red4 = 25 AND red5 = 26 AND red6 = 32 AND blue = 8;
DELETE FROM spot WHERE period = '2025084' AND red1 = 12 AND red2 = 13 AND red3 = 22 AND red4 = 26 AND red5 = 29 AND red6 = 30 AND blue = 10;

-- Delete Period 2025086 records
DELETE FROM spot WHERE period = '2025086' AND red1 = 8 AND red2 = 9 AND red3 = 19 AND red4 = 24 AND red5 = 26 AND red6 = 31 AND blue = 3;
DELETE FROM spot WHERE period = '2025086' AND red1 = 3 AND red2 = 8 AND red3 = 10 AND red4 = 13 AND red5 = 23 AND red6 = 32 AND blue = 13;
DELETE FROM spot WHERE period = '2025086' AND red1 = 9 AND red2 = 24 AND red3 = 27 AND red4 = 29 AND red5 = 31 AND red6 = 33 AND blue = 9;
DELETE FROM spot WHERE period = '2025086' AND red1 = 4 AND red2 = 13 AND red3 = 20 AND red4 = 24 AND red5 = 26 AND red6 = 28 AND blue = 15;
DELETE FROM spot WHERE period = '2025086' AND red1 = 8 AND red2 = 9 AND red3 = 10 AND red4 = 14 AND red5 = 18 AND red6 = 24 AND blue = 14;
