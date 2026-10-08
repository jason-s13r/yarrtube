## Why

Plex shows movies with portrait 2:3 posters, so a YouTube video's 16:9 thumbnail gets cropped. Plex shows TV episodes with 16:9 thumbnails, which fit YouTube thumbnails. Users want to load yarrtube's videos into a Plex **TV Shows** library using the **Plex NFO Series** agent: one show per tracked channel, with the channel avatar as its poster and one season per publish year. In Plex the seasons can be hidden, so a channel reads as one stream of videos. (sergigp/yarrtube#89)

## What Changes

- New `YARRTUBE_LIBRARY_LAYOUT` environment variable, `movie` (default, current behaviour unchanged) or `tv`. Any other value fails startup.
- In `tv` layout, each tracked channel's and playlist's output folder is a Plex TV show:
  - `tvshow.nfo` at the folder root (show title, studio, `uniqueid type="youtube"`), rewritten on every reconcile pass.
  - `poster.<ext>` at the folder root, copied from the channel's stored avatar (channels only, when it has one).
- In `tv` layout, videos go directly into season folders, with no folder per video (Plex's documented `Show/Season XX/episode` structure):
  ```
  <show>/Season 2026/S2026E01021530 - My Video.{mp4,nfo,jpg}
  ```
  - The episode NFO (root `<episodedetails>`) and the 16:9 thumbnail share the media file's base name, so Plex pairs them. The NFO replaces `movie.nfo`.
  - A video's episode files (those sharing its base name) are the unit for deletion, failed-download cleanup and orphan cleanup, like the per-video folder in `movie` layout. Season folders are never swept.
  - Why flat: a check on a real Plex server showed the Plex NFO Series agent only looks for `tvshow.nfo` one folder above a media file's folder. With an extra folder per episode, new shows stayed unmatched (folder name as title, no poster).
- Season and episode numbering uses YouTube's publish timestamp (`snippet.publishedAt`, UTC), the same value already used for `aired`/`premiered`:
  - **Channel video:** season = publish year; episode = `MMDDhhmm`, e.g. `S2026E01021530`. It depends only on the video, so it never changes, sorts by actual upload time, and needs no counter or lock.
  - **Playlist video:** `Season 01`, episode = playlist position (`S01E03`).
  - Plex documents no limit on episode numbers, and 8-digit ones are unusual (ytdl-sub uses `MMDD` + a 2-digit same-day index). A check on a real Plex server confirmed they display and sort correctly.
- In `tv` layout, the thumbnail fetched ahead of a download fetches the video's metadata to name itself after the episode in its season folder; the download then reuses that season folder and name. Without metadata, that early fetch is skipped.
- In `tv` layout a video's YouTube metadata is required to name its file. If it can't be fetched before the download, the attempt fails retryably instead of downloading under a name Plex can't place.
- In `tv` layout the Plex collections reconcile task is neither scheduled nor handled (logged at startup). Its item matching assumes movie items. Scan-on-download (`YARRTUBE_PLEX_VIDEOS_PATH`) still works, and for a `tv` layout video it scans the whole show folder so a new show is matched from its `tvshow.nfo`.
- Out of scope: migrating files already downloaded in `movie` layout. Switching layouts affects new downloads only. Existing videos keep their `movie.nfo` and names until redownloaded.
- Out of scope: filing playlist videos under their channel's show with the playlist as a Plex collection. A playlist is its own show for now. That needs a separate change to the folder ownership model.

## Capabilities

### New Capabilities
- `tv-library-layout`: the layout switch, show-level files (written for every tracked channel and playlist, existing ones included), season folders and episode naming, numbering, the episode NFO format, and cleanup of episode files.

### Modified Capabilities
- `video-metadata`: the sidecar's name and root element depend on the layout. A metadata fetch failure fails the download attempt in `tv` layout.
- `plex-collections`: the integration is never scheduled in `tv` layout, and a sync task left scheduled by an earlier run is removed whenever the integration is off.
- `video-naming`: the "no video ID by default" and collision fallback rules also cover `tv` layout episode names inside a season folder.

## Impact

- Rust only:
  - New domain types: `LibraryLayout`, `EpisodeSlot`/`EpisodeNumber`, `ShowMetadata` and `NfoFile`.
  - `VideoDownloader` and `ThumbnailFetcher` take the layout.
  - Video deletion and orphan cleanup understand episode files grouped by base name inside `Season N/`.
  - The yt-dlp adapter creates a missing season dir, and names the file after the desired base name when it's given an existing folder (the season folder).
  - There's a new filesystem `ShowMetadataRepository` port; `ShowMetadataWriter` resolves each show folder from the videos root.
  - The Plex folder scan after a download scans the show folder for a `tv` layout video.
  - `serve.rs` parses the env var and gates Plex collections.
- No DB schema, HTTP API, DTO or web UI changes. No new dependencies.
- Docs: `doc/INSTALLATION.md` (env var) and `doc/PLEX.md` (TV library setup with the Plex NFO Series agent and the Plex TV Series scanner).
