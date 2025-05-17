-- Re-add the created_by column (nullable initially)
ALTER TABLE tasks ADD COLUMN created_by INTEGER;

-- Copy data from user_id back to created_by
UPDATE tasks SET created_by = user_id;

-- Make created_by NOT NULL
ALTER TABLE tasks ALTER COLUMN created_by SET NOT NULL;

-- Re-add the foreign key constraint
ALTER TABLE tasks ADD CONSTRAINT tasks_created_by_fkey FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE;

-- Re-create the index for created_by
CREATE INDEX idx_tasks_created_by ON tasks(created_by); 