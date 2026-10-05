## Files

- `announcements/announcements.json`: the published source. Seeded with `[]` and edited on `main` to announce.
- `web/src/api/types.ts`: `Announcement` type.
- `web/src/api/client.ts`: `ANNOUNCEMENTS_URL` + `fetchAnnouncements`. This is the only call outside `/api`, so it bypasses `send()`.
- `web/src/api/queries.ts`: `useAnnouncements`, fetched once per load (no polling, no refetch on focus).
- `web/src/lib/announcements.ts`: pure logic. Validates the body, picks the first unread, and reads/writes dismissed ids in localStorage.
- `web/src/lib/announcementText.ts`: pure `[label](url)` parser that splits text into text and link segments.
- `web/src/components/AnnouncementBar.tsx`: renders the bar and owns the dismissed-id state.
- `web/src/App.tsx`: mounts `<AnnouncementBar />` as the first child of the root flex column, above `<header>`.
- `web/src/test/helpers.tsx`: `anAnnouncement` fixture builder.
- `smoke-tests/tests/announcements.spec.js`: end-to-end coverage with the GitHub URL stubbed through `page.route`.
- Tests are colocated: `AnnouncementBar.test.tsx`, `announcements.test.ts`, `announcementText.test.ts`.

## Types & Signatures

```ts
// api/types.ts
export interface Announcement {
  id: string
  text: string
}

// api/client.ts
export const ANNOUNCEMENTS_URL =
  'https://raw.githubusercontent.com/sergigp/yarrtube/main/announcements/announcements.json'
export async function fetchAnnouncements(): Promise<Announcement[]> // non-2xx -> throws; body -> parseAnnouncements

// api/queries.ts
export function useAnnouncements(): UseQueryResult<Announcement[]>
// queryKey ['announcements'], staleTime: Infinity, refetchOnWindowFocus: false, retry: false

// lib/announcements.ts
export function parseAnnouncements(body: unknown): Announcement[] // non-array -> throws; drops entries without string id/text
export function firstUnread(
  announcements: readonly Announcement[],
  dismissed: ReadonlySet<string>,
): Announcement | undefined
export function readDismissedAnnouncements(): string[] // key 'yarrtube.dismissedAnnouncements'; unreadable/corrupt -> []
export function writeDismissedAnnouncements(ids: readonly string[]): void // swallows storage errors

// lib/announcementText.ts
export type AnnouncementSegment =
  | { kind: 'text'; text: string }
  | { kind: 'link'; label: string; url: string }
export function parseAnnouncementText(text: string): AnnouncementSegment[] // only http(s) urls become links

// components/AnnouncementBar.tsx
export function AnnouncementBar(): JSX.Element | null
```

## Call Stack

Load:
```
App
 -> AnnouncementBar
     -> useAnnouncements()
         -> fetchAnnouncements()
             -> fetch(ANNOUNCEMENTS_URL)
             -> parseAnnouncements(await response.json())
     -> useState(() => new Set(readDismissedAnnouncements()))
     -> firstUnread(data, dismissed)          // undefined / loading / error -> render null
     -> parseAnnouncementText(announcement.text)
         -> <span> per text segment, <a href target="_blank" rel="noopener noreferrer"> per link
```

Dismiss:
```
<button aria-label="Dismiss announcement"> onClick
 -> next = new Set(dismissed).add(announcement.id)
 -> setDismissed(next)                         // re-render -> firstUnread picks the next one
 -> writeDismissedAnnouncements([...next])
```

## Test Plan

Vitest names follow the repo's plain-sentence `it('...')` style, not `it_should_...`.

1. Behaviour: `AnnouncementBar.test.tsx` (`renderWithProviders` + `mockApi` routing `GET <ANNOUNCEMENTS_URL>`)
   1. `shows the newest unread announcement`: `[b, a]` -> text of `b` visible.
   2. `renders inline links that open in a new tab`: `docs` link has the right `href`, `target="_blank"` and `rel="noopener noreferrer"`.
   3. `shows the next announcement after dismissing`: click `Dismiss announcement` -> text of `a` visible, `b` gone.
   4. `skips announcements dismissed earlier`: storage pre-seeded with `["b"]` -> `a` shown.
   5. `renders nothing once every announcement is dismissed`: storage `["a","b"]` -> no dismiss button.
   6. `remembers dismissals across remounts`: dismiss, unmount, re-render -> `a` shown.
   7. `renders nothing when the fetch fails`: route `{ status: 500 }` -> no bar.
   8. `renders nothing when the body is not an array`: route `{ id, text }` -> no bar.
   9. `skips entries without a string id or text`: `[{id:'b'}, a]` -> `a` shown.
2. Pure lib and storage adapter
   1. `announcementText.test.ts`: `returns plain text as a single text segment`.
   2. `announcementText.test.ts`: `splits multiple links from the surrounding text`.
   3. `announcementText.test.ts`: `leaves non-http links as literal text`: `javascript:` and `mailto:` stay text.
   4. `announcementText.test.ts`: `leaves unclosed link syntax as literal text`.
   5. `announcements.test.ts`: `reads no dismissed ids and writes without throwing when storage is unavailable`.
   6. `announcements.test.ts`: `reads no dismissed ids when the stored value is corrupt`.
