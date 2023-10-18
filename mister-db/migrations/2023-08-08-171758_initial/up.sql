CREATE TABLE cores
(
    id            INTEGER PRIMARY KEY NOT NULL,
    name          VARCHAR             NOT NULL,
    slug          VARCHAR             NOT NULL,
    version       VARCHAR             NOT NULL,
    path          VARCHAR             NOT NULL,
    author        VARCHAR             NOT NULL,
    description   VARCHAR             NOT NULL,
    released_at   TIMESTAMP           NOT NULL,
    last_played   TIMESTAMP,
    favorite      BOOLEAN             NOT NULL DEFAULT FALSE,
    downloaded_at TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX cores_name_version_idx ON cores (name, version);

CREATE TABLE games
(
    id            INTEGER PRIMARY KEY NOT NULL,
    name          VARCHAR             NOT NULL,
    slug          VARCHAR             NOT NULL,
    core_id       INTEGER             NOT NULL REFERENCES cores (id) ON DELETE CASCADE,
    version       VARCHAR             NOT NULL,
    path          VARCHAR             NOT NULL,
    description   VARCHAR             NOT NULL,
    last_played   TIMESTAMP,
    favorite      BOOLEAN             NOT NULL DEFAULT FALSE,
    released_at   TIMESTAMP           NOT NULL,
    downloaded_at TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP
);

