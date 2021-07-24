-- Your SQL goes here
CREATE TABLE dictionary (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    chord TEXT NOT NULL UNIQUE,
    translation TEXT NOT NULL
);
INSERT INTO dictionary(chord, translation) VALUES ("SAP", "sap");

