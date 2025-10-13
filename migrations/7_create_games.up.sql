  
CREATE TABLE IF NOT EXISTS events (
    id VARCHAR(36) PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),

    start_time TIMESTAMP NOT NULL,
    stop_time TIMESTAMP,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE location_kind AS ENUM ('home', 'away', 'other');

CREATE TABLE IF NOT EXISTS games (
    id VARCHAR(36) PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),

    opponent VARCHAR(255) NOT NULL,
    location VARCHAR(255) NOT NULL,
    location_kind location_kind NOT NULL,

    event_id VARCHAR(36) NOT NULL REFERENCES events(id) ON DELETE RESTRICT,
    invited_roles user_roles[] NOT NULL DEFAULT '{}'::user_roles[],

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE game_invite_status AS ENUM ('pending', 'accepted', 'declined', 'unsure', 'uninvited');

CREATE TABLE IF NOT EXISTS game_invites (
    id VARCHAR(36) PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),

    game_id  VARCHAR(36) NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    user_id  VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status game_invite_status NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(user_id, game_id)
);