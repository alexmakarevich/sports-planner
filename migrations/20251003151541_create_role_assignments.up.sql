CREATE TYPE user_roles AS ENUM ('super_admin', 'org_admin', 'coach', 'player');

CREATE TABLE IF NOT EXISTS role_assignments (
    id CHAR(36) PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    user_id VARCHAR(36)  NOT NULL,
    role user_roles NOT NULL,
    UNIQUE (user_id, role),
    CONSTRAINT fk_user
        FOREIGN KEY(user_id)
	    REFERENCES users(id)
	    ON DELETE CASCADE
);