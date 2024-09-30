-- Shortcut tables. Related to a user and can contain additional
-- free-form information.
CREATE TABLE shortcuts
(
    id      INTEGER PRIMARY KEY,
    user_id INTEGER      NOT NULL REFERENCES users (id),
    key     VARCHAR(255) NOT NULL,
    value   TEXT         NOT NULL,
    meta    JSON,
    CONSTRAINT shortcuts_user_id_key UNIQUE (user_id, key)
);
