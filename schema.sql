CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE guilds (
    guild_id TEXT PRIMARY KEY
);

CREATE TABLE guild_settings (
    guild_id TEXT PRIMARY KEY REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    logs_channel_id TEXT,
    logs_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    logs_color INTEGER NOT NULL DEFAULT 65280
);

CREATE TABLE guild_admins (
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    user_id TEXT NOT NULL
);

-- Stores the limits that are applied to a guild
CREATE TABLE limits (
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    limit_id TEXT PRIMARY KEY DEFAULT uuid_generate_v4(),
    limit_name TEXT NOT NULL default 'Untitled',
    limit_type TEXT NOT NULL,
    limit_action TEXT NOT NULL,
    limit_per INTEGER NOT NULL,
    limit_time INTERVAL NOT NULL,
    limit_timeout_duration INTERVAL
);

-- Stores users and roles that are exempt from limit tracking in a guild
CREATE TABLE guild_whitelist (
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    entity_id TEXT NOT NULL,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('user', 'role')),
    PRIMARY KEY (guild_id, entity_id)
);


-- Stores a list of user actions and which limits they have hit
-- A user action contributes to a limit
CREATE TABLE user_actions (
    action_id TEXT PRIMARY KEY,
    limit_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id TEXT NOT NULL,
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    action_target TEXT NOT NULL,
    limits_hit TEXT[] NOT NULL DEFAULT '{}'
);

-- Stores the past limits that have been applied in a guild
CREATE TABLE past_hit_limits (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    limit_id TEXT NOT NULL REFERENCES limits(limit_id) ON DELETE CASCADE ON UPDATE CASCADE,
    cause TEXT[] NOT NULL DEFAULT '{}',
    notes TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);