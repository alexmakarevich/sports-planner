CREATE TABLE IF NOT EXISTS config (
  is_initialized BOOLEAN NOT NULL DEFAULT FALSE
);

INSERT INTO config DEFAULT VALUES;

-- CREATE TYPE global_roles AS ENUM ('super_admin'); -- for now all global users are always global admins
-- TODO: add lower roles for other global users

CREATE TABLE IF NOT EXISTS global_users (
  id VARCHAR(36) PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
  username text     NOT NULL UNIQUE,
  password  text    NOT NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS clubs (
  id TEXT PRIMARY KEY NOT NULL,
  title TEXT     NOT NULL UNIQUE,
  
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS users (
  id VARCHAR(36) PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
  username text     NOT NULL UNIQUE,
  password  text    NOT NULL,
  club_id VARCHAR(36)  NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_club
    FOREIGN KEY(club_id)
	  REFERENCES clubs(id)
	  ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS sessions (
    id VARCHAR(36) PRIMARY KEY NOT NULL,
    user_id VARCHAR(36)  NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_user
        FOREIGN KEY(user_id)
	    REFERENCES users(id)
	    ON DELETE CASCADE
);

-- TODO: replace with global roles, club_roles, team_roles
CREATE TYPE user_roles AS ENUM ('super_admin', 'club_admin', 'coach', 'player');

CREATE TABLE IF NOT EXISTS role_assignments (
    id VARCHAR(36) PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    user_id VARCHAR(36)  NOT NULL,
    role user_roles NOT NULL,
  
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (user_id, role),
    CONSTRAINT fk_user
        FOREIGN KEY(user_id)
	    REFERENCES users(id)
	    ON DELETE CASCADE
);

-- roles on the level of the whole service
CREATE TYPE global_roles AS ENUM ('admin', 'user');

CREATE TABLE IF NOT EXISTS global_role_assignments (
    id VARCHAR(36) PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    user_id VARCHAR(36)  NOT NULL,
    role global_roles NOT NULL,
  
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (user_id, role),
    CONSTRAINT fk_user
        FOREIGN KEY(user_id)
	    REFERENCES users(id)
	    ON DELETE CASCADE
);

-- TODO: different types of invites: 
--   open for all
--   multi-user with confirmation
--   single-iser (one-time) /maybe with confirmation too

CREATE TABLE IF NOT EXISTS service_invites (
  id VARCHAR(16) PRIMARY KEY NOT NULL,
  club_id VARCHAR(36)  NOT NULL,
-- TODO: expiration
--   expires_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_club
    FOREIGN KEY(club_id)
	  REFERENCES clubs(id)
	  ON DELETE CASCADE
);

CREATE TABLE teams (
    id          TEXT PRIMARY KEY,          -- nanoid(6)
    club_id      TEXT NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE DEFAULT now(),
    updated_at  TIMESTAMP WITH TIME ZONE DEFAULT now(),

    UNIQUE (club_id, slug)           -- a slug is unique *within* an club
);

  
CREATE TABLE IF NOT EXISTS events (
    id VARCHAR(36) PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),

    start_time TIMESTAMPTZ NOT NULL,
    stop_time TIMESTAMPTZ,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE location_kind AS ENUM ('home', 'away', 'other');

CREATE TABLE IF NOT EXISTS games (
    id VARCHAR(36) PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),

    team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    opponent VARCHAR(255) NOT NULL,
    location VARCHAR(255) NOT NULL,
    location_kind location_kind NOT NULL,

    -- an event is a synthetic entity that carries the generic info.
    -- exactly one generic event is attarched to a specific event (like a game)
    event_id VARCHAR(36) UNIQUE NOT NULL REFERENCES events(id) ON DELETE RESTRICT,
    invited_roles user_roles[] NOT NULL DEFAULT '{}'::user_roles[],

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE game_invite_response AS ENUM ('pending', 'accepted', 'declined', 'unsure');

CREATE TABLE IF NOT EXISTS game_invites (
    id VARCHAR(36) PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),

    game_id  VARCHAR(36) NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    user_id  VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    response game_invite_response NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(user_id, game_id)
);