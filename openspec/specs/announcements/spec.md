# announcements Specification

## Purpose

Lets the maintainer publish short announcements (releases, breaking changes, known issues) from the public repository. The web UI shows them as a dismissible bar, and each browser remembers what it has already dismissed.

## Requirements

### Requirement: Announcements Source
The web UI SHALL fetch announcements once per page load from `https://raw.githubusercontent.com/sergigp/yarrtube/main/announcements/announcements.json`. The file is a JSON array of objects with a string `id` and a string `text`, ordered newest first. Entries missing a string `id` or `text` SHALL be ignored.

#### Scenario: Valid announcements file
- **WHEN** the UI loads and the file returns `[{"id":"b","text":"Second"},{"id":"a","text":"First"}]`
- **THEN** both announcements are available for display, in that order

#### Scenario: Malformed entries are skipped
- **WHEN** the file returns `[{"id":"b"},{"id":"a","text":"First"}]`
- **THEN** only announcement `a` is available for display

### Requirement: Announcements Fail Silently
The web UI SHALL show no announcements bar and SHALL NOT show any error when the announcements fetch fails, returns a non-2xx status, or returns a body that is not a JSON array. The rest of the application SHALL keep working normally.

#### Scenario: GitHub unreachable
- **WHEN** the announcements request fails with a network error
- **THEN** no bar and no error message appear, and the rest of the UI renders normally

#### Scenario: Body is not an array
- **WHEN** the file returns `{"id":"a","text":"First"}`
- **THEN** no bar appears

### Requirement: Show Newest Unread Announcement
The web UI SHALL show a full-width bar above the application header containing the text of the first announcement, in file order, whose `id` has not been dismissed in this browser. When every announcement has been dismissed, or the list is empty, no bar SHALL be shown.

#### Scenario: Unread announcement shown
- **WHEN** the file returns `[{"id":"b","text":"Second"},{"id":"a","text":"First"}]` and nothing is dismissed
- **THEN** a bar above the header shows `Second`

#### Scenario: Previously dismissed announcement skipped
- **WHEN** the file returns `[{"id":"b","text":"Second"},{"id":"a","text":"First"}]` and `b` was dismissed earlier
- **THEN** the bar shows `First`

#### Scenario: Everything dismissed
- **WHEN** every announcement in the file has been dismissed
- **THEN** no bar is shown

### Requirement: Dismiss Announcement
The bar SHALL offer a button with the accessible name `Dismiss announcement`. Activating it SHALL mark the shown announcement's `id` as dismissed in this browser and immediately replace the bar with the next unread announcement, or hide it if none remain. Dismissals SHALL persist across reloads in the same browser and SHALL NOT be shared with other browsers or devices.

#### Scenario: Dismiss reveals the next one
- **WHEN** the bar shows `Second` and the user clicks `Dismiss announcement`
- **THEN** the bar shows `First`

#### Scenario: Dismissal survives reload
- **WHEN** the user dismisses every announcement and reloads the page
- **THEN** no bar is shown

#### Scenario: Storage unavailable
- **WHEN** browser storage cannot be read or written and the user dismisses an announcement
- **THEN** the bar still advances for the rest of the session, without an error

### Requirement: Inline Links In Announcements
Announcement text SHALL render each `[label](url)` occurrence whose `url` starts with `http://` or `https://` as a link showing `label`, pointing to `url`, and opening in a new tab with `rel="noopener noreferrer"`. Anything else, including links with other URL schemes and any HTML, SHALL render as literal text.

#### Scenario: Link rendered
- **WHEN** the shown announcement's text is `New release. [Notes](https://github.com/sergigp/yarrtube/releases)`
- **THEN** the bar shows `New release. Notes`, where `Notes` is a link to `https://github.com/sergigp/yarrtube/releases` that opens in a new tab

#### Scenario: Unsafe scheme not linked
- **WHEN** the text is `[click](javascript:alert(1))`
- **THEN** the bar shows the literal text `[click](javascript:alert(1))` with no link

#### Scenario: HTML not interpreted
- **WHEN** the text is `<b>bold</b>`
- **THEN** the bar shows the literal text `<b>bold</b>`
