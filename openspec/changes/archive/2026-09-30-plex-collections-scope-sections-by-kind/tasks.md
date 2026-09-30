## 1. Configuration

- [x] 1.1 In `src/serve.rs`, change `PlexIntegration` to carry
  `playlist_section_ids: Vec<String>` and `channel_section_ids: Vec<String>`,
  and rewrite `plex_integration()` to parse `YARRTUBE_PLEX_PLAYLIST_SECTION_ID`
  and `YARRTUBE_PLEX_CHANNEL_SECTION_ID` (each comma-separated), enabling the
  integration only when `YARRTUBE_PLEX_URL`, `YARRTUBE_PLEX_TOKEN`, and at
  least one non-empty section list are set. Verify with a unit test per case
  (disabled when both lists empty; enabled with only playlists; only channels;
  both).

## 2. Reconciler

- [x] 2.1 Change `PlexCollectionReconciler` to hold both section lists and
  update `PlexCollectionReconciler::new` accordingly; fix the `serve.rs`
  wiring at `plex_collection_reconciler(...)`.
- [x] 2.2 In `reconcile_all` / `reconcile_section`, run the tracked-playlist
  pass only over playlist sections and the tracked-channel pass only over
  channel sections (split the shared per-section setup from the per-kind
  loop). Verify with a test where a video shared between a playlist and a
  tracked channel does NOT produce a channel collection in the playlist
  section (the leak reproduction), plus a test that a channel section still
  creates the channel's collection.
- [x] 2.3 Verify existing reconciler behavior is preserved: playlist
  collection creation, no empty collections, membership convergence, and
  idempotency tests still pass (update their construction to the new
  signature).

## 3. Deletion

- [x] 3.1 Change `PlexCollectionDeleter` to hold both section lists and split
  its API so a playlist deletion scans only playlist sections and a channel
  deletion only channel sections; update the playlist-deleted and
  channel-deleted subscribers and the `serve.rs` wiring at
  `event_subscribers(...)`. Verify with tests that a deleted channel does not
  delete a same-named collection in a playlist section, and each kind deletes
  from its own sections.

## 4. Documentation

- [x] 4.1 Update `doc/PLEX.md` (step 4 "Find the library section ID(s)" and
  step 5 "Configure yarrtube") and any compose/README examples to use
  `YARRTUBE_PLEX_PLAYLIST_SECTION_ID` / `YARRTUBE_PLEX_CHANNEL_SECTION_ID`,
  explaining that each library must be one kind. Verify by rereading the doc
  end to end for stale `YARRTUBE_PLEX_SECTION_ID` references.

## 5. Validation

- [x] 5.1 Run `cargo test --locked`, `cargo fmt --all -- --check`, and
  `cargo clippy --all-targets --all-features --locked -- -D warnings`; all
  pass.
