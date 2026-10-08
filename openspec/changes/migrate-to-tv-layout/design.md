## Context

`InternalVideoReconciler::reconcile` runs once per reconcile pass of a channel or playlist. It reads the folder's actual state once (`read_actual_state`), then redownloads broken files, repairs missing metadata, recovers errored videos, reschedules stranded downloads, schedules missing thumbnails and deletes orphans, all from that one snapshot. The flat tv layout (change `plex-tv-library-layout`) already provides `EpisodeNumber::resolve`, `episode_folder_name`, `is_season_dir`, `is_named_after`, `video_entry`, `NfoFile::episode` and the collision rule `collision_suffixed_folder`. Files are moved within one output dir, so on one filesystem.

## Goals / Non-Goals

**Goals:** migrate `Downloaded` videos in `tv` layout in place, numbered and named exactly like a new download, without re-downloading, and never lose a file or leave a record naming a missing file.

**Non-Goals:** migrating back to `movie` layout; telling Plex to rescan migrated shows (the next download's show-folder scan, or Plex's own scheduled scan, picks them up); moving a legacy flat show-root `movie.nfo` (legacy flat videos share it, and it's harmless).

## Decisions

1. **Migrate inside `InternalVideoReconciler`, as the last step of a pass.** Orphan cleanup protects files by the *recorded* paths in the pass's snapshot. Migrating earlier would move files to paths cleanup doesn't know yet and delete them. Running last, cleanup still protects the old folder, then migration moves the files out and records the new paths for the next pass. *Alternative:* a separate one-shot migration task. Rejected: it would race with the reconcile pass over the same folders, and it adds scheduling for a job the pass already covers.
2. **A video needs migrating when its recorded filename isn't directly inside a season folder** (`Season N/<file>`, no deeper folder): movie layout `F/F.mp4`, legacy flat `F.mp4` and first-build `Season N/X/X.mp4` all qualify. Only `Downloaded` videos.
3. **Publish time from the stored metadata first**, else from `MetadataGenerator` (a YouTube call), else skip the video. The sort position comes from the pass's `DesiredState::sort_positions`, like metadata repair.
4. **Claim the episode name by listing the season folder** through `VideoFileRepository::list` and applying `is_named_after` + `collision_suffixed_folder`, the same rule as `prepare_episode`. *Alternative:* inject `VideoDownloaderRepository` for `prepare_episode`. Rejected: a second constructor change for a port the reconciler otherwise doesn't use; the rule is two pure functions.
5. **New port operation `VideoFileRepository::rename(output_dir, from, to)`**: creates `to`'s parent folders, refuses to overwrite an existing `to`, then `std::fs::rename`. Atomic on one filesystem, so a crash never leaves a half-copied file.
6. **Order per video:** move the media file (abort the video on failure); move the thumbnail (on failure, record no thumbnail and keep the old folder); record the new paths with a new `Video::relocate` transition; save the metadata as an episode NFO referencing the moved thumbnail; delete the old folder (`video_dir` of the old filename, never a season folder) only when every move succeeded. A kept old folder is unrecorded afterwards, so the next pass's orphan cleanup deletes it; a missing thumbnail is refetched by `schedule_missing`.
7. **`InternalVideoReconciler::new` takes the `LibraryLayout`.** Every construction site passes it (`LibraryLayout::Movie` in existing tests).

## Risks / Trade-offs

- [Plex treats moved files as new items, losing Plex-side watch state] → documented in `doc/PLEX.md`; yarrtube's own watch state is unchanged.
- [`video_repository.update` writes the whole row, so a watch-state change made during the few milliseconds of a migration could be overwritten] → accepted, the same race the reconciler already accepts for other updates.
- [A big library migrates in one pass, with many YouTube metadata calls for videos without stored metadata] → stored metadata covers every video downloaded with metadata; the rest cost one call each, once.
- [A crash between moving the file and recording it leaves the record naming a missing file] → the next pass's broken-file check resets it for redownload, as for any missing file.

## Migration Plan

Deploy normally. On the first `tv` layout reconcile pass of each channel and playlist, existing downloads are migrated. Rolling back to an older image leaves migrated videos in the flat tv layout, which that image (from `plex-tv-library-layout`) already reads.

## Files

- `src/domain/video/video.rs` — `Video::relocate(filename, thumbnail_filename, now)` and `Video::needs_tv_layout_migration()`.
- `src/domain/services/internal_video_reconciler.rs` — takes `LibraryLayout`; new last step `migrate_to_tv_layout`.
- `src/infrastructure/repositories/filesystem_video_file_repository.rs` — `VideoFileRepository::rename` + filesystem adapter + fake (`with_rename_error`).
- `src/serve.rs` and every `InternalVideoReconciler::new` call site — pass the layout.
- `src/application/tasks/reconcile_channel_task.rs`, `reconcile_playlist_task.rs` — acceptance tests.
- `doc/PLEX.md` — existing downloads are migrated.
- `openspec/changes/plex-tv-library-layout/specs/tv-library-layout/spec.md` — narrow "Layout Switch Does Not Migrate Existing Files" to the `movie` direction (done with this proposal).

## Types & Signatures

```rust
// domain/video/video.rs
impl Video {
    /// Downloaded and its recorded file isn't directly inside a season folder.
    pub fn needs_tv_layout_migration(&self) -> bool;
    /// The video's files moved: records their new paths, nothing else changes.
    pub fn relocate(self, filename: impl Into<String>, thumbnail_filename: Option<String>, now: DateTime<Utc>) -> Self;
}

// infrastructure/repositories/filesystem_video_file_repository.rs
pub trait VideoFileRepository {
    /// Moves `from` to `to` (both relative to `output_dir`), creating `to`'s
    /// parent folders; fails if `from` is missing or `to` already exists.
    fn rename(&self, output_dir: &Path, from: &str, to: &str) -> anyhow::Result<()>;
}

// domain/services/internal_video_reconciler.rs
impl InternalVideoReconciler {
    pub fn new(/* existing args */, layout: LibraryLayout) -> Self;
}
```

## Call Stack

`Reconcile{Channel,Playlist}Task::handle` → `reconcile_*` → `InternalVideoReconciler::reconcile(desired, delta)`:
1. existing steps, unchanged, ending with `delete_orphaned_files(&actual)`;
2. `migrate_to_tv_layout(desired, &actual)` when `layout == Tv`: for each `video` in `actual.videos` with `needs_tv_layout_migration()`, `migrate_video(desired, &actual, video)`, logging and swallowing its error:
   - `published_at`: `video_metadata_repository.find(&id)` else `metadata_generator.generate(..)`; `None` → skip;
   - `episode = EpisodeNumber::resolve(published_at, sort_position)`; `season = episode.season_dir()`;
   - `base`: `episode_folder_name(..)`, suffixed when `video_file_repository.list(output_dir/season)` has a name `is_named_after` it;
   - `rename(old filename → season/base.<ext>)`, then the thumbnail likewise;
   - `video_repository.update(&video.relocate(..))`;
   - `video_metadata_repository.save(&id, &metadata.with_thumb(..), &NfoFile::episode(..), output_dir/season)`;
   - `video_file_repository.delete(output_dir, old folder)` when every move succeeded.

## Test Plan

Acceptance (`ReconcileChannelTask` unless stated; real SQLite, real temp dirs with `FilesystemVideoFileRepository` unless stated):
1. `it_should_migrate_a_movie_layout_video_into_its_season_folder_in_tv_layout` — watched video with a duration and stored metadata; asserts the season folder's files, the old folder gone, the record (`relocate`d, still watched) and the recorded episode metadata.
2. `it_should_migrate_a_first_build_episode_folder_in_tv_layout`.
3. `it_should_migrate_a_legacy_flat_playlist_video_by_its_position_in_tv_layout` (`ReconcilePlaylistTask`).
4. `it_should_number_a_migrated_video_from_youtube_without_stored_metadata`.
5. `it_should_suffix_a_migrated_episode_name_already_used_in_the_season_folder`.
6. `it_should_leave_a_video_already_in_its_season_folder_alone`.
7. `it_should_not_migrate_in_movie_layout`.
8. `it_should_leave_a_video_without_a_publish_time_unmigrated`.
9. `it_should_keep_the_record_if_moving_the_media_file_fails` (`FakeVideoFileRepository::with_rename_error`).
10. `it_should_not_migrate_a_video_that_is_not_downloaded`.

Infrastructure (`FilesystemVideoFileRepository`, real temp dirs):
11. `it_should_rename_a_file_creating_its_parent_folder`.
12. `it_should_refuse_to_rename_over_an_existing_file`.
13. `it_should_fail_to_rename_a_missing_file`.
