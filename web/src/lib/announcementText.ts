export type AnnouncementSegment =
  | { kind: 'text'; text: string }
  | { kind: 'link'; label: string; url: string }

// Only http(s) targets become links, so a `javascript:` url stays inert text.
const LINK = /\[([^\]]+)\]\((https?:\/\/[^)\s]+)\)/g

/** Splits `text` into plain text and `[label](url)` link segments, in order. */
export function parseAnnouncementText(text: string): AnnouncementSegment[] {
  const segments: AnnouncementSegment[] = []
  let cursor = 0
  for (const match of text.matchAll(LINK)) {
    if (match.index > cursor) {
      segments.push({ kind: 'text', text: text.slice(cursor, match.index) })
    }
    segments.push({ kind: 'link', label: match[1], url: match[2] })
    cursor = match.index + match[0].length
  }
  if (cursor < text.length) {
    segments.push({ kind: 'text', text: text.slice(cursor) })
  }
  return segments
}
