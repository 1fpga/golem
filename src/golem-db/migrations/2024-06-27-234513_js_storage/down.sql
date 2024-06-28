-- This file should undo anything in `up.sql`

CREATE TABLE `storage_temp`
(
    `id`    INT          NOT NULL PRIMARY KEY,
    `key`   VARCHAR(255) NOT NULL,
    `value` TEXT         NOT NULL
);

CREATE UNIQUE INDEX storage_key_idx ON storage (`key`);

INSERT INTO `storage_temp` (`key`, `value`)
SELECT `key`, `value`
FROM `storage`;

DROP TABLE `storage`;

ALTER TABLE `storage_temp`
    RENAME TO `storage`;

