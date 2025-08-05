-- 创建性别枚举类型
CREATE TYPE gender_type AS ENUM ('male', 'female', 'other');

-- 创建学历枚举类型
CREATE TYPE education_type AS ENUM ('primary', 'secondary', 'bachelor', 'master', 'doctorate', 'other');

-- 创建 user_ext 表
CREATE TABLE user_ext (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE,
    age INTEGER,
    gender gender_type,
    education education_type,
    hometown VARCHAR(100),
    address VARCHAR(255)
);
