## ADDED Requirements

### Requirement: TV Layout Downloads Migrated In Movie Layout
In `movie` layout, every reconcile pass of a tracked channel or playlist SHALL migrate each of its `Downloaded` videos whose recorded media file is inside a `Season <n>` folder into the movie layout, without downloading it again: its own folder named from its sanitized title, with the `video-naming` capability's collision fallback, and its media file and thumbnail named after that folder.

#### Scenario: Flat episode migrated
- **WHEN** a video titled "My Video" is stored as `Season 2026/S2026E01021530 - My Video.mp4` with `.jpg` and `.nfo` beside it, and its channel is reconciled in `movie` layout
- **THEN** it is stored as `My Video/My Video.mp4` with `My Video/My Video.jpg` and `My Video/movie.nfo`, and its `S2026E01021530 - My Video.*` files are gone

#### Scenario: First-build episode folder migrated
- **WHEN** a video is stored as `Season 2026/S2026E01021530 - My Video/S2026E01021530 - My Video.mp4` and its show is reconciled in `movie` layout
- **THEN** it is stored as `My Video/My Video.mp4`, and the episode folder is gone

#### Scenario: Folder name taken
- **WHEN** another video's folder `My Video` already exists and a tv-layout video titled "My Video" is migrated
- **THEN** it is stored as `My Video [<video id>]/My Video [<video id>].mp4`

#### Scenario: Already in movie layout
- **WHEN** a video is stored as `My Video/My Video.mp4`
- **THEN** reconciliation leaves its files as they are

#### Scenario: TV layout
- **WHEN** a channel is reconciled in `tv` layout
- **THEN** none of its videos are moved back into the movie layout

### Requirement: Movie Migration Keeps The Video's Record Consistent
After migrating a video, its recorded media filename and thumbnail filename SHALL name the moved files and its status, quality, duration and watch state SHALL be unchanged. When its metadata is known (stored, else fetched from YouTube) a `movie.nfo` referencing the moved thumbnail SHALL be written and the metadata recorded; otherwise the video SHALL still be migrated, without a `movie.nfo`, which metadata repair writes later.

#### Scenario: Watched video migrated
- **WHEN** a watched video with a recorded duration is migrated
- **THEN** it is still watched, with the same duration and playback position, and its recorded filename is the moved media file

#### Scenario: Metadata unknown
- **WHEN** a tv-layout video has no stored metadata and its YouTube metadata can't be fetched
- **THEN** it is still moved into its movie-layout folder, and no `movie.nfo` is written

### Requirement: Empty Season Folders Removed
After migrating a show's videos in `movie` layout, a `Season <n>` folder at its root left with no files SHALL be removed. A season folder still holding files SHALL be kept.

#### Scenario: Last episode of a season migrated
- **WHEN** the only video in `Season 2025` is migrated into the movie layout
- **THEN** `Season 2025` is removed

### Requirement: Movie Migration Is Best-Effort
Migrating a video back into the movie layout SHALL never fail its reconcile pass. When moving its files fails, the failure SHALL be logged, its record SHALL only name files that exist, and the migration SHALL be tried again on the next pass. Only `Downloaded` videos SHALL be migrated.

#### Scenario: Move fails
- **WHEN** moving a video's media file fails during migration
- **THEN** the failure is logged, the video's record still names its original file, and the rest of the pass still runs

#### Scenario: Video not downloaded
- **WHEN** an excluded video still has a recorded file inside a season folder
- **THEN** migration doesn't move it
