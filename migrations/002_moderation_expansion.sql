ALTER TABLE limits ADD COLUMN IF NOT EXISTS limit_timeout_duration INTERVAL;

CREATE TABLE IF NOT EXISTS guild_whitelist (
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    entity_id TEXT NOT NULL,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('user', 'role')),
    PRIMARY KEY (guild_id, entity_id)
);
