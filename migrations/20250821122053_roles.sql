-- Add migration script here
-- Creating table for Role
CREATE TABLE roles (
    id int4 PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT
);

INSERT INTO roles (id, name, description) VALUES
(1, 'Admin', 'Administrator with full access to post-related actions'),
(2, 'Anonymous User', 'Anonymous user with limited access'),
(3, 'LoggedIn User', 'Logged-in user with full access to post-related actions');
