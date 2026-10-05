## 1. Walking skeleton

- [x] 1.1 Create every file and signature from design.md and wire them end to end. Seed `announcements/announcements.json` with `[]`. Add the `Announcement` type, `ANNOUNCEMENTS_URL` + `fetchAnnouncements`, `useAnnouncements`, and the `lib/announcements.ts` + `lib/announcementText.ts` functions with trivial bodies (empty arrays, `undefined`, no-op write). Add `AnnouncementBar`, returning `null`, mounted in `App.tsx` above `<header>`, plus the `anAnnouncement` builder in `src/test/helpers.tsx`. Existing tests that render `App` and see an unrouted announcements request get the route added. Done when `npm run check` passes with no new tests.

## 2. Behaviour (TDD)

- [x] 2.1 `shows the newest unread announcement`: drives the fetch, parsing, `firstUnread` and bar rendering above the header.
- [x] 2.2 `renders inline links that open in a new tab`: drives `parseAnnouncementText` link segments and `<a target="_blank" rel="noopener noreferrer">`.
- [x] 2.3 `shows the next announcement after dismissing`: drives the `Dismiss announcement` button and the in-memory dismissed set.
- [x] 2.4 `skips announcements dismissed earlier`: drives `readDismissedAnnouncements`.
- [x] 2.5 `renders nothing once every announcement is dismissed`: drives the empty state.
- [x] 2.6 `remembers dismissals across remounts`: drives `writeDismissedAnnouncements`.
- [x] 2.7 `renders nothing when the fetch fails`: drives the silent error path (non-2xx).
- [x] 2.8 `renders nothing when the body is not an array`: drives `parseAnnouncements` rejecting non-arrays.
- [x] 2.9 `skips entries without a string id or text`: drives `parseAnnouncements` per-entry validation.

## 3. Infrastructure adapters (TDD)

- [x] 3.1 `announcementText.test.ts`: `returns plain text as a single text segment`.
- [x] 3.2 `announcementText.test.ts`: `splits multiple links from the surrounding text`.
- [x] 3.3 `announcementText.test.ts`: `leaves non-http links as literal text`.
- [x] 3.4 `announcementText.test.ts`: `leaves unclosed link syntax as literal text`.
- [x] 3.5 `announcements.test.ts` (localStorage adapter): `reads no dismissed ids and writes without throwing when storage is unavailable`.
- [x] 3.6 `announcements.test.ts` (localStorage adapter): `reads no dismissed ids when the stored value is corrupt`.

## 4. Verification

- [x] 4.1 Write `smoke-tests/tests/announcements.spec.js` covering both scenarios of the `Announcements Bar Coverage` requirement, with the GitHub URL stubbed through `page.route`. Done when it passes under `scripts/run-smoke-tests.sh`.
- [x] 4.2 Run `npm run check` (in `web/`), `cargo test --locked`, `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings`. Done when all pass.
- [x] 4.3 Run `scripts/run-smoke-tests.sh` (full suite) and confirm every existing smoke test still passes with the bar in place.
- [x] 4.4 Run `scripts/run-local.sh`, temporarily point the fetch at a local fixture or a branch file with one linked announcement, and check the bar's look on desktop and mobile widths, the link, dismissal and reload.
