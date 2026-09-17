# Sentinel

Sentinel is a Discord moderation safety bot that helps server administrators detect destructive moderation activity and respond before a mistake or raid becomes a disaster.

It tracks moderation events, lets administrators define thresholds, and can automatically remove roles, kick users, or ban users when a configured limit is reached. Sentinel is designed to be focused and understandable rather than a replacement for a large, all-in-one moderation suite.

## Features

- Monitor role, channel, kick, ban, and unban events.
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

Other useful commands include `/actions view`, `/help`, `/simplehelp`, and `/stats`.

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
```

`TOKEN`, `CLIENT_ID`, and `CLIENT_SECRET` are also accepted as compatibility aliases. `PROXY_URL` can be omitted.

## Run locally

Install Rust and PostgreSQL, create the database schema from [schema.sql](schema.sql), then run:

```powershell
cargo run
```

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

## Project identity

The user-facing bot name is **Sentinel**. The Rust package and executable are still named `skynet` for deployment compatibility with existing services, Docker commands, and databases. Renaming those internal identifiers requires a coordinated deployment migration.