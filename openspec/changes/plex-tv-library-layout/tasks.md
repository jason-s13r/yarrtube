## 1. Walking skeleton

- [x] 1.1 Create every file, type and signature from design.md and wire them in:
  - **New types and functions:** `LibraryLayout`, `EpisodeNumber`, `ShowMetadata`, `NfoFile`, `render_episode_nfo`, `render_tvshow_nfo`, `episode_folder_name`, `video_entry`, `is_season_dir`, `strip_episode_prefix`, `Video::recorded_video_entry`, `ShowMetadataWriter`, `ShowMetadataRepository` (filesystem adapter + fake), `prepare_folder` call recording in `FakeVideoDownloaderRepository`, `library_layout`, `schedules_plex_collections`.
  - **Metadata repository:** `save`, `write_nfo` and `remove_nfo` take an `NfoFile` or sidecar filename. Update every caller and test to pass `NfoFile::movie(..)` / `"movie.nfo"`.
  - **Constructors:** `VideoDownloader::new` gets `layout`; `ThumbnailFetcher::new` gets `metadata_generator`, `playlist_video_repository` and `layout`; both reconcilers' `new` get `show_metadata_writer`.
  - **Wiring:** in `serve.rs` (layout parsed from `YARRTUBE_LIBRARY_LAYOUT`) and in every test constructor (`LibraryLayout::Movie`, a fake show repo).
  - **Trivial bodies:**
    - `parse` → `Ok(Movie)`
    - `EpisodeNumber` constructors → season/episode 0, `season_dir`/`code` → empty; `parse_prefix` and `strip_episode_prefix` → `None`
    - `video_entry` → `top_level_entry`; `is_season_dir` → `false`; `recorded_video_entry` → `None`
    - `NfoFile::episode` / `for_media_file` → `NfoFile::movie`
    - writer methods do nothing
    - `schedules_plex_collections` → `plex.is_some()`

  Done when `cargo build` succeeds and all existing tests pass unchanged.

## 2. Behaviour (TDD)

- [x] 2.1 `it_should_default_to_the_movie_layout_when_unset` — `LibraryLayout::parse` of `None`/empty.
- [x] 2.2 `it_should_parse_the_tv_layout_case_insensitively`.
- [x] 2.3 `it_should_reject_an_unknown_layout` — error names `YARRTUBE_LIBRARY_LAYOUT`.
- [x] 2.4 `it_should_number_a_channel_video_by_publish_year_and_minute` — drives `EpisodeNumber::resolve`/`season_dir`/`code`.
- [x] 2.5 `it_should_number_a_playlist_video_by_its_position`.
- [x] 2.6 `it_should_parse_the_episode_prefix_of_a_name` — drives `EpisodeNumber::parse_prefix` and `strip_episode_prefix`.
- [x] 2.7 `it_should_resolve_the_video_entry_of_a_recorded_path` — drives `video_entry`, `is_season_dir`, `video_dir_for_filename`.
- [x] 2.8 `it_should_render_a_well_formed_episode_nfo` — drives `render_episode_nfo` + `NfoFile::episode`.
- [x] 2.9 `it_should_render_a_well_formed_tvshow_nfo` — drives `render_tvshow_nfo` + `ShowMetadata`.
- [x] 2.10 `it_should_download_a_channel_video_into_its_episode_folder_in_tv_layout` — drives `VideoDownloader`'s tv path + `episode_folder_name`.
- [x] 2.11 `it_should_name_a_playlist_video_by_its_position_in_tv_layout`.
- [x] 2.12 `it_should_reuse_the_episode_folder_of_a_prefetched_thumbnail_in_tv_layout` — drives `Video::recorded_video_entry`.
- [x] 2.13 `it_should_write_the_episode_nfo_before_the_video_lands_in_tv_layout`.
- [x] 2.14 `it_should_record_the_episode_nfo_after_a_tv_layout_download`.
- [x] 2.15 `it_should_remove_the_episode_nfo_and_fresh_folder_if_a_tv_layout_download_fails`.
- [x] 2.16 `it_should_fail_retryably_without_metadata_in_tv_layout`.
- [x] 2.17 `it_should_fetch_a_thumbnail_into_its_episode_folder_in_tv_layout` — drives `ThumbnailFetcher`'s tv path.
- [x] 2.18 `it_should_skip_the_thumbnail_fetch_without_metadata_in_tv_layout`.
- [x] 2.19 `it_should_write_tvshow_nfo_and_poster_in_tv_layout` — drives `ShowMetadataWriter::write_for_channel` + its call from `ChannelVideoReconciler`.
- [x] 2.20 `it_should_skip_the_poster_for_a_channel_without_an_avatar`.
- [x] 2.21 `it_should_not_write_show_files_in_movie_layout`.
- [x] 2.22 `it_should_keep_reconciling_when_show_files_cannot_be_written`.
- [x] 2.23 `it_should_keep_show_files_and_season_folders_during_orphan_cleanup`.
- [x] 2.24 `it_should_delete_orphaned_episode_folders_inside_season_folders` — drives listing inside season dirs + `video_entry` protection.
- [x] 2.25 `it_should_keep_the_episode_folder_of_a_download_in_flight` — drives `strip_episode_prefix` protection.
- [x] 2.26 `it_should_repair_missing_metadata_as_an_episode_nfo` — drives `NfoFile::for_media_file` in `generate_metadata`.
- [x] 2.27 `it_should_repair_missing_metadata_of_a_movie_layout_video_as_movie_nfo`.
- [x] 2.28 `it_should_delete_only_the_episode_folder_of_a_removed_tv_video` — drives `VideoFileDeleter` using `video_entry`.
- [x] 2.29 `it_should_write_tvshow_nfo_without_a_poster_for_a_playlist_in_tv_layout`.
- [x] 2.30 `it_should_not_schedule_plex_collections_in_tv_layout` — drives `schedules_plex_collections` + the startup warning.
- [x] 2.31 `it_should_schedule_plex_collections_in_movie_layout_when_configured`.

## 3. Infrastructure adapters (TDD)

`YtDlpVideoDownloaderRepository` / `ytdlp`

- [x] 3.1 `it_should_create_a_missing_season_dir_when_preparing_a_folder`.
- [x] 3.2 `it_should_name_the_downloaded_file_after_the_last_component_of_a_nested_folder` (also covers `fetch_thumbnail`).

`FilesystemShowMetadataRepository`

- [x] 3.3 `it_should_write_tvshow_nfo_at_the_output_dir_root`.
- [x] 3.4 `it_should_copy_the_avatar_as_poster_keeping_its_extension`.
- [x] 3.5 `it_should_leave_an_existing_poster_unchanged`.
- [x] 3.6 `it_should_fail_when_the_avatar_file_is_missing`.

`SqliteVideoMetadataRepository`

- [x] 3.7 `it_should_write_the_given_nfo_file_on_save` (replaces `it_should_write_movie_nfo_and_record_a_row_on_save`).
- [x] 3.8 `it_should_remove_only_the_named_nfo_file`.

## 4. Verification

- [x] 4.1 Update `doc/INSTALLATION.md` (`YARRTUBE_LIBRARY_LAYOUT`) and `doc/PLEX.md`:
  - TV Shows library with the Plex TV Series scanner, the Plex NFO Series agent and "Seasons: Hide";
  - Plex collections are unsupported in tv layout;
  - existing files aren't migrated.
- [x] 4.2 `cargo test --locked`, `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings`.
- [x] 4.3 Final manual check via `scripts/run-local.sh` (or a local image) with `YARRTUBE_LIBRARY_LAYOUT=tv`, after section 5. Track a new channel and a playlist, and confirm on disk:
  - `tvshow.nfo` and `poster.*` at each show's root;
  - `Season YYYY/S…E… - Title.{mp4,nfo,jpg}` directly in the season folder, with no folder per episode.

  Then in a Plex TV Shows library (NFO Series agent) whose folders are the `channels` and `playlists` folders, confirm a brand-new channel becomes its own show with its real name and avatar poster, without a manual refresh or fix match. Already confirmed on a real server with the first build: episodes are found, episode NFOs are read, 16:9 thumbnails and 8-digit episode numbers display and sort correctly.

## 5. Flat season folders (TDD)

Plex's NFO Series agent only reads `tvshow.nfo` one folder above a media file's folder, so episodes go directly into `Season N/` (see design.md's "Revision: flat season folders").

- [x] 5.1 `it_should_scan_the_show_folder_of_a_tv_layout_episode` — drives `PlexFolderScanner::scan_downloaded_video`.
- [x] 5.2 `it_should_resolve_the_video_entry_of_a_flat_episode_file` — drives `video_entry` and `entry_owns`.
- [x] 5.3 `it_should_claim_an_episode_name_in_a_missing_season_dir` — drives `VideoDownloaderRepository::prepare_episode` (yt-dlp adapter) + fake.
- [x] 5.4 `it_should_suffix_an_episode_name_already_used_in_the_season_dir`.
- [x] 5.5 `it_should_name_the_file_after_the_desired_name_inside_a_season_folder` — replaces 3.2's nested-folder test (also covers `fetch_thumbnail`).
- [x] 5.6 `it_should_download_a_channel_video_into_its_season_folder_in_tv_layout` — replaces 2.10; adapts 2.11, 2.13, 2.14 to the flat paths.
- [x] 5.7 `it_should_reuse_the_season_folder_and_name_of_a_prefetched_thumbnail_in_tv_layout` — replaces 2.12.
- [x] 5.8 `it_should_remove_the_episode_files_if_a_tv_layout_download_fails` — replaces 2.15; drives `delete_video_entry`.
- [x] 5.9 `it_should_remove_only_the_episode_files_if_the_video_is_deleted_during_a_tv_layout_download`.
- [x] 5.10 `it_should_fetch_a_thumbnail_into_its_season_folder_in_tv_layout` — replaces 2.17.
- [x] 5.11 `it_should_delete_orphaned_episode_files_inside_season_folders` — replaces 2.24; adapts 2.26.
- [x] 5.12 `it_should_keep_the_episode_files_of_a_download_in_flight` — replaces 2.25.
- [x] 5.13 `it_should_delete_only_the_episode_files_of_a_removed_tv_video` — replaces 2.28.
- [x] 5.14 Update `doc/PLEX.md` (flat tree; library folders are `channels` and `playlists`, not the videos root), then `cargo test --locked`, `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings`.
