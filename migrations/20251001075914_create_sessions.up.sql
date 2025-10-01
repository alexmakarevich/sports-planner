-- Add up migration script here
CREATE TABLE IF NOT EXISTS sessions (
    id VARCHAR(36) PRIMARY KEY NOT NULL,
    user_id VARCHAR(36)  NOT NULL,

    CONSTRAINT fk_user
        FOREIGN KEY(user_id)
	    REFERENCES users(id)
	    ON DELETE CASCADE
)