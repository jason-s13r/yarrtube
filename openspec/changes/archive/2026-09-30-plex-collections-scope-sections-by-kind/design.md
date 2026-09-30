## Context

See proposal.md — Why. The current reconciler holds one flat
`section_ids: Vec<String>` and, in `reconcile_section`, runs the playlist
loop (`plex_collection_reconciler.rs:94`) and the channel loop (`:107`) over
the *same* section, gated only by "is this video's YouTube ID in the
section's scanned set" (`downloaded_scanned_rating_keys`, `:184`). The
membership rule is inherently ambiguous for a video that belongs to both a
playlist and a subscribed channel, because a single scanned copy carries one
YouTube ID that both loops match. `PlexCollectionDeleter` has the same flat
list and deletes a collection by name from every section
(`plex_collection_deleter.rs`).

The config is parsed in `serve.rs::plex_integration()` into
`PlexIntegration { repository, section_ids }`, threaded into both the
reconciler (`:399`) and the deleter (`:352`).

## Goals / Non-Goals

**Goals:**
- A configured section is reconciled against exactly one kind — playlists or
  channels — so the channel pass can never touch a playlist library.
- Preserve multi-library support within each kind (comma-separated lists).
- Keep YouTube-ID matching, membership convergence, alphabetical sorting, and
  task scheduling untouched.

**Non-Goals:**
- Auto-removing the stray collections already created in production (removed
  once by hand; the next pass won't recreate them).
- Supporting a single Plex library that mixes playlists and channels without
  overlap collisions — the design assumes (and `doc/PLEX.md` recommends)
  separate libraries per kind. Listing one section id under both vars is
  allowed but reintroduces the overlap for that library, and is the operator's
  choice.

## Decisions

### Two kind-scoped env vars instead of one flat list

Replace `YARRTUBE_PLEX_SECTION_ID` with `YARRTUBE_PLEX_PLAYLIST_SECTION_ID`
and `YARRTUBE_PLEX_CHANNEL_SECTION_ID`, each still comma-separated. The
section→kind association lives in configuration because only the operator
knows which Plex library points at `/videos/playlists` versus
`/videos/channels`; yarrtube cannot infer it (matching is by YouTube ID, not
path — see the spec).

*Alternative considered — keep one flat list, disambiguate by file path:*
inspect each scanned Plex item's `Part` path and only match it to an entity
whose storage root (`/videos/playlists` vs `/videos/channels`) is a prefix.
Rejected: heavier (needs the item file path plumbed through `PlexItem` and a
storage-root notion in the domain), and the config would stay ambiguous about
which library is which — the operator would still need to express intent.

*Alternative considered — a membership threshold* (only create a collection
when most of its videos are scanned in the section). Rejected as a fragile
heuristic that still mislabels edge cases and adds no clarity.

### Model sections as a kind-tagged pair on `PlexIntegration`

`PlexIntegration` carries `playlist_section_ids: Vec<String>` and
`channel_section_ids: Vec<String>`. The reconciler stores both and, in
`reconcile_all`, iterates playlist sections running only the playlist pass and
channel sections running only the channel pass. Concretely, split
`reconcile_section` into the shared setup (list scanned items + existing
collections) plus a per-kind loop, so a playlist section never enumerates
channels and vice versa.

*Alternative considered — a single `Vec<(String, SectionKind)>`.* Equivalent;
two named vectors read more clearly at the call sites in `serve.rs` and match
how the operator thinks about the two libraries. Either is acceptable at
implementation time.

### Scope deletion by kind

The playlist-deleted and channel-deleted subscribers already know which kind
they are handling. Give `PlexCollectionDeleter` both section lists and split
its API into `delete_playlist_collection(name)` and
`delete_channel_collection(name)` (or a `delete(name, kind)`), each scanning
only its kind's sections. This keeps a deleted channel from ever deleting a
same-named collection sitting in a playlist library.

## Risks / Trade-offs

- **Breaking config change** → deployments silently lose the integration if
  they keep only the old `YARRTUBE_PLEX_SECTION_ID`. Mitigation: `doc/PLEX.md`
  and compose/README examples updated in the same change; the startup log
  already shows whether the reconcile task was scheduled, making a
  misconfiguration visible on the next boot.
- **Operator lists a section under both vars** → the overlap leak returns for
  that one library. Mitigation: documented as unsupported/discouraged; the
  recommended setup is one library per kind.
- **Stray collections linger after deploy** → cosmetic only; documented as a
  one-time manual delete in Plex.
