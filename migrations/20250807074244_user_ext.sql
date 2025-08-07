-- Add migration script here

-- 创建 user_ext 表
CREATE TABLE user_ext (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE,
    age INTEGER,
    gender varchar(50),
    education varchar(50),
    hometown VARCHAR(100),
    address VARCHAR(255)
);

