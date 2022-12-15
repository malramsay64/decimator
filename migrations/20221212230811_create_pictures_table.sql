-- Add migration script here
-- Create Pictures Table

CREATE TABLE IF NOT EXISTS picture (
    -- This is a UUID value, which is not natively supported so we use the blob
    -- type instead.
    id BLOB NOT NULL PRIMARY KEY,
    directory TEXT NOT NULL,
    filename TEXT NOT NULL,
    raw_filename TEXT,
    short_hash BLOB,
    full_hash BLOB,
    capture_time DATETIME,
    rating TEXT,
    flag TEXT
);

CREATE INDEX IF NOT EXISTS index_pictures_id ON picture(id);
CREATE INDEX IF NOT EXISTS index_pictures_directory ON picture(directory);
CREATE INDEX IF NOT EXISTS index_pictures_short_hash ON picture(short_hash);
CREATE INDEX IF NOT EXISTS index_pictures_full_has ON picture(full_hash);
