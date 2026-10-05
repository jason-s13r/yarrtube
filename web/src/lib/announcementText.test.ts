import { describe, expect, it } from 'vitest'
import { parseAnnouncementText } from './announcementText'

describe('parseAnnouncementText', () => {
  it('returns plain text as a single text segment', () => {
    expect(parseAnnouncementText('Plex integration landed.')).toEqual([
      { kind: 'text', text: 'Plex integration landed.' },
    ])
  })

  it('splits multiple links from the surrounding text', () => {
    expect(
      parseAnnouncementText('See [notes](https://a.example/n) and [docs](http://b.example/d).'),
    ).toEqual([
      { kind: 'text', text: 'See ' },
      { kind: 'link', label: 'notes', url: 'https://a.example/n' },
      { kind: 'text', text: ' and ' },
      { kind: 'link', label: 'docs', url: 'http://b.example/d' },
      { kind: 'text', text: '.' },
    ])
  })

  it('leaves non-http links as literal text', () => {
    expect(parseAnnouncementText('[click](javascript:alert(1)) [mail](mailto:a@b.c)')).toEqual([
      { kind: 'text', text: '[click](javascript:alert(1)) [mail](mailto:a@b.c)' },
    ])
  })

  it('leaves unclosed link syntax as literal text', () => {
    expect(parseAnnouncementText('See [docs](https://example.com/docs')).toEqual([
      { kind: 'text', text: 'See [docs](https://example.com/docs' },
    ])
  })
})
