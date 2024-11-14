CREATE TABLE users
(
    id         INTEGER PRIMARY KEY,
    username   VARCHAR(255) NOT NULL UNIQUE,
    password   VARCHAR(255),
    created_at TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    admin      BOOLEAN      NOT NULL DEFAULT FALSE
);

CREATE TABLE user_storage
(
    id         INTEGER PRIMARY KEY,
    user_id    INTEGER      NOT NULL REFERENCES users (id),
    key        VARCHAR(255) NOT NULL,
    value      JSON         NOT NULL,
    updated_at TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT user_storage_user_id_key UNIQUE (user_id, key)
);

CREATE TABLE global_storage
(
    id         INTEGER PRIMARY KEY,
    key        VARCHAR(255) NOT NULL UNIQUE,
    value      JSON         NOT NULL,
    updated_at TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_cores
(
    id              INTEGER PRIMARY KEY,
    user_id         INTEGER   NOT NULL REFERENCES users (id),
    catalog_core_id INTEGER   NOT NULL REFERENCES cores (id),
    favorite        BOOLEAN   NOT NULL DEFAULT FALSE,
    last_played_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- The list of all games available on the filesystem, or cores that should
-- be shown in the games list.
CREATE TABLE games
(
    id       INTEGER PRIMARY KEY,
    name     TEXT,
    games_id INTEGER REFERENCES games_identification (id),
    core_id  INTEGER REFERENCES cores (id),
    path     TEXT,
    size     INTEGER,
    sha256   VARCHAR(64),
    -- If the game is a core, it must have a name.
    CONSTRAINT core_has_name
        CHECK ( (core_id IS NOT NULL AND name IS NOT NULL)
            OR (core_id IS NULL) ),
    -- If path is specified, size and sha256 must be specified as well.
    CONSTRAINT path_size_sha256
        CHECK ( (path IS NOT NULL AND size IS NOT NULL AND sha256 IS NOT NULL)
            OR (path IS NULL AND size IS NULL AND sha256 IS NULL) )
);

-- Record information about the games that a user has played.
-- This is not the database of all games available on the system,
-- but rather the games that a user has played.
CREATE TABLE user_games
(
    id             INTEGER PRIMARY KEY,
    user_id        INTEGER NOT NULL REFERENCES users (id),
    games_id       INTEGER NOT NULL REFERENCES games (id),
    -- If the user selected a core for it. Otherwise the core will be default.
    cores_id       INTEGER REFERENCES cores (id),
    favorite       BOOLEAN NOT NULL DEFAULT FALSE,
    last_played_at TIMESTAMP        DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT user_games_user_id_games_id UNIQUE (user_id, games_id)
);

CREATE TABLE savestates
(
    id              INTEGER PRIMARY KEY,
    core_id         INTEGER   NOT NULL REFERENCES cores (id),
    game_id         INTEGER   NOT NULL REFERENCES user_games (id),
    user_id         INTEGER   NOT NULL,
    state_path      TEXT      NOT NULL,
    screenshot_path TEXT      NOT NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

--------------------------------------------------------------------------------
-- CATALOG DATABASE

CREATE TABLE tags
(
    id   INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE catalogs
(
    id               INTEGER PRIMARY KEY,
    name             VARCHAR(255) NOT NULL UNIQUE,
    url              TEXT         NOT NULL UNIQUE,
    -- The last time this was updated.
    latest_update_at TIMESTAMP             DEFAULT CURRENT_TIMESTAMP,
    -- The `lastUpdated` field from the JSON.
    last_updated     VARCHAR(255),
    -- The `version` field.
    version          VARCHAR(255),
    priority         INTEGER      NOT NULL DEFAULT 0,
    update_pending   BOOLEAN               DEFAULT FALSE
);

CREATE TABLE systems
(
    id          INTEGER PRIMARY KEY,
    catalog_id  INTEGER      NOT NULL REFERENCES catalogs (id),
    name        VARCHAR(255) NOT NULL,
    unique_name VARCHAR(255) NOT NULL UNIQUE,
    category    VARCHAR(255),
    description TEXT
);

CREATE TABLE systems_tags
(
    id        INTEGER PRIMARY KEY,
    system_id INTEGER NOT NULL REFERENCES systems (id),
    tag_id    INTEGER NOT NULL REFERENCES tags (id),
    CONSTRAINT systems_tags_unique UNIQUE (system_id, tag_id)
);

-- Games that can be identified by their files (see `catalog_games_db_files`).
CREATE TABLE games_identification
(
    id          INTEGER PRIMARY KEY,
    system_id   INTEGER      NOT NULL REFERENCES systems (id),
    catalog_id  INTEGER      NOT NULL REFERENCES catalogs (id),
    name        VARCHAR(255) NOT NULL,
    shortname   VARCHAR(255),
    region      VARCHAR(255),
    languages   VARCHAR(255),
    description TEXT,
    CONSTRAINT catalog_games_db_unique UNIQUE (system_id, catalog_id, name, region, languages)
);

-- Files that are related to a game.
CREATE TABLE games_identification_files
(
    id         INTEGER PRIMARY KEY,
    games_id   INTEGER      NOT NULL REFERENCES games_identification (id),
    -- Keep the catalog_id for when we need to update the catalog and the identification.
    catalog_id INTEGER      NOT NULL REFERENCES catalogs (id),
    extension  VARCHAR(255) NOT NULL,
    size       INTEGER      NOT NULL,
    sha256     VARCHAR(64)  NOT NULL,
    CONSTRAINT catalog_games_db_files_size_hash UNIQUE (extension, size, sha256)
);

-- Installed cores.
CREATE TABLE cores
(
    id          INTEGER PRIMARY KEY,
    catalog_id  INTEGER      NOT NULL REFERENCES catalogs (id),
    name        VARCHAR(255) NOT NULL,
    unique_name VARCHAR(255) NOT NULL UNIQUE,
    rbf_path    TEXT,
    description TEXT,
    version     VARCHAR(255)
);

CREATE TABLE cores_tags
(
    id      INTEGER PRIMARY KEY,
    core_id INTEGER NOT NULL REFERENCES cores (id),
    tag_id  INTEGER NOT NULL REFERENCES tags (id),
    CONSTRAINT cores_tags_unique UNIQUE (core_id, tag_id)
);

CREATE TABLE cores_systems
(
    id        INTEGER PRIMARY KEY,
    core_id   INTEGER NOT NULL REFERENCES cores (id),
    system_id INTEGER NOT NULL REFERENCES systems (id),
    CONSTRAINT cores_systems_unique UNIQUE (core_id, system_id)
);
