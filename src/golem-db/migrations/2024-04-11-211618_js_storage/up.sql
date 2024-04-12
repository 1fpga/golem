CREATE TABLE `storage`
(
    `id`    INT          NOT NULL PRIMARY KEY,
    `key`   VARCHAR(255) NOT NULL,
    `value` TEXT         NOT NULL
);

CREATE UNIQUE INDEX storage_key_idx ON storage (`key`);
