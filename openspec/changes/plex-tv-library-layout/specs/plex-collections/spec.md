## MODIFIED Requirements

### Requirement: Plex integration is optional and off by default

The system SHALL enable the Plex collections integration only when
`YARRTUBE_PLEX_URL`, `YARRTUBE_PLEX_TOKEN`, and at least one of
`YARRTUBE_PLEX_PLAYLIST_SECTION_ID` or `YARRTUBE_PLEX_CHANNEL_SECTION_ID` are
set to a non-empty value, and the library layout (see the
`tv-library-layout` capability) is `movie`. Each section variable accepts a
single section id or a comma-separated list.
`YARRTUBE_PLEX_PLAYLIST_SECTION_ID` names the library sections reconciled
against tracked playlists, and `YARRTUBE_PLEX_CHANNEL_SECTION_ID` those
reconciled against tracked channels. When the integration is disabled the
system MUST NOT reconcile any Plex collection. When it is disabled only
because the layout is `tv`, the daemon SHALL log at startup that Plex
collections are not supported in the TV layout. Scanning a downloaded
video's folder (`YARRTUBE_PLEX_VIDEOS_PATH`) SHALL keep working in either
layout.

#### Scenario: Integration disabled

- **WHEN** the daemon starts without `YARRTUBE_PLEX_URL`, without
  `YARRTUBE_PLEX_TOKEN`, or with neither section variable set to a non-empty
  value
- **THEN** no Plex sync task is scheduled and no requests are made to a Plex
  server

#### Scenario: Integration enabled

- **WHEN** the daemon starts in `movie` layout with `YARRTUBE_PLEX_URL`,
  `YARRTUBE_PLEX_TOKEN`, and at least one of
  `YARRTUBE_PLEX_PLAYLIST_SECTION_ID` or `YARRTUBE_PLEX_CHANNEL_SECTION_ID`
  set
- **THEN** a recurring global Plex collections sync task is scheduled

#### Scenario: TV layout with Plex configured

- **WHEN** the daemon starts in `tv` layout with every Plex variable set
- **THEN** no Plex collections sync task is scheduled, a startup log line
  says collections are unsupported in the TV layout, and downloaded videos'
  folders are still scanned in Plex

#### Scenario: Sync task left by an earlier run

- **WHEN** the daemon starts with the integration disabled (no Plex
  variables, or `tv` layout) and a Plex collections sync task scheduled by
  an earlier run is still pending
- **THEN** that task is removed, so no Plex collection is reconciled and the
  task doesn't fail and retry
