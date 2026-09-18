# Sentinel

Sentinel is a Discord moderation safety bot that helps server administrators detect destructive moderation activity and respond before a mistake or raid becomes a disaster.

It tracks moderation events, lets administrators define thresholds, and can automatically remove roles, kick users, or ban users when a configured limit is reached. Sentinel is designed to be focused and understandable rather than a replacement for a large, all-in-one moderation suite.

## Features

- Monitor role, channel, kick, ban, and unban audit events.
- Send audit notifications to a server-selected logs channel.
- Customize audit notification color or disable notifications without disabling limit tracking.
- Configure per-server thresholds and time windows.
- Automatically remove all roles, kick, or ban when a threshold is reached.
- Review configured limits, triggered limits, and moderation actions.
- Discord OAuth data export for authorized administrators.
- Slash-command choices and autocomplete for supported command inputs.

## Commands

The main workflow is:

1. Run `/setup` once in the server.
2. Run `/limits guide` for an explanation of the limit system.
3. Run `/limits add` and choose the event, threshold, time window, and response.
4. Use `/limits view` to review limits.
5. Use `/limits hit` to review triggered limits.
6. Use `/limits remove` to remove a limit from the autocomplete list.
7. Use `/settings` to configure the audit logs channel, notification color, or notification switch.

Other useful commands include `/actions view`, `/help`, `/simplehelp`, and `/stats`.

`/settings` accepts optional values so you can change one setting at a time:

- `logs_channel`: the channel where Sentinel posts audit event embeds.
- `logs_enabled`: whether Sentinel posts notifications. Limits and action history continue to work when this is disabled.
- `logs_color`: an embed color such as `0x3498DB` or its decimal value.

The settings command does not clear an existing channel when `logs_channel` is omitted. Run it with the channel you want to replace. Only guild owners and Sentinel guild admins can use it.

## Required Discord permissions

Invite Sentinel with the permissions required by the responses you configure:

- View Audit Log
- View Channels
- Send Messages
- Embed Links
- Read Message History
- Manage Roles, if using Remove All Roles
- Kick Members, if using Kick User
- Ban Members, if using Ban User

The configured logs channel must also allow Sentinel to view the channel, send messages, and embed links. Sentinel records role and channel create/update/delete events, plus member kick, ban, and unban events. Other Discord audit event types are currently ignored.

Sentinel cannot manage roles above its highest role. Move the bot role above the roles it needs to remove, and enable the required privileged intents if the Discord application configuration asks for them.

## Configuration

Sentinel supports either `config.yaml` or environment variables. Environment variables are convenient for Docker and Dokploy deployments. A `.env` file is also accepted for local development and should not be committed.

Required environment variables:

```text
DATABASE_URL=postgres://user:password@host:5432/sentinel
DISCORD_TOKEN=your-bot-token
DISCORD_CLIENT_ID=your-application-client-id
DISCORD_CLIENT_SECRET=your-oauth-client-secret
```

Optional variables:

```text
FRONTEND_URL=https://your-frontend.example
PROXY_URL=https://your-proxy.example
API_BIND_ADDR=127.0.0.1:4950
```

`TOKEN`, `CLIENT_ID`, and `CLIENT_SECRET` are also accepted as compatibility aliases. `PROXY_URL` can be omitted.

## Run locally

Install Rust and PostgreSQL, create the database schema from [schema.sql](schema.sql), then run:

```powershell
cargo run
```

### Existing installations and migrations

If Sentinel is already installed, apply [migrations/001_guild_settings.sql](migrations/001_guild_settings.sql) once against the same PostgreSQL database before deploying the updated bot:

```powershell
psql "$env:DATABASE_URL" -f migrations/001_guild_settings.sql
```

The migration is idempotent and creates default settings for existing guilds. New installations should use `schema.sql`, which already includes the `guild_settings` table. Sentinel does not run migrations automatically.

For a release build:

```powershell
cargo build --release
```

## Docker

The included [Dockerfile](Dockerfile) installs the native build dependencies required by the Rust dependencies and includes Git metadata during the build so the bot can display its commit information.

```powershell
docker build -t sentinel-bot .
docker run --rm --env-file .env sentinel-bot
```

Do not copy `.env` into the image or commit it to the repository.

## Dokploy and Railpack

Create a service from this repository and provide the required environment variables in the service settings. Dokploy can build the included Dockerfile. Railpack can use [railpack.json](railpack.json), but the Dockerfile is the preferred path when you need the repository metadata and native build packages to be explicit.

The application listens for the OAuth web server when the bot becomes ready. Make sure the configured `FRONTEND_URL` and any reverse-proxy or public routing settings match the deployment.

The same server exposes public read-only endpoints for the website:

- `GET /api/stats` returns availability, release, and capability metadata.
- `GET /api/commands` returns the public command catalog used by the website.

Set `API_BIND_ADDR` to the address required by your deployment. The web frontend can proxy these endpoints by setting `BOT_API_URL` to the bot's base URL.

## Project identity

The user-facing bot name is **Sentinel**. The Rust package and executable are still named `skynet` for deployment compatibility with existing services, Docker commands, and databases. Renaming those internal identifiers requires a coordinated deployment migration.