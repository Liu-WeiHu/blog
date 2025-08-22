-- Add migration script here
-- Creating table for Permission
CREATE TABLE permissions (
    id int4 PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    permission_type VARCHAR(20) NOT NULL DEFAULT 'operation',
    description TEXT
);

INSERT INTO permissions (id,"name",permission_type,description) VALUES
	 (5,'create_post','operation','Permission to create a post'),
	 (6,'edit_post','operation','Permission to edit a post'),
	 (7,'get_post','operation','Permission to retrieve a post'),
	 (8,'delete_post','operation','Permission to delete a post'),
	 (9,'list_post','operation','Permission to list posts'),
	 (1,'view_post_creation','visual','Permission to view post creation page'),
	 (2,'view_post_edit','visual','Permission to view post edit page'),
	 (3,'view_post_detail','visual','Permission to view post details'),
	 (4,'view_post_delete','visual','Permission to view post deletion page');

