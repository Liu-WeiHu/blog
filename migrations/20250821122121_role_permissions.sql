-- Add migration script here
-- Creating table for RolePermission (junction table for roles and permissions)
CREATE TABLE role_permissions (
    role_id int4 NOT NULL,
    permission_id int4 NOT NULL,
    PRIMARY KEY (role_id, permission_id)
);

INSERT INTO role_permissions (role_id, permission_id) VALUES
(1, 1), -- view_post_creation
(1, 2), -- view_post_edit
(1, 3), -- view_post_detail
(1, 4), -- view_post_delete
(1, 5), -- create_post
(1, 6), -- edit_post
(1, 7), -- get_post
(1, 8), -- delete_post
(1, 9); -- list_post

INSERT INTO role_permissions (role_id, permission_id) VALUES
(2, 9); -- list_post

INSERT INTO role_permissions (role_id, permission_id) VALUES
(3, 1), -- view_post_creation
(3, 3), -- view_post_detail
(3, 5), -- create_post
(3, 7), -- get_post
(3, 9); -- list_post
