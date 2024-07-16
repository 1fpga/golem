-- Your SQL goes here
ALTER TABLE `storage`
    ADD COLUMN `username` VARCHAR(255);

CREATE UNIQUE INDEX storage_key_username ON `storage` (`key`, `username`);
