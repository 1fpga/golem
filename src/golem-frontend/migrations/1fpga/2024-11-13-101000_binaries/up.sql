CREATE TABLE catalog_binaries
(
    id             INTEGER PRIMARY KEY,
    catalog_id     INTEGER      NOT NULL REFERENCES catalogs (id),
    name           VARCHAR(255) NOT NULL,
    version        VARCHAR(255),
    update_pending BOOLEAN DEFAULT FALSE,
    CONSTRAINT catalog_binaries_unique UNIQUE (catalog_id, name)
)
