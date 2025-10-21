-- Sample SQL commands for mini_sqlite_golang

-- Create and switch to a database
CREATE DATABASE demo;

-- Create a users table
CREATE TABLE users (id INT, name TEXT, email TEXT);

-- Insert some data
INSERT INTO users VALUES (1, 'Alice', 'alice@example.com');
INSERT INTO users VALUES (2, 'Bob', 'bob@example.com');
INSERT INTO users VALUES (3, 'Charlie', 'charlie@example.com');

-- Query all users
SELECT * FROM users;

-- Query with condition
SELECT name, email FROM users WHERE id = 2;

-- Create an index
CREATE INDEX users id;

-- Update a record
UPDATE users SET email = 'alice.smith@example.com' WHERE id = 1;

-- Add a column
ALTER TABLE users ADD COLUMN age INT;

-- Update with new column
UPDATE users SET age = 30 WHERE id = 1;
UPDATE users SET age = 25 WHERE id = 2;

-- Create another table for joins
CREATE TABLE orders (order_id INT, user_id INT, product TEXT);

INSERT INTO orders VALUES (101, 1, 'Laptop');
INSERT INTO orders VALUES (102, 1, 'Mouse');
INSERT INTO orders VALUES (103, 2, 'Keyboard');

-- Join query
SELECT users.name, orders.product FROM users INNER JOIN orders ON users.id = orders.user_id;

-- Delete a record
DELETE FROM users WHERE id = 3;

-- Commit changes
COMMIT;
