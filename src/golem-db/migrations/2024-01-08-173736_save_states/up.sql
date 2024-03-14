CREATE TABLE savestates
(
    id              INTEGER PRIMARY KEY           NOT NULL,
    name            VARCHAR,
    core_id         INTEGER REFERENCES cores (id) NOT NULL,
    game_id         INTEGER REFERENCES games (id) NOT NULL,

    path            VARCHAR                       NOT NULL,
    screenshot_path VARCHAR,

    favorite        BOOLEAN                       NOT NULL DEFAULT FALSE,

    created_at      DATETIME                      NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_played     DATETIME                               DEFAULT CURRENT_TIMESTAMP
);

