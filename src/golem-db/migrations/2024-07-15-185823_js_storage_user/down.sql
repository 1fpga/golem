-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS storage_key_username;

ALTER TABLE `storage`
    DROP COLUMN `username`;
