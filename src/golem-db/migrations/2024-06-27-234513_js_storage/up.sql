-- Your SQL goes here
CREATE TABLE `storage_temp`
(
    `key`   VARCHAR(255) PRIMARY KEY,
    `value` TEXT NOT NULL
);

INSERT INTO `storage_temp` (`key`, `value`)
SELECT `key`, `value`
FROM `storage`;

DROP TABLE `storage`;

ALTER TABLE `storage_temp`
    RENAME TO `storage`;
