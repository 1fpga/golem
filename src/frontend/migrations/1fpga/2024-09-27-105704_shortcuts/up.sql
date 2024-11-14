-- Shortcut tables. Related to a user and can contain additional
-- free-form information.
CREATE TABLE shortcuts
(
    id       INTEGER PRIMARY KEY,
    user_id  INTEGER      NOT NULL REFERENCES users (id),

    -- The shortcut unique key, like "resetCore".
    key      VARCHAR(255) NOT NULL,

    -- The shortcut chosen by the user, like "Ctrl+Shift+R".
    shortcut TEXT         NOT NULL,

    -- Any additional metadata.
    meta     JSON,

    -- It is illegal to have two identical shortcuts for the same user.
    -- The system would not know which one to trigger.
    CONSTRAINT shortcuts_user_id_key UNIQUE (user_id, shortcut)
);
