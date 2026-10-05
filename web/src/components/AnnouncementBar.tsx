import { useState } from 'react'
import { X } from 'lucide-react'
import { useAnnouncements } from '@/api/queries'
import { Button } from '@/components/ui/button'
import {
  firstUnread,
  readDismissedAnnouncements,
  writeDismissedAnnouncements,
} from '@/lib/announcements'
import { parseAnnouncementText } from '@/lib/announcementText'

/**
 * Shows the newest announcement this browser hasn't dismissed, above the
 * header. Renders nothing while loading, on any fetch error, or when every
 * announcement has been read.
 */
export function AnnouncementBar() {
  const { data } = useAnnouncements()
  const [dismissed, setDismissed] = useState<ReadonlySet<string>>(
    () => new Set(readDismissedAnnouncements()),
  )
  const announcement = data && firstUnread(data, dismissed)
  if (!announcement) {
    return null
  }

  const dismiss = () => {
    const next = new Set(dismissed).add(announcement.id)
    setDismissed(next)
    writeDismissedAnnouncements([...next])
  }

  return (
    <div
      role="region"
      aria-label="Announcement"
      className="flex shrink-0 items-center gap-3 border-b border-border bg-muted px-4 py-2 text-sm sm:px-6"
    >
      <p className="min-w-0 flex-1">
        {parseAnnouncementText(announcement.text).map((segment, index) =>
          segment.kind === 'link' ? (
            <a
              key={index}
              href={segment.url}
              target="_blank"
              rel="noopener noreferrer"
              className="font-medium underline underline-offset-2"
            >
              {segment.label}
            </a>
          ) : (
            <span key={index}>{segment.text}</span>
          ),
        )}
      </p>
      <Button
        variant="ghost"
        size="icon"
        className="size-7 shrink-0"
        aria-label="Dismiss announcement"
        onClick={dismiss}
      >
        <X className="size-4" />
      </Button>
    </div>
  )
}
