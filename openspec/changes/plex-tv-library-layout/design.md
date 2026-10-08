## Revision: flat season folders

The first build put each episode in its own folder (`Season N/<episode>/<episode>.mp4`). A check on a real Plex server showed the Plex NFO Series agent only looks for `tvshow.nfo` one folder above a media file's folder, so shows stayed unmatched (`local://` guid, folder name as title, no poster). Copying `tvshow.nfo` into the season folder made the agent match. The `tv` layout is therefore flat, Plex's documented `<show>/Season N/<episode file>`:

```
<show>/Season 2026/S2026E01021530 - My Video.{mp4,nfo,jpg}
```

This section supersedes every per-episode-folder detail in Files, Types & Signatures, Call Stack and Test Plan below (episode folder creation via `prepare_folder`, `Season N/<folder>` entries, naming files after a nested folder's last component). The rest of the design stands.

**Video entry (the per-video unit).** `video_entry(relative_path)`:
- `"Season 2026/X.mp4"` → `"Season 2026/X"`: the season folder plus the media/thumbnail file's stem, i.e. the episode base name.
- `"Season 2026/X/X.mp4"` (a first-build nested episode) → `"Season 2026/X"`, so those keep working.
- `"F/F.mp4"` → `"F"`, `"F.mp4"` → `"F.mp4"`: unchanged.

`entry_owns(entry, file)`: `file == entry`, or for a season entry `file` starts with `entry + "."`. An entry's files are every listed season-folder file it owns (`X.mp4`, `X.nfo`, `X.jpg`, a `X.f137.mp4.part` is never listed).

**Deleting an entry.** `delete_video_entry(video_file_repository, output_dir, entry)` (domain, next to `VideoFileDeleter`): a season entry lists its season folder and deletes each owned file; any other entry is deleted as before. Used by `VideoFileDeleter`, `VideoDownloader`'s failed-download and deleted-mid-download cleanup, and orphan cleanup. A season folder itself is never deleted.

**Claiming an episode name.** New `VideoDownloaderRepository::prepare_episode(season_dir: &Path, desired_name, video_id) -> anyhow::Result<String>`: creates the season folder if missing and returns `desired_name`, or `"<desired_name> [<video_id>]"` when a file in the season folder already uses that base name. `VideoDownloader` and `ThumbnailFetcher` call it in `tv` layout instead of `prepare_folder`. Folder collision handling for `movie` layout is unchanged.

**yt-dlp output template.** When the folder `yt-dlp` runs in is a season folder (`is_season_dir` of its last component), the `-o` template is `<desired_filename>.%(ext)s`; otherwise it is named after the folder, as before. So `download(.., desired "S2026E01021530 - T", existing_folder Some("Season 2026"))` writes `Season 2026/S2026E01021530 - T.mp4` and reports folder `Season 2026`.

**Reuse.** In `tv` layout a recorded entry `"Season N/<base>"` (a thumbnail fetched ahead) is reused as folder `Season N` + base `<base>`; a recorded movie-layout entry `"F"` is reused as folder `F` + base `F`.

**Orphan cleanup.** Season-folder files are kept when owned by the `video_entry` of a downloaded file or recorded thumbnail, or when their title (after `strip_episode_prefix`) equals an unrecorded folder candidate or starts with one followed by `.` (a download or thumbnail fetch in flight).

**Plex scan.** `PlexFolderScanner::scan_downloaded_video(output_dir, folder)` scans the whole show folder when `folder`'s first component is a season folder (implemented), so a new show is matched from its `tvshow.nfo` and poster.

**Other deviations already implemented.**
- `ShowMetadataWriter::new(layout, show_metadata_repository, videos_path)`; `write_for_channel(&Channel)` / `write_for_playlist(&Playlist)` resolve the show folder themselves (the channel reconciler doesn't know the videos root).
- The Plex collections task handler is only registered in `movie` layout, so a task left scheduled by an earlier `movie` run never runs in `tv`.
- `NfoFile::for_media_file` yields an episode NFO only for a file inside a `Season N` folder whose base name has an episode prefix.
- `FilesystemShowMetadataRepository::write_tvshow_nfo` creates a missing show folder (a new channel's first pass runs before any download).
- `tvshow.nfo` and the poster are written on every reconcile pass of every tracked channel and playlist, so channels tracked before switching to `tv` get them too.

**Tests (TDD, section 5 of tasks.md).**
40. `it_should_resolve_the_video_entry_of_a_flat_episode_file` — `video_entry`, `entry_owns`.
41. `it_should_download_a_channel_video_into_its_season_folder_in_tv_layout` — `prepare_episode(<out>/Season 2026, "S2026E01021530 - My Video", "yt1")`, then `download(.., "S2026E01021530 - My Video", .., Some("Season 2026"))`; recorded `Season 2026/S2026E01021530 - My Video.mp4` (replaces 10, adapts 11).
42. `it_should_reuse_the_season_folder_and_name_of_a_prefetched_thumbnail_in_tv_layout` (replaces 12).
43. `it_should_remove_the_episode_files_if_a_tv_layout_download_fails` — the season folder and other episodes stay (replaces 15).
44. `it_should_remove_only_the_episode_files_if_the_video_is_deleted_during_a_tv_layout_download`.
45. `it_should_fetch_a_thumbnail_into_its_season_folder_in_tv_layout` (replaces 17).
46. `it_should_delete_orphaned_episode_files_inside_season_folders` (replaces 24).
47. `it_should_keep_the_episode_files_of_a_download_in_flight` (replaces 25).
48. `it_should_delete_only_the_episode_files_of_a_removed_tv_video` (replaces 28).
49. `it_should_claim_an_episode_name_in_a_missing_season_dir` and `it_should_suffix_an_episode_name_already_used_in_the_season_dir` (adapter; replace 32).
50. `it_should_name_the_file_after_the_desired_name_inside_a_season_folder` (adapter; replaces 33).
51. `it_should_scan_the_show_folder_of_a_tv_layout_episode` (implemented).

## Files

- `src/domain/shared/library_layout.rs` — new `LibraryLayout` value object (`movie` | `tv`), parsed from the env var.
- `src/domain/shared/mod.rs` — export `LibraryLayout`.
- `src/domain/video_metadata/episode_number.rs` — new `EpisodeNumber`: season/episode from the publish time or playlist position, its season dir and code, and parsing a name's `S…E… - ` prefix.
- `src/domain/video_metadata/show_metadata.rs` — new `ShowMetadata` (what `tvshow.nfo` holds), from a `Channel` or `Playlist`.
- `src/domain/video_metadata/nfo_file.rs` — new `NfoFile` (sidecar filename + rendered XML). Picks `movie.nfo` or an episode NFO.
- `src/domain/video_metadata/nfo.rs` — add `render_episode_nfo` and `render_tvshow_nfo` next to `render_movie_nfo`.
- `src/domain/video_metadata/mod.rs` — export the above.
- `src/domain/video/video_filename.rs` — add `episode_folder_name` (`"S2026E01021530 - Title"`, sanitized and truncated like a title).
- `src/domain/video/video_output_entry.rs` — add `video_entry` (the per-video unit of a recorded path: `Season N/<folder>` or the top-level entry), `is_season_dir`, `strip_episode_prefix`. `video_dir_for_filename` returns the file's parent dir.
- `src/domain/video/video.rs` — add `recorded_video_entry` (the folder a prefetched thumbnail or download already lives in).
- `src/domain/services/show_metadata_writer.rs` — new domain service: writes `tvshow.nfo` + poster in `tv` layout; a no-op in `movie`; best-effort.
- `src/domain/services/mod.rs` — export `ShowMetadataWriter`.
- `src/domain/services/video_downloader.rs` — takes `LibraryLayout`. In `tv`: requires metadata, reuses the recorded episode folder or prepares one in the season dir, writes/removes the episode `NfoFile`.
- `src/domain/services/thumbnail_fetcher.rs` — takes `LibraryLayout` and `MetadataGenerator`. In `tv`: prepares the episode folder before fetching, skips without metadata.
- `src/domain/services/internal_video_reconciler.rs` — metadata repair renders `NfoFile::for_media_file`. Orphan cleanup lists inside season dirs, protects by `video_entry`, never deletes `tvshow.nfo`, `poster.*` or season dirs.
- `src/domain/services/video_file_deleter.rs` — deletes `video_entry` instead of `top_level_entry`.
- `src/domain/services/channel_video_reconciler.rs` / `playlist_video_reconciler.rs` — call `ShowMetadataWriter` once per pass.
- `src/infrastructure/repositories/youtube_video_downloader_repository.rs` — fake records `prepare_folder` calls.
- `src/infrastructure/shared/ytdlp.rs` — creating a fresh folder also creates a missing parent (season) dir; output templates name the file after the folder's last path component, so a nested `Season N/<folder>` works.
- `src/infrastructure/repositories/sqlite_video_metadata_repository.rs` — `save` / `write_nfo` / `remove_nfo` take an `NfoFile` / sidecar filename instead of hardcoding `movie.nfo`.
- `src/infrastructure/repositories/filesystem_show_metadata_repository.rs` — new port `ShowMetadataRepository` + filesystem adapter (`avatars_dir`) + fake.
- `src/infrastructure/repositories/mod.rs` — export it.
- `src/serve.rs` — parse `YARRTUBE_LIBRARY_LAYOUT`, wire the layout and `ShowMetadataWriter`, skip scheduling Plex collections in `tv`.
- `src/application/tasks/{download_video,fetch_thumbnail,reconcile_channel,reconcile_playlist,delete_video_file}_task.rs` — behaviour tests only.
- `doc/INSTALLATION.md`, `doc/PLEX.md` — env var + TV library setup (Plex TV Series scanner, Plex NFO Series agent, hide seasons).

## Types & Signatures

```rust
// domain/shared/library_layout.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryLayout { #[default] Movie, Tv }
impl LibraryLayout {
    /// `None`/empty → Movie; case-insensitive "movie"/"tv"; else Err.
    pub fn parse(value: Option<&str>) -> Result<Self, ValidationError>;
}

// domain/video_metadata/episode_number.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpisodeNumber { pub season: u32, pub episode: u32, episode_width: usize }
impl EpisodeNumber {
    /// season = UTC year, episode = MMDDhhmm (width 8).
    pub fn for_channel_video(published_at: DateTime<Utc>) -> Self;
    /// season = 1, episode = position (width 2).
    pub fn for_playlist_video(position: i64) -> Self;
    /// Playlist position when `Some`, else publish time — same rule as `resolve_sorttitle`.
    pub fn resolve(published_at: DateTime<Utc>, sort_position: Option<i64>) -> Self;
    pub fn season_dir(&self) -> String; // "Season 2026" / "Season 01"
    pub fn code(&self) -> String;       // "S2026E01021530" / "S01E03"
    /// Reads a folder/file name's "S<n>E<n> - " prefix; None for a movie-layout name.
    pub fn parse_prefix(name: &str) -> Option<Self>;
}

// domain/video_metadata/show_metadata.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowMetadata { pub title: String, pub studio: String, pub uniqueid: String }
impl ShowMetadata {
    pub fn for_channel(channel: &Channel) -> Self;   // uniqueid = youtube_channel_id
    pub fn for_playlist(playlist: &Playlist) -> Self; // uniqueid = playlist id
}

// domain/video_metadata/nfo_file.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NfoFile { pub filename: String, pub content: String }
impl NfoFile {
    pub fn movie(metadata: &VideoMetadata) -> Self; // "movie.nfo"
    pub fn episode(metadata: &VideoMetadata, episode: EpisodeNumber, media_basename: &str) -> Self; // "<basename>.nfo"
    /// Episode NFO if the media file's basename has an episode prefix, else movie.nfo.
    pub fn for_media_file(metadata: &VideoMetadata, media_filename: &str) -> Self;
}

// domain/video_metadata/nfo.rs
pub fn render_episode_nfo(metadata: &VideoMetadata, episode: EpisodeNumber) -> String;
pub fn render_tvshow_nfo(show: &ShowMetadata) -> String;

// domain/video/video_filename.rs
pub fn episode_folder_name(episode: EpisodeNumber, title: &str) -> VideoFilename;

// domain/video/video_output_entry.rs
/// "Season 2026/F/F.mp4" → "Season 2026/F"; "F/F.mp4" → "F"; "F.mp4" → "F.mp4".
pub fn video_entry(relative_path: &str) -> &str;
pub fn is_season_dir(name: &str) -> bool;                 // "Season <digits>"
pub fn strip_episode_prefix(name: &str) -> Option<&str>;  // "S2026E01021530 - T" → "T"
/// The recorded file's parent dir (was: its first path component).
pub fn video_dir_for_filename(output_dir: &Path, filename: &str) -> PathBuf;

// domain/video/video.rs (changed)
impl Video {
    /// `video_entry` of the recorded filename, else of the recorded thumbnail.
    pub fn recorded_video_entry(&self) -> Option<&str>;
}

// domain/services/show_metadata_writer.rs
pub struct ShowMetadataWriter {
    layout: LibraryLayout,
    show_metadata_repository: Arc<dyn ShowMetadataRepository>,
}
pub trait ShowMetadataWriterApi: Send + Sync {
    /// tvshow.nfo + poster from avatar. No-op in Movie. Failures logged, never returned.
    fn write_for_channel(&self, channel: &Channel, output_dir: &Path);
    /// tvshow.nfo only. No-op in Movie.
    fn write_for_playlist(&self, playlist: &Playlist, output_dir: &Path);
}

// domain/services/video_downloader.rs, thumbnail_fetcher.rs (changed)
impl VideoDownloader { pub fn new(/* existing args */, layout: LibraryLayout) -> Self; }
impl ThumbnailFetcher {
    pub fn new(/* existing args */, metadata_generator: Arc<MetadataGenerator>,
               playlist_video_repository: Arc<dyn PlaylistVideoRepository>, layout: LibraryLayout) -> Self;
}

// domain/services/{channel,playlist}_video_reconciler.rs (changed)
pub fn new(/* existing args */, show_metadata_writer: Arc<ShowMetadataWriter>) -> Self;

// infrastructure/repositories/youtube_video_downloader_repository.rs — signatures unchanged.
// tv layout calls the existing `prepare_folder(&output_dir.join(season_dir), episode_name, youtube_id, None)`;
// its collision fallback (" [<youtube_id>]") applies inside the season dir.

// infrastructure/repositories/sqlite_video_metadata_repository.rs (changed)
pub trait VideoMetadataRepository: Send + Sync {
    fn save(&self, video_id: &VideoRecordId, metadata: &VideoMetadata, nfo: &NfoFile, video_dir: &Path) -> anyhow::Result<()>;
    fn find(&self, video_id: &VideoRecordId) -> anyhow::Result<Option<VideoMetadata>>;
    fn write_nfo(&self, nfo: &NfoFile, video_dir: &Path) -> anyhow::Result<()>;
    fn remove_nfo(&self, nfo_filename: &str, video_dir: &Path) -> anyhow::Result<()>;
}

// infrastructure/repositories/filesystem_show_metadata_repository.rs
pub trait ShowMetadataRepository: Send + Sync {
    fn write_tvshow_nfo(&self, output_dir: &Path, content: &str) -> anyhow::Result<()>;
    /// Copies avatars_dir/<avatar_filename> to output_dir/poster.<ext>; no-op if that poster exists.
    fn write_poster(&self, output_dir: &Path, avatar_filename: &str) -> anyhow::Result<()>;
}
pub struct FilesystemShowMetadataRepository { avatars_dir: PathBuf }
pub struct FakeShowMetadataRepository { /* recorded writes, failing() */ }

// serve.rs
fn library_layout() -> anyhow::Result<LibraryLayout>;
fn schedules_plex_collections(layout: LibraryLayout, plex: Option<&PlexConfig>) -> bool;
```

## Call Stack

**Thumbnail fetch, tv layout** (`FetchThumbnailTask::handle(payload)`)
1. `ThumbnailFetcher::fetch(&video, output_dir)`. It's unchanged when a thumbnail is already recorded or the video isn't fetchable.
2. → `video.recorded_video_entry()`. `Some(folder)` (a downloaded video missing its thumbnail) → reuse it.
3. → otherwise `metadata_generator.generate(&video, sort_position, None)`. `None` → log + return, with no folder.
4. → `episode = EpisodeNumber::resolve(metadata.published_at, sort_position)`
5. → `name = episode_folder_name(episode, &video.title)`; `folder = format!("{}/{}", episode.season_dir(), video_downloader_repository.prepare_folder(&output_dir.join(episode.season_dir()), name, youtube_id, None)?)`
6. → `video_downloader_repository.fetch_thumbnail(url, title, youtube_id, output_dir, Some(&folder))`. The file is named after the folder's last component.
7. → `record_thumbnail` → `thumbnail_filename = "Season 2026/S2026E01021530 - T/S2026E01021530 - T.jpg"`

**Download, tv layout** (`DownloadVideoTask::handle(payload, is_last_attempt)`)
1. `VideoDownloader::download(video_id, quality, output_dir, is_last_attempt)`
2. → `fetch_metadata(&video)` → `(Option<VideoMetadata>, sort_position)`
3. → `run_download(..)`:
   - **Missing metadata:** `metadata == None` → `fail_retryably(video, "YouTube metadata unavailable, needed to name the episode", ..)`. No folder is prepared and `yt-dlp` isn't run.
   - **Choose the folder:** `folder = video.recorded_video_entry()` (reused), else prepared in `EpisodeNumber::resolve(..).season_dir()` as in thumbnail step 5 (fresh).
   - **Episode number:** `episode = EpisodeNumber::parse_prefix(last component of folder)`.
   - **NFO ahead of the download:** `write_nfo(&NfoFile::episode(&metadata.with_thumb("<name>.jpg"), episode, name), video_dir)`.
   - **Download:** `video_downloader_repository.download(url, title, youtube_id, quality, output_dir, Some(&folder))`.
   - **On failure:** `remove_nfo(&nfo.filename, video_dir)`, plus `video_file_repository.delete(output_dir, &folder)` if the folder was fresh.
4. → `record_downloaded(..)` → `save(&id, &metadata, &NfoFile::episode(..), video_dir)`. The recorded filename is `"<folder>/<name>.mp4"`.

In `Movie`, every step is as today, with `NfoFile::movie`.

**Video removed** (`DeleteVideoFileTask`): `VideoFileDeleter::delete_video_file` → `video_file_repository.delete(output_dir, video_entry(filename))`. In tv layout that's `Season 2026/<episode folder>`; in movie layout it's the top-level folder, as before.

**Channel/playlist reconcile** (`Reconcile{Channel,Playlist}Task::handle`)
1. `reconcile_*` → `show_metadata_writer.write_for_channel(&channel, &output_dir)` / `write_for_playlist(..)`
   - `write_tvshow_nfo(output_dir, &render_tvshow_nfo(&ShowMetadata::for_channel(channel)))`
   - channel with `avatar_filename = Some(f)` → `write_poster(output_dir, f)`
2. → `internal_video_reconciler.reconcile(&desired, &delta)`
   - **Read actual state:** `read_actual_state` lists `output_dir`. For each `is_season_dir` entry it also lists that dir, adding `"Season N/<entry>"` to `files`. Season dirs, `tvshow.nfo` and `poster.*` themselves are never candidates.
   - **Protected entries:** `video_entry(..)` of every downloaded filename and recorded thumbnail. Inside a season dir, an entry whose `strip_episode_prefix` matches an unrecorded video's `unrecorded_folder_candidates` is also protected (a reservation in flight).
   - **Metadata repair:** `generate_metadata` → `save(&id, &m, &NfoFile::for_media_file(&m, filename), video_dir_for_filename(..))`.

**Startup** (`serve`)
1. `library_layout()?` (error → daemon exits)
2. Construct `VideoDownloader::new(.., layout)`, `ThumbnailFetcher::new(.., metadata_generator, playlist_video_repository, layout)`, and `ShowMetadataWriter::new(layout, Arc::new(FilesystemShowMetadataRepository::new(avatars_path)))`. Pass the writer to both reconcilers.
3. `if schedules_plex_collections(layout, plex.as_ref())` → schedule the collections task; `else if layout == Tv && plex.is_some()` → `warn!`. The Plex folder-scan subscribers stay registered whenever `plex` is `Some`.

## Test Plan

### 1. Behaviour

Domain (pure unit tests in their own modules):
1. `it_should_default_to_the_movie_layout_when_unset` — `parse(None)` and `parse(Some(""))` → `Movie`.
2. `it_should_parse_the_tv_layout_case_insensitively` — `"TV"` → `Tv`.
3. `it_should_reject_an_unknown_layout` — `"series"` → `Err` naming `YARRTUBE_LIBRARY_LAYOUT`.
4. `it_should_number_a_channel_video_by_publish_year_and_minute` — `2026-01-02T15:30:45Z`, no position → `Season 2026`, `S2026E01021530`.
5. `it_should_number_a_playlist_video_by_its_position` — position 3 → `Season 01`, `S01E03`.
6. `it_should_parse_the_episode_prefix_of_a_name` — `"S2026E01021530 - T"` → season 2026, episode 1021530. `"T"` → `None`.
7. `it_should_resolve_the_video_entry_of_a_recorded_path` — season, top-level and legacy flat paths.
8. `it_should_render_a_well_formed_episode_nfo` — root `episodedetails`, `season`, `episode`, `aired`, `youtube` `uniqueid`, `thumb`; escapes `&<>"'`.
9. `it_should_render_a_well_formed_tvshow_nfo` — root `tvshow`, `title`, `studio`, `youtube` `uniqueid`, escaped.

`DownloadVideoTask` (tv layout unless stated; real sqlite + temp dirs + fake downloader):

10. `it_should_download_a_channel_video_into_its_episode_folder_in_tv_layout` — the fake gets `prepare_folder(<out>/Season 2026, "S2026E01021530 - My Video", ..)`, then `download(.., Some("Season 2026/S2026E01021530 - My Video"))`. The recorded filename is under that folder.
11. `it_should_name_a_playlist_video_by_its_position_in_tv_layout` — `Season 01`, `S01E03 - My Video`.
12. `it_should_reuse_the_episode_folder_of_a_prefetched_thumbnail_in_tv_layout` — no `prepare_folder` call; downloads into the thumbnail's folder.
13. `it_should_write_the_episode_nfo_before_the_video_lands_in_tv_layout` — `<name>.nfo` exists when the download hook runs; no `movie.nfo`.
14. `it_should_record_the_episode_nfo_after_a_tv_layout_download` — the final NFO references `<name>.jpg`; metadata is recorded.
15. `it_should_remove_the_episode_nfo_and_fresh_folder_if_a_tv_layout_download_fails`.
16. `it_should_fail_retryably_without_metadata_in_tv_layout` — `Err`, errored-retrying, no `prepare_folder` or download call.

`FetchThumbnailTask`:

17. `it_should_fetch_a_thumbnail_into_its_episode_folder_in_tv_layout` — the recorded thumbnail path is `Season 2026/S2026E01021530 - My Video/S2026E01021530 - My Video.jpg`.
18. `it_should_skip_the_thumbnail_fetch_without_metadata_in_tv_layout` — no `prepare_folder` or fetch call; nothing recorded.

`ReconcileChannelTask`:

19. `it_should_write_tvshow_nfo_and_poster_in_tv_layout`.
20. `it_should_skip_the_poster_for_a_channel_without_an_avatar`.
21. `it_should_not_write_show_files_in_movie_layout`.
22. `it_should_keep_reconciling_when_show_files_cannot_be_written`.
23. `it_should_keep_show_files_and_season_folders_during_orphan_cleanup`.
24. `it_should_delete_orphaned_episode_folders_inside_season_folders`.
25. `it_should_keep_the_episode_folder_of_a_download_in_flight` — an unrecorded `Season 2026/S2026E01021530 - My Video` for a pending "My Video" survives.
26. `it_should_repair_missing_metadata_as_an_episode_nfo`.
27. `it_should_repair_missing_metadata_of_a_movie_layout_video_as_movie_nfo`.

`DeleteVideoFileTask`:

28. `it_should_delete_only_the_episode_folder_of_a_removed_tv_video` — `Season 2026` and its sibling episodes remain.

`ReconcilePlaylistTask`:

29. `it_should_write_tvshow_nfo_without_a_poster_for_a_playlist_in_tv_layout`.

`serve.rs`:

30. `it_should_not_schedule_plex_collections_in_tv_layout`.
31. `it_should_schedule_plex_collections_in_movie_layout_when_configured`.

### 2. Infrastructure

`YtDlpVideoDownloaderRepository` / `ytdlp` (real temp dirs; `yt-dlp` stubbed as in the existing tests):

32. `it_should_create_a_missing_season_dir_when_preparing_a_folder` — `prepare_folder(<out>/Season 2026, "S2026E01021530 - T", ..)` creates both dirs.
33. `it_should_name_the_downloaded_file_after_the_last_component_of_a_nested_folder` — `-o` template is `S2026E01021530 - T.%(ext)s`, run inside `Season 2026/S2026E01021530 - T`.

`FilesystemShowMetadataRepository` (real temp dirs):

34. `it_should_write_tvshow_nfo_at_the_output_dir_root`.
35. `it_should_copy_the_avatar_as_poster_keeping_its_extension`.
36. `it_should_leave_an_existing_poster_unchanged`.
37. `it_should_fail_when_the_avatar_file_is_missing`.

`SqliteVideoMetadataRepository`:

38. `it_should_write_the_given_nfo_file_on_save` (replaces `it_should_write_movie_nfo_and_record_a_row_on_save`).
39. `it_should_remove_only_the_named_nfo_file`.
