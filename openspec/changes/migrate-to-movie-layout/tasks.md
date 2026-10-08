## 1. Walking skeleton

- [x] 1.1 Add `Video::needs_movie_layout_migration` (→ `false`) and replace the last `reconcile` step with `migrate_layout`, calling `migrate_to_tv_layout` in `tv` layout and an empty `migrate_to_movie_layout` in `movie` layout. Done when `cargo test --locked` passes unchanged.

## 2. Migration back to the movie layout (TDD, acceptance through `ReconcileChannelTask`)

- [x] 2.1 `it_should_migrate_a_flat_episode_back_into_its_movie_folder_in_movie_layout` — drives `needs_movie_layout_migration`, the folder claim, the moves, `movie.nfo` and the leftover clean-up; shares the tv direction's move helpers.
- [x] 2.2 `it_should_migrate_a_first_build_episode_folder_back_in_movie_layout`.
- [x] 2.3 `it_should_suffix_a_movie_folder_name_already_taken_when_migrating_back`.
- [x] 2.4 `it_should_migrate_back_without_movie_nfo_when_metadata_is_unknown`.
- [x] 2.5 `it_should_remove_a_season_folder_left_empty_by_migrating_back`.
- [x] 2.6 `it_should_leave_a_movie_layout_video_alone`.
- [x] 2.7 `it_should_not_migrate_back_in_tv_layout`.
- [x] 2.8 `it_should_keep_the_record_if_moving_back_fails`.
- [x] 2.9 `it_should_not_migrate_back_a_video_that_is_not_downloaded`.

## 3. Docs and verification

- [x] 3.1 Update `doc/PLEX.md` (switching back to `movie` moves videos back into their own folders with `movie.nfo`; `tvshow.nfo` and posters stay); verify by reading the TV section.
- [x] 3.2 `cargo test --locked`, `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings` all pass.
- [x] 3.3 Manual check on a throwaway copy, not the live library: run a local build in `movie` layout over a videos folder holding tv-layout downloads; after one sync, confirm the files sit in `<Title>/<Title>.*` with `movie.nfo`, the episode files and empty season folders are gone, and the videos still play in the web UI.
