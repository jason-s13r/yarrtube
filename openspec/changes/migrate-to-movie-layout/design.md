## Context

`migrate-to-tv-layout` added `migrate_to_tv_layout` as the last step of `InternalVideoReconciler::reconcile`, with helpers to move a media file and thumbnail (`VideoFileRepository::rename`, `moved_path`), record them (`Video::relocate`), save metadata and remove the old folder. `delete_video_entry` deletes an entry's files (`Season N/<base>` owns `<base>.*` and a first-build `<base>` folder). New movie-layout downloads are named by `VideoFilename::from_title`, with `collision_suffixed_folder` when an entry of that name already exists in the output dir, and the file named after the folder.

## Goals / Non-Goals

**Goals:** move tv-layout videos back into exactly the folder and file names a new movie-layout download would get, reusing the tv migration's machinery.

**Non-Goals:** removing `tvshow.nfo` or `poster.*` (harmless to a Movies library, reused if switched back to `tv`); telling Plex to rescan.

## Decisions

1. **One step per direction, picked by the layout.** `reconcile` ends with `migrate_layout(desired, &actual)`, which runs the tv or the movie migration. Both run last, after orphan cleanup, for the reason given in `migrate-to-tv-layout`'s design.
2. **A video needs moving back when it is `Downloaded` and its recorded filename's first component is a season folder** (`Video::needs_movie_layout_migration`), covering flat episodes and first-build episode folders.
3. **Folder claim by listing the output dir**: `VideoFilename::from_title(title)`, suffixed with `collision_suffixed_folder` when an entry of that name is listed. The files take the folder's name, as `yt-dlp` names a movie-layout download after its folder. *Alternative:* `VideoDownloaderRepository::prepare_folder`. Rejected for the same reason as the tv direction's name claim: an extra port for a two-function rule.
4. **Metadata is optional.** Stored metadata, else `MetadataGenerator`; when neither, move anyway and write no `movie.nfo` (metadata repair writes it on a later pass, as for any video missing metadata).
5. **Clean-up after the moves:** `delete_video_entry` on the old entry removes the episode NFO and leftovers (the moved files are no longer in the season folder), then each season folder touched in the pass is removed when `list` finds it empty. The old entry is only cleaned when every move succeeded, as in the tv direction.
6. **Shared helpers.** `moved_path`, `move_thumbnail`, the metadata lookup and the per-video error logging are shared between the two directions; `move_thumbnail` and `moved_path` take a target folder instead of a season.

## Risks / Trade-offs

- [Switching layouts repeatedly moves every video each time, and Plex treats each move as a new item] → documented; a layout switch is a deliberate, rare change.
- [Removing an empty season folder races with a tv-layout download into it] → can't happen: the movie direction only runs in `movie` layout, where nothing downloads into season folders.

## Files

- `src/domain/video/video.rs` — `Video::needs_movie_layout_migration()`.
- `src/domain/services/internal_video_reconciler.rs` — `migrate_layout` picking `migrate_to_tv_layout` / `migrate_to_movie_layout`; movie-direction helpers.
- `src/application/tasks/reconcile_channel_task.rs` — acceptance tests.
- `doc/PLEX.md` — switching back to `movie` moves videos back.
- `openspec/changes/plex-tv-library-layout/specs/tv-library-layout/spec.md` — narrow "Layout Switch Does Not Migrate Existing Files" to its metadata repair rule (done with this proposal).

## Types & Signatures

```rust
// domain/video/video.rs
impl Video {
    /// Downloaded and its recorded file is inside a season folder.
    pub fn needs_movie_layout_migration(&self) -> bool;
}
```

## Call Stack

`InternalVideoReconciler::reconcile` → … → `delete_orphaned_files(&actual)` → `migrate_layout(desired, &actual)`; in `movie` layout, for each `video` with `needs_movie_layout_migration()`, `migrate_video_to_movie(&actual, video)` (errors logged, never returned):
- `folder`: `VideoFilename::from_title(&video.title)`, suffixed when `video_file_repository.list(output_dir)` lists it;
- `rename(old → folder/folder.<ext>)`, then the thumbnail likewise;
- `video_repository.update(&video.relocate(..))`;
- metadata (stored, else generated) → `save(&id, &metadata.with_thumb(..), &NfoFile::movie(..), output_dir/folder)` when known;
- when every move succeeded, `delete_video_entry(old entry)`;
then, once per pass, each touched season folder that `list` finds empty is deleted.

## Test Plan

Acceptance (`ReconcileChannelTask`, real SQLite, real temp dirs with `FilesystemVideoFileRepository` unless stated):
1. `it_should_migrate_a_flat_episode_back_into_its_movie_folder_in_movie_layout` — watched, stored metadata; asserts files, record, metadata and `movie.nfo`.
2. `it_should_migrate_a_first_build_episode_folder_back_in_movie_layout`.
3. `it_should_suffix_a_movie_folder_name_already_taken_when_migrating_back`.
4. `it_should_migrate_back_without_movie_nfo_when_metadata_is_unknown`.
5. `it_should_remove_a_season_folder_left_empty_by_migrating_back` (and keep one still holding files).
6. `it_should_leave_a_movie_layout_video_alone`.
7. `it_should_not_migrate_back_in_tv_layout`.
8. `it_should_keep_the_record_if_moving_back_fails` (`FakeVideoFileRepository::with_rename_error`).
9. `it_should_not_migrate_back_a_video_that_is_not_downloaded`.
