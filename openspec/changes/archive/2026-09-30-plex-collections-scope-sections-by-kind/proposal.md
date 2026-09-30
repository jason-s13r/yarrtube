## Why

The Plex collections reconciler runs **both** the tracked-playlist pass and
the tracked-channel pass against **every** configured library section. A
section that is a playlists library therefore also gets the channel pass, and
whenever a video is both in a playlist stored there **and** in a channel the
user separately subscribes to, that shared video's YouTube ID is scanned into
the playlists library — so a spurious channel-named collection leaks into it.

Observed in production: a playlists library (section 19) correctly produced
one collection per playlist, but also grew stray `David Ondrej` (2),
`Kun Chen` (1) and `Víctor Falcón` (1) collections — each a tracked channel
whose video also sat in the `watch-later` playlist.

## What Changes

- **BREAKING**: Replace the single `YARRTUBE_PLEX_SECTION_ID` env var with two
  kind-scoped vars, each still accepting a comma-separated list of section
  ids:
  - `YARRTUBE_PLEX_PLAYLIST_SECTION_ID` — libraries reconciled against tracked
    **playlists** only.
  - `YARRTUBE_PLEX_CHANNEL_SECTION_ID` — libraries reconciled against tracked
    **channels** only.
- The reconciler tags each configured section with its kind and, per section,
  runs **only** the matching pass: a playlist section never gets the channel
  pass, and vice versa. This removes the cross-contamination at its source.
- Collection deletion on playlist/channel removal follows the same scoping: a
  deleted playlist's collection is removed only from playlist sections, a
  deleted channel's only from channel sections.
- The integration stays off unless `YARRTUBE_PLEX_URL`, `YARRTUBE_PLEX_TOKEN`
  and at least one of the two section lists are set.

Out of scope: this change does not retroactively delete the stray collections
already created in production — those are removed once by hand in Plex and the
next pass will not recreate them.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `plex-collections`: the "One collection per tracked playlist and channel"
  requirement and the "Deleting a playlist or channel deletes its collection"
  requirement change so a section is scoped to playlists **or** channels; the
  "Integration is optional and off by default" requirement changes to reflect
  the two new env vars.

## Impact

- Configuration (**BREAKING**): deployments must rename
  `YARRTUBE_PLEX_SECTION_ID` to `YARRTUBE_PLEX_PLAYLIST_SECTION_ID` (and add
  `YARRTUBE_PLEX_CHANNEL_SECTION_ID` for a channels library). `doc/PLEX.md`
  and any compose/README examples need updating.
- Code: `src/serve.rs` (`plex_integration` parsing, `PlexIntegration`, the
  reconciler/deleter wiring), `src/domain/services/plex_collection_reconciler.rs`
  (section→kind tagging, per-kind pass selection),
  `src/domain/services/plex_collection_deleter.rs` (scope deletion by kind),
  and their tests.
- No change to the YouTube-ID matching, membership convergence, alphabetical
  sorting, or the recurring-task scheduling.
