-- Add migration script here
-- Creating table for UserRole (junction table for users and roles)
CREATE TABLE user_roles (
    user_id int4 NOT NULL,
    role_id int4 NOT NULL,
    PRIMARY KEY (user_id, role_id)
);
