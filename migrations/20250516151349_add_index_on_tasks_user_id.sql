-- Add migration script here
CREATE INDEX idx_tasks_user_id ON tasks(user_id);
