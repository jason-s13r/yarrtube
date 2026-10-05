import { beforeEach, describe, expect, it } from 'vitest'
import { readDismissedAnnouncements, writeDismissedAnnouncements } from './announcements'

describe('dismissed announcements storage', () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  it('reads no dismissed ids and writes without throwing when storage is unavailable', () => {
    const original = Object.getOwnPropertyDescriptor(window, 'localStorage')
    const denied = () => {
      throw new Error('storage denied')
    }
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      get: denied,
    })

    try {
      expect(readDismissedAnnouncements()).toEqual([])
      expect(() => writeDismissedAnnouncements(['a'])).not.toThrow()
    } finally {
      Object.defineProperty(window, 'localStorage', original!)
    }
  })

  it('reads no dismissed ids when the stored value is corrupt', () => {
    window.localStorage.setItem('yarrtube.dismissedAnnouncements', '{"a":1}')
    expect(readDismissedAnnouncements()).toEqual([])

    window.localStorage.setItem('yarrtube.dismissedAnnouncements', '["a",2]')
    expect(readDismissedAnnouncements()).toEqual(['a'])

    window.localStorage.setItem('yarrtube.dismissedAnnouncements', 'not json')
    expect(readDismissedAnnouncements()).toEqual([])
  })
})
