## Why

`migrate-to-tv-layout` moves existing downloads into the flat `tv` layout, but switching an install back to `YARRTUBE_LIBRARY_LAYOUT=movie` leaves them there: episodes flat in `Season N/` with episode NFOs, many to a folder. A Plex Movies library (Plex NFO Movie agent) expects one folder per video with a `movie.nfo`, so those videos import badly next to the new movie-layout downloads. Switching layouts should be safe in both directions.

## What Changes

- In `movie` layout, every reconcile pass of a tracked channel or playlist migrates its `Downloaded` videos stored inside a `Season <n>` folder (flat episode files, or the first tv build's per-episode folders) back into the movie layout, `<show>/<Title>/<Title>.{mp4,jpg}` + `movie.nfo`, without re-downloading:
  - the folder is named from the sanitized title with the existing folder collision fallback (`<Title> [<video id>]`), and the files are named after the folder, exactly like a new movie-layout download;
  - the media file and thumbnail are moved, `movie.nfo` is written and the metadata recorded when the video's metadata is known (stored, else fetched from YouTube), and the video's recorded filename and thumbnail are updated;
  - the episode's leftover files (its episode NFO and anything else sharing its base name, or its whole first-build episode folder) are then deleted, and a season folder left empty is removed.
- Best-effort per video, like `migrate-to-tv-layout`: a failed move is logged and retried next pass and never fails the pass. Unlike the tv direction, an unknown publish time doesn't block it: the movie layout names videos by title only, and a missing `movie.nfo` is repaired by the existing metadata repair.
- Only `Downloaded` videos; nothing is migrated back in `tv` layout.
- `tvshow.nfo` and `poster.*` at a show's root are left in place (harmless to a Movies library, and reused if the layout is switched to `tv` again).
- The `plex-tv-library-layout` change's "Layout Switch Does Not Migrate Existing Files" requirement no longer holds in either direction; it is narrowed to the metadata repair rule.
- Plex sees a migrated video as a new item, as in the tv direction.

## Capabilities

### New Capabilities
- `movie-layout-migration`: moving already-downloaded tv-layout videos of tracked channels and playlists back into the movie layout during reconciliation.

### Modified Capabilities
- None under `openspec/specs/`. The narrowed requirement lives in the unarchived `plex-tv-library-layout` change and is revised there.

## Impact

- Rust only, in `InternalVideoReconciler`: a `migrate_to_movie_layout` step next to `migrate_to_tv_layout`, sharing its move/record/clean-up helpers; `Video::needs_movie_layout_migration`. Reuses `VideoFileRepository::rename` and `delete_video_entry`.
- No DB schema, HTTP API, DTO or web UI changes. No new dependencies.
- Docs: `doc/PLEX.md` ("Switching back to `movie` doesn't move files" → videos are moved back into the movie layout).
