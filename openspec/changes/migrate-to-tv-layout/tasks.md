## 1. Walking skeleton

- [ ] 1.1 Create every type and signature from design.md and wire them in, with trivial bodies: `Video::needs_tv_layout_migration` → `false`, `Video::relocate` → `self`, `VideoFileRepository::rename` (filesystem adapter → `Ok(())`, fake recording calls + `with_rename_error`), `InternalVideoReconciler::new(.., layout)` storing the layout and an empty `migrate_to_tv_layout` step called last in `reconcile`; pass the layout from `serve.rs` and `LibraryLayout::Movie` at every test call site. Done when `cargo test --locked` passes unchanged.

## 2. Filesystem rename (TDD)

- [ ] 2.1 `it_should_rename_a_file_creating_its_parent_folder` — drives `FilesystemVideoFileRepository::rename`.
- [ ] 2.2 `it_should_refuse_to_rename_over_an_existing_file`.
- [ ] 2.3 `it_should_fail_to_rename_a_missing_file`.

## 3. Migration (TDD, acceptance through the reconcile tasks)

- [ ] 3.1 `it_should_migrate_a_movie_layout_video_into_its_season_folder_in_tv_layout` — drives `needs_tv_layout_migration`, `relocate`, the episode name claim, the moves, the episode NFO and deleting the old folder.
- [ ] 3.2 `it_should_migrate_a_first_build_episode_folder_in_tv_layout`.
- [ ] 3.3 `it_should_migrate_a_legacy_flat_playlist_video_by_its_position_in_tv_layout` (`ReconcilePlaylistTask`).
- [ ] 3.4 `it_should_number_a_migrated_video_from_youtube_without_stored_metadata`.
- [ ] 3.5 `it_should_suffix_a_migrated_episode_name_already_used_in_the_season_folder`.
- [ ] 3.6 `it_should_leave_a_video_already_in_its_season_folder_alone`.
- [ ] 3.7 `it_should_not_migrate_in_movie_layout`.
- [ ] 3.8 `it_should_leave_a_video_without_a_publish_time_unmigrated`.
- [ ] 3.9 `it_should_keep_the_record_if_moving_the_media_file_fails`.
- [ ] 3.10 `it_should_not_migrate_a_video_that_is_not_downloaded`.

## 4. Docs and verification

- [ ] 4.1 Update `doc/PLEX.md` ("Existing files are not migrated" → existing downloads are migrated in place on the next sync, and Plex sees them as new items); verify by reading the TV section.
- [ ] 4.2 `cargo test --locked`, `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings` all pass.
- [ ] 4.3 Manual check: run an image of this branch in `tv` layout against a videos folder holding movie-layout and first-build episode-folder downloads; after one sync, confirm on disk the files sit in `Season N/` with episode NFOs and the old folders are gone, and that the videos still play in the web UI.
