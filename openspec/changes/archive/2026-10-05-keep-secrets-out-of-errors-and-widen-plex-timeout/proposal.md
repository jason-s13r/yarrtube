## Why

Last night's logs (2026-10-04 18:00 → 2026-10-05 07:28 UTC) showed two
problems. A DNS failure at 18:30:31 logged the full YouTube Data API key,
because every YouTube adapter sends it as a `key=` query parameter and the
HTTP client's connection errors include the request URL. The same message is
stored as the task's `last_error`, persisted in SQLite and shown by
`GET /api/tasks` and the web UI's task list. Separately, a Plex sync pass at
00:24 UTC, during Plex's nightly maintenance, took 29.3 s to list a section
against the 30 s request timeout set in #75 (normally about 1 s), so the next
slow night is likely to fail it.

## What Changes

- All four YouTube Data API adapters (videos, playlistItems, playlists,
  channels) send the API key in the `X-Goog-Api-Key` request header instead
  of the `key` query parameter, so no request URL, and therefore no error
  message built from one, carries it.
- New logging requirement: the YouTube API key never appears in log output or
  in a task's recorded error.
- Plex request timeout raised from 30 s to 60 s. This is an implementation
  setting with no spec change.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `logging`: adds the requirement that the YouTube API key is never written to
  log output or recorded task errors.

## Impact

- Code: `youtube_metadata_repository.rs`, `youtube_playlist_items_repository.rs`,
  `youtube_playlist_repository.rs`, `youtube_channel_repository.rs` (key moves
  from query to header; their mockito tests match the header);
  `plex_collection_repository.rs` (`REQUEST_TIMEOUT`).
- No config or API change. Google accepts `X-Goog-Api-Key` for the YouTube
  Data API v3, so the key's restrictions and quota are unchanged.
- Already-recorded `last_error` values and old container logs still contain
  the key; rotating it is an operator step, outside this change.
