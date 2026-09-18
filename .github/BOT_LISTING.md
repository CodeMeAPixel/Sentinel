# Sentinel Bot Listing

Reusable copy for Discord bot directories, invite pages, release posts, and server descriptions.

## Short description

Sentinel is a focused Discord safety bot that watches moderation activity, detects destructive patterns, and responds automatically when configured thresholds are reached.

## Medium description

Sentinel helps Discord server administrators protect their communities from moderation mistakes and raid activity. Configure limits for role changes, channel changes, kicks, bans, and unbans, then choose a time window and response. Sentinel can post customizable audit notifications to a selected logs channel, remove roles, kick users, or ban users when a moderator reaches a configured threshold, while keeping a history of actions and triggered limits for review.

## Long description

Sentinel is a moderation safety layer for Discord communities that want clear, configurable protection without an oversized moderation suite. Administrators define named limits for the events they care about, such as repeated bans, kicks, role changes, or channel changes. Each limit has a threshold, a time window, and a response.

When a moderator reaches a threshold, Sentinel can remove all of that user's roles, kick the user, or ban the user. Administrators can inspect configured limits with `/limits view`, see triggered limits with `/limits hit`, and review recorded moderation actions with `/actions view`. The `/settings` command selects the audit logs channel, enables or disables notifications, and changes the embed color. The `/limits guide` command explains the system inside Discord, while slash-command choices and autocomplete make setup easier.

Sentinel is intended to reduce the impact of destructive moderation activity. It should be configured with care and should complement, rather than replace, normal moderation practices and a larger anti-raid strategy.

## Feature bullets

- Configurable moderation-event limits.
- Role create, update, and remove monitoring.
- Channel create, update, and remove monitoring.
- Kick, ban, and unban monitoring.
- Configurable audit logs channel and notification appearance.
- Optional audit notifications without disabling limit tracking.
- Per-limit thresholds and time windows.
- Remove all roles, kick, or ban responses.
- Triggered-limit history.
- Moderation-action history.
- Slash commands with choices and autocomplete.
- OAuth-backed administrator data access.
- PostgreSQL persistence.

## Suggested tags

`moderation`, `security`, `antinuke`, `anti-raid`, `logging`, `administration`, `automod`, `discord bot`

## Suggested category

Moderation or Security

## Suggested invite description

Sentinel needs access to your server's audit log, the configured logs channel, and the moderation permissions required by the responses you choose. Grant Manage Roles only when using the Remove All Roles response, Kick Members when using Kick User, and Ban Members when using Ban User.

## Permissions

Recommended baseline permissions:

- View Audit Log
- View Channels
- Send Messages
- Embed Links
- Read Message History
- View and send messages in the configured logs channel

Response-specific permissions:

- Manage Roles for Remove All Roles
- Kick Members for Kick User
- Ban Members for Ban User

The Sentinel bot role must be above any roles it may remove. Avoid granting Administrator unless it is required by a specific deployment policy.

## One-line variants

- Configurable Discord anti-nuke and anti-raid protection for moderation events.
- Watch moderation activity, configure audit notifications, and respond automatically with Sentinel.
- A focused Discord safety bot for detecting destructive moderation patterns.

## Disclaimer

Sentinel is a safety layer, not a guarantee against raids or compromised accounts. Review thresholds before enabling automatic responses and keep a separate recovery and moderation plan.
