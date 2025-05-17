-- Add migration script here

-- Add user_id column to tasks table
ALTER TABLE tasks
ADD COLUMN user_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL;
