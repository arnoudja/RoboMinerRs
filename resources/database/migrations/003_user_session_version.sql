-- Bump User.sessionVersion on password change so signed session cookies can be revoked.
ALTER TABLE User
    ADD COLUMN sessionVersion INT NOT NULL DEFAULT 0;
