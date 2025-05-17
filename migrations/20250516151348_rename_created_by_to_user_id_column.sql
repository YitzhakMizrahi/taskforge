-- Add migration script here

-- Ensure user_id has the values from created_by (for data migration)
UPDATE tasks SET user_id = created_by;

-- Drop the old created_by column
ALTER TABLE tasks DROP COLUMN created_by;
