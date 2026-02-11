CREATE TABLE teams (
    id          TEXT PRIMARY KEY,          -- nanoid(6)
    club_id      TEXT NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE DEFAULT now(),
    updated_at  TIMESTAMP WITH TIME ZONE DEFAULT now(),

    UNIQUE (club_id, slug)           -- a slug is unique *within* an club
);