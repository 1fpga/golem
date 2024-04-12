-- Your SQL goes here

CREATE TABLE core_files
(
    id          INTEGER PRIMARY KEY           NOT NULL,
    name        VARCHAR,
    core_id     INTEGER REFERENCES cores (id) NOT NULL,
    core_index  INTEGER                       NOT NULL,
    game_id     INTEGER REFERENCES games (id) NOT NULL,

    path        VARCHAR                       NOT NULL,

    created_at  DATETIME                      NOT NULL,
    last_loaded DATETIME DEFAULT CURRENT_TIMESTAMP
);
