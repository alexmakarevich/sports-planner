CREATE TYPE USER_ROLES AS ENUM ('super_admin', 'org_admin', 'coach', 'player');

CREATE TABLE IF NOT EXISTS role_assignments (
    id VARCHAR(36) PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    user_id VARCHAR(36)  NOT NULL,
    role USER_ROLES NOT NULL,
  
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (user_id, role),
    CONSTRAINT fk_user
        FOREIGN KEY(user_id)
	    REFERENCES users(id)
	    ON DELETE CASCADE
);