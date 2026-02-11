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