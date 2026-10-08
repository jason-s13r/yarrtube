## Why

Switching an install to `YARRTUBE_LIBRARY_LAYOUT=tv` (change `plex-tv-library-layout`) only affects new downloads. Videos downloaded before the switch stay in the movie layout (`<show>/<Title>/<Title>.mp4` + `movie.nfo`, or a legacy flat `<show>/<Title>.mp4`), or in the first tv build's per-episode folders (`<show>/Season N/<episode>/<episode>.mp4`). Plex's TV scanner and NFO Series agent can't place those files, so long-tracked channels show mislabelled episodes (e.g. "Episode 11" with no title) next to the correct new ones. Re-adding a channel fixes it only by re-downloading every video.

## What Changes

- In `tv` layout, every reconcile pass of a tracked channel or playlist migrates its already-downloaded videos that aren't in the flat tv layout yet into `<show>/Season N/S…E… - Title.{mp4,nfo,jpg}`, without re-downloading:
  - the episode is numbered from the video's publish time (from its stored metadata, else fetched from YouTube) and its playlist position, exactly as for a new download;
  - the episode name is claimed in the season folder with the existing collision rule;
  - the media file and thumbnail are moved, the episode NFO is written and the metadata recorded, and the video's recorded filename and thumbnail are updated;
  - the old per-video folder (with its `movie.nfo` and any leftovers) is then deleted.
- Best-effort per video: a failure is logged and the video is retried on the next pass; it never fails the reconcile pass. A video whose publish time can't be determined is left as it is.
- Only `Downloaded` videos are migrated, never one with a download in flight, and nothing is migrated in `movie` layout (switching back to `movie` still doesn't move files).
- The `plex-tv-library-layout` change's "Layout Switch Does Not Migrate Existing Files" requirement is narrowed accordingly: existing files are migrated in the `tv` direction only.
- Plex sees a migrated video as a new item, so Plex-side watch state and play history for it start over; yarrtube's own watch state is unchanged.

Assumption: migration is automatic in `tv` layout (no separate opt-in switch), since a `tv` library can only show migrated videos correctly.

## Capabilities

### New Capabilities
- `tv-layout-migration`: moving already-downloaded videos of tracked channels and playlists into the flat `tv` layout during reconciliation.

### Modified Capabilities
- None under `openspec/specs/`. The requirement this change narrows (`tv-library-layout`'s "Layout Switch Does Not Migrate Existing Files") still lives in the unarchived `plex-tv-library-layout` change, so it is revised there.

## Impact

- Rust only:
  - `InternalVideoReconciler` gains a migration step in `tv` layout (it needs the layout and the downloader port's `prepare_episode`).
  - `Video` gains a transition that records a video's new filename and thumbnail.
  - `VideoFileRepository` gains a `rename` operation (move a file within an output dir); filesystem adapter + fake.
- No DB schema, HTTP API, DTO or web UI changes. No new dependencies.
- Docs: `doc/PLEX.md`'s "Existing files are not migrated" note becomes "existing downloads are migrated in place".
