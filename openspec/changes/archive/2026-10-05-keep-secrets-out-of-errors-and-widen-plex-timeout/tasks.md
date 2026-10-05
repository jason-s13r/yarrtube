## 1. YouTube API key in a header

- [x] 1.1 `youtube_metadata_repository.rs`: send the key as `X-Goog-Api-Key` instead of the `key` query param; existing mockito tests also match the header, and a new test against an unroutable base URL asserts the error cause chain doesn't contain the key (red before, green after)
- [x] 1.2 Same change and tests for `youtube_playlist_items_repository.rs`
- [x] 1.3 Same change and tests for `youtube_playlist_repository.rs`
- [x] 1.4 Same change and tests for `youtube_channel_repository.rs`; then `grep -rn '"key"' src/infrastructure/repositories/youtube_*` returns nothing

## 2. Plex timeout

- [x] 2.1 Raise `REQUEST_TIMEOUT` in `plex_collection_repository.rs` to 60 s and update its comment; verify `cargo test plex` passes

## 3. Checks

- [x] 3.1 Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings` and `cargo test --locked`; all must pass
- [ ] 3.2 After deploying: confirm in `yarrlogs` that playlist/channel reconciles succeed (the header is accepted), and that no logged URL contains `key=`
