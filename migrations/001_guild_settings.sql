CREATE TABLE IF NOT EXISTS guild_settings (
    guild_id TEXT PRIMARY KEY REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    logs_channel_id TEXT,
    logs_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    logs_color INTEGER NOT NULL DEFAULT 65280
);

INSERT INTO guild_settings (guild_id)
SELECT guild_id FROM guilds
ON CONFLICT (guild_id) DO NOTHING;