-- Your SQL goes here

CREATE TABLE cores
(
    id            INTEGER PRIMARY KEY NOT NULL,
    name          VARCHAR             NOT NULL,
    version       VARCHAR             NOT NULL,
    path          VARCHAR             NOT NULL,
    author        VARCHAR             NOT NULL,
    description   VARCHAR             NOT NULL,
    released_at   TIMESTAMP           NOT NULL,
    downloaded_at TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX cores_name_version_idx ON cores (name, version);
