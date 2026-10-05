## ADDED Requirements

### Requirement: Announcements Bar Coverage
The system SHALL verify, through the browser against the real image, that the announcements bar displays, links, dismisses, and persists correctly. The GitHub announcements URL SHALL be stubbed with a fixed fixture so the test does not depend on the live file.

#### Scenario: Show, follow link target, dismiss, persist
- **WHEN** the announcements URL is stubbed to return `[{"id":"smoke-b","text":"Second [docs](https://example.com/docs)"},{"id":"smoke-a","text":"First"}]` and the user opens `/` in a fresh browser context
- **THEN** a bar shows `Second docs`, where `docs` is a link to `https://example.com/docs` with `target="_blank"`
- **WHEN** the user clicks `Dismiss announcement`
- **THEN** the bar shows `First`
- **WHEN** the user dismisses it and reloads the page
- **THEN** no announcements bar is shown

#### Scenario: Announcements fetch fails
- **WHEN** the announcements URL is stubbed to abort and the user opens `/`
- **THEN** no announcements bar is shown and the header and sidebar render normally
