CREATE TABLE IF NOT EXISTS messages
(
	id INT PRIMARY KEY NOT NULL,
	message TEXT NOT NULL
);

INSERT INTO messages(id, message) VALUES(1, "Hello world!");
INSERT INTO messages(id, message) VALUES(2, "Hello galaxy!");
INSERT INTO messages(id, message) VALUES(3, "Hello universe!");