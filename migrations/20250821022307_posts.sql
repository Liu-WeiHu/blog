-- Add migration script here
create table posts (
    id SERIAL primary key,
    title VARCHAR(200) not null,
    content TEXT not null,
    user_id INTEGER not null,
    status VARCHAR(10) not null default 'published',
    created_at timestamp default CURRENT_TIMESTAMP null
);
