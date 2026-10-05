import type { Announcement } from '@/api/types'

export function parseAnnouncements(body: unknown): Announcement[] {
  if (!Array.isArray(body)) {
    throw new Error('announcements file is not a JSON array')
  }
  return body.filter(isAnnouncement)
}

export function firstUnread(
  announcements: readonly Announcement[],
  dismissed: ReadonlySet<string>,
): Announcement | undefined {
  return announcements.find((announcement) => !dismissed.has(announcement.id))
}

const DISMISSED_KEY = 'yarrtube.dismissedAnnouncements'

export function readDismissedAnnouncements(): string[] {
  try {
    const stored: unknown = JSON.parse(window.localStorage.getItem(DISMISSED_KEY) ?? '[]')
    return Array.isArray(stored) ? stored.filter((id) => typeof id === 'string') : []
  } catch {
    // Unreadable storage (private windows, blocked site data) means nothing
    // dismissed yet, never an error.
    return []
  }
}

export function writeDismissedAnnouncements(ids: readonly string[]): void {
  try {
    window.localStorage.setItem(DISMISSED_KEY, JSON.stringify(ids))
  } catch {
    // Not remembered; the bar still advances for this session.
  }
}

function isAnnouncement(entry: unknown): entry is Announcement {
  return (
    typeof entry === 'object' &&
    entry !== null &&
    typeof (entry as { id?: unknown }).id === 'string' &&
    typeof (entry as { text?: unknown }).text === 'string'
  )
}
