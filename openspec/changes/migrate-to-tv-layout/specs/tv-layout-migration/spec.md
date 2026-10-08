## ADDED Requirements

### Requirement: Existing Downloads Migrated In TV Layout
In `tv` layout, every reconcile pass of a tracked channel or playlist SHALL migrate each of its `Downloaded` videos whose recorded media file isn't directly inside a `Season <n>` folder into the flat tv layout, without downloading it again. The video SHALL be numbered and named exactly as a new download in `tv` layout would be, including the episode name collision fallback.

#### Scenario: Movie-layout video migrated
- **WHEN** a channel's video "My Video", published at `2026-01-02T15:30:45Z`, is stored as `My Video/My Video.mp4` with `My Video/My Video.jpg` and `My Video/movie.nfo`, and the channel is reconciled in `tv` layout
- **THEN** it is stored as `Season 2026/S2026E01021530 - My Video.mp4` with `S2026E01021530 - My Video.jpg` and `S2026E01021530 - My Video.nfo` beside it, and the `My Video` folder is gone

#### Scenario: First-build episode folder migrated
- **WHEN** a video is stored as `Season 2026/S2026E01021530 - My Video/S2026E01021530 - My Video.mp4` and its show is reconciled in `tv` layout
- **THEN** it is stored as `Season 2026/S2026E01021530 - My Video.mp4` with its thumbnail and episode NFO beside it, and the episode folder is gone

#### Scenario: Legacy flat video migrated
- **WHEN** a video is stored as a bare `My Video.mp4` at the root of its playlist's folder and the playlist is reconciled in `tv` layout, the video being at position 3
- **THEN** it is stored as `Season 01/S01E03 - My Video.mp4`

#### Scenario: Already flat
- **WHEN** a video is stored as `Season 2026/S2026E01021530 - My Video.mp4`
- **THEN** reconciliation leaves its files as they are

#### Scenario: Movie layout
- **WHEN** a channel is reconciled in `movie` layout
- **THEN** none of its videos' files are moved

### Requirement: Migration Keeps The Video's Record Consistent
After migrating a video, its recorded media filename and thumbnail filename SHALL name the moved files, and its metadata SHALL be recorded with an episode NFO that references the moved thumbnail. Its status, quality, duration and watch state SHALL be unchanged.

#### Scenario: Watched video migrated
- **WHEN** a watched video with a recorded duration is migrated
- **THEN** it is still watched, with the same duration and playback position, and its recorded filename is the moved media file

### Requirement: Migration Is Best-Effort
Migrating a video SHALL never fail its reconcile pass. When a video's publish time can't be determined (no stored metadata and the YouTube metadata can't be fetched) it SHALL be left as it is. When moving its files fails, the failure SHALL be logged, its record SHALL only name files that exist, and the migration SHALL be tried again on the next pass.

#### Scenario: Publish time unavailable
- **WHEN** a movie-layout video has no stored metadata and its YouTube metadata can't be fetched during a `tv` layout reconcile pass
- **THEN** its files and record are left unchanged and the rest of the pass still runs

#### Scenario: Move fails
- **WHEN** moving a video's media file fails during migration
- **THEN** the failure is logged, the video's record still names its original file, and the rest of the pass still runs

### Requirement: Videos In Flight Not Migrated
Only `Downloaded` videos SHALL be migrated. A video that is pending, downloading, errored or excluded SHALL be left to its download, which writes it in the tv layout.

#### Scenario: Video downloading
- **WHEN** a video is being re-downloaded during a `tv` layout reconcile pass
- **THEN** its files are not moved by migration
