## MODIFIED Requirements

### Requirement: Plex integration is optional and off by default

The system SHALL enable the Plex collections integration only when
`YARRTUBE_PLEX_URL`, `YARRTUBE_PLEX_TOKEN`, and at least one of
`YARRTUBE_PLEX_PLAYLIST_SECTION_ID` or `YARRTUBE_PLEX_CHANNEL_SECTION_ID` are
set to a non-empty value. Each section variable accepts a single section id or
a comma-separated list. `YARRTUBE_PLEX_PLAYLIST_SECTION_ID` names the
library sections reconciled against tracked playlists, and
`YARRTUBE_PLEX_CHANNEL_SECTION_ID` those reconciled against tracked channels.
When the integration is disabled the system MUST NOT contact any Plex server
and MUST behave exactly as it does today.

#### Scenario: Integration disabled

- **WHEN** the daemon starts without `YARRTUBE_PLEX_URL`, without
  `YARRTUBE_PLEX_TOKEN`, or with neither section variable set to a non-empty
  value
- **THEN** no Plex sync task is scheduled and no requests are made to a Plex
  server

#### Scenario: Integration enabled

- **WHEN** the daemon starts with `YARRTUBE_PLEX_URL`, `YARRTUBE_PLEX_TOKEN`,
  and at least one of `YARRTUBE_PLEX_PLAYLIST_SECTION_ID` or
  `YARRTUBE_PLEX_CHANNEL_SECTION_ID` set
- **THEN** a recurring global Plex collections sync task is scheduled

### Requirement: One collection per tracked playlist and channel

Each configured library section SHALL be scoped to exactly one kind: a
playlist section (from `YARRTUBE_PLEX_PLAYLIST_SECTION_ID`) or a channel
section (from `YARRTUBE_PLEX_CHANNEL_SECTION_ID`). A sync pass SHALL reconcile
a playlist section against tracked **playlists only** and a channel section
against tracked **channels only**; it MUST NOT create or converge a channel's
collection in a playlist section, nor a playlist's collection in a channel
section, even when a video belonging to the other kind has been scanned into
that section.

For every tracked playlist, a sync pass SHALL ensure a Plex collection named
after the playlist exists in each configured **playlist** section holding
videos of that playlist that Plex has scanned; likewise for every tracked
channel in each configured **channel** section. The system SHALL support
multiple configured sections of each kind, reconciling each independently, so
yarrtube-fed content split across several Plex libraries is covered.
Collections created by the system SHALL be configured to sort alphabetically,
so that yarrtube's position-prefixed `sorttitle` values order members by
playlist position / publish date.

#### Scenario: Collection created for a playlist with scanned videos

- **WHEN** a sync pass runs and a tracked playlist has downloaded videos that
  Plex has scanned in a configured playlist section, and no collection with
  the playlist's name exists in that section
- **THEN** a collection with the playlist's name is created in that section
  containing those videos, with alphabetical sorting configured

#### Scenario: Channel not leaked into a playlist section

- **WHEN** a sync pass runs over a configured playlist section and a video is
  both in a tracked playlist stored there and in a separately tracked channel,
  so the video is scanned into that section
- **THEN** the video is placed only in its playlist's collection, and no
  collection named after the channel is created in that playlist section

#### Scenario: Content split across libraries

- **WHEN** two playlist sections are configured and a playlist's scanned
  videos live in the second one
- **THEN** the playlist's collection is created in the second section, and no
  collection for it is created in the first

#### Scenario: No empty collection for content Plex has not scanned

- **WHEN** a sync pass runs and none of a playlist's downloaded videos are
  scanned by Plex yet in a configured playlist section
- **THEN** no collection is created for that playlist in this pass

### Requirement: Deleting a playlist or channel deletes its collection

When a playlist is deleted, the system SHALL delete its Plex collection from
every configured **playlist** section where one exists; when a channel is
deleted, from every configured **channel** section. A missing collection MUST
NOT be treated as an error.

#### Scenario: Playlist deleted

- **WHEN** a tracked playlist with an existing Plex collection in a configured
  playlist section is deleted
- **THEN** its Plex collection is deleted from that section

#### Scenario: Channel deleted

- **WHEN** a tracked channel with an existing Plex collection in a configured
  channel section is deleted
- **THEN** its Plex collection is deleted from that section

#### Scenario: Deletion with integration disabled

- **WHEN** a playlist or channel is deleted while the Plex integration is
  disabled
- **THEN** deletion completes exactly as today, with no Plex requests
