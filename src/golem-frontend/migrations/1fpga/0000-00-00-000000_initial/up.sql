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
    value      TEXT         NOT NULL,
    updated_at TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT user_storage_user_id_key UNIQUE (user_id, key)
);

CREATE TABLE global_storage
(
    id         INTEGER PRIMARY KEY,
    key        VARCHAR(255) NOT NULL UNIQUE,
    value      TEXT         NOT NULL,
    updated_at TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_cores
(
    id              INTEGER PRIMARY KEY,
    user_id         INTEGER   NOT NULL REFERENCES users (id),
    catalog_core_id INTEGER   NOT NULL REFERENCES catalog_cores (id),
    favorite        BOOLEAN   NOT NULL DEFAULT FALSE,
    last_played_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_games
(
    id              INTEGER PRIMARY KEY,
    user_id         INTEGER   NOT NULL REFERENCES users (id),
    catalog_game_id INTEGER   NOT NULL,
    favorite        BOOLEAN   NOT NULL DEFAULT FALSE,
    last_played_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE savestates
(
    id              INTEGER PRIMARY KEY,
    core_id         INTEGER   NOT NULL REFERENCES catalog_cores (id),
    game_id         INTEGER   NOT NULL REFERENCES user_games (id),
    user_id         INTEGER   NOT NULL,
    state_path      TEXT      NOT NULL,
    screenshot_path TEXT      NOT NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE catalogs
(
    id               INTEGER PRIMARY KEY,
    name             VARCHAR(255) NOT NULL UNIQUE,
    url              TEXT         NOT NULL UNIQUE,
    -- The last time this was checked for updates.
    latest_check_at  TIMESTAMP             DEFAULT CURRENT_TIMESTAMP,
    -- The last time this was updated.
    latest_update_at TIMESTAMP             DEFAULT CURRENT_TIMESTAMP,
    -- The `lastUpdated` field from the JSON.
    last_updated     VARCHAR(255),
    -- The `version` field.
    version          VARCHAR(255),
    priority         INTEGER      NOT NULL DEFAULT 0
);

CREATE TABLE catalog_systems
(
    id          INTEGER PRIMARY KEY,
    catalog_id  INTEGER      NOT NULL REFERENCES catalogs (id),
    name        VARCHAR(255) NOT NULL,
    unique_name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    icon_path   TEXT,
    image_path  TEXT
);

CREATE TABLE catalog_games
(
    id          INTEGER PRIMARY KEY,
    system_id   INTEGER      NOT NULL REFERENCES catalog_systems (id),
    catalog_id  INTEGER      NOT NULL REFERENCES catalogs (id),
    name        VARCHAR(255) NOT NULL,
    unique_id   VARCHAR(255) NOT NULL,
    description TEXT         NOT NULL,
    executable  TEXT         NOT NULL,
    icon_path   TEXT,
    image_path  TEXT,
    CONSTRAINT catalog_games_system_id_unique_id UNIQUE (system_id, catalog_id, unique_id)
);

-- Installed cores from a catalog.
CREATE TABLE cores
(
    id          INTEGER PRIMARY KEY,
    system_id   INTEGER      NOT NULL REFERENCES catalog_systems (id),
    catalog_id  INTEGER      NOT NULL REFERENCES catalogs (id),
    name        VARCHAR(255) NOT NULL,
    unique_name VARCHAR(255) NOT NULL UNIQUE,
    rbf_path    VARCHAR(255),
    description TEXT,
    version     VARCHAR(255),
    icon_path   TEXT,
    image_path  TEXT
);
