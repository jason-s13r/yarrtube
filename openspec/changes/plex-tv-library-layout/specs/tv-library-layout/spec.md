## ADDED Requirements

### Requirement: Library Layout Is Configurable
The system SHALL read the library layout from `YARRTUBE_LIBRARY_LAYOUT`, accepting `movie` or `tv`, case-insensitively. When the variable is unset or empty, the layout SHALL be `movie`, and every file the system writes SHALL be exactly as before this capability. Any other value SHALL make the daemon fail to start with an error naming the variable.

#### Scenario: Variable unset
- **WHEN** the daemon starts without `YARRTUBE_LIBRARY_LAYOUT`
- **THEN** the layout is `movie` and videos are written with `movie.nfo` as before

#### Scenario: TV layout selected
- **WHEN** the daemon starts with `YARRTUBE_LIBRARY_LAYOUT=tv`
- **THEN** the layout is `tv`

#### Scenario: Invalid value
- **WHEN** the daemon starts with `YARRTUBE_LIBRARY_LAYOUT=series`
- **THEN** startup fails with an error naming `YARRTUBE_LIBRARY_LAYOUT`

### Requirement: Show Metadata Written Per Tracked Source
In `tv` layout, every reconcile pass of a tracked channel or playlist SHALL write a `tvshow.nfo` file at the root of its output folder, overwriting any existing one. In `movie` layout no `tvshow.nfo` SHALL be written.

#### Scenario: Channel reconciled in TV layout
- **WHEN** a tracked channel's reconcile pass runs in `tv` layout
- **THEN** its output folder contains a `tvshow.nfo`

#### Scenario: Movie layout
- **WHEN** a tracked channel's reconcile pass runs in `movie` layout
- **THEN** no `tvshow.nfo` is written

### Requirement: Show NFO Content
A `tvshow.nfo` SHALL be well-formed XML with root element `tvshow`, containing `title` (the channel's or playlist's name), `studio` (the channel's name, or the playlist's name for a playlist) and a `uniqueid` of type `youtube` (the YouTube channel ID or playlist ID), with every text value XML-escaped.

#### Scenario: Channel show NFO
- **WHEN** `tvshow.nfo` is written for a channel named "Rock & Roll" with YouTube channel ID `UCabc`
- **THEN** it has root `tvshow`, `title` `Rock &amp; Roll` and `<uniqueid type="youtube">UCabc</uniqueid>`

### Requirement: Show Files Are Best-Effort
A failure to write `tvshow.nfo` or a poster SHALL be logged and SHALL NOT fail the reconcile pass.

#### Scenario: Write failure
- **WHEN** writing `tvshow.nfo` fails during a reconcile pass
- **THEN** the failure is logged and the rest of the pass still runs

### Requirement: Channel Avatar Used As Show Poster
In `tv` layout, a channel's reconcile pass SHALL copy the channel's stored avatar into the root of its output folder as `poster.<ext>`, keeping the avatar's extension, if the channel has a recorded avatar and that poster file does not exist yet. A channel without an avatar, and a playlist, SHALL get no poster.

#### Scenario: Channel with an avatar
- **WHEN** a channel with a stored `jpg` avatar is reconciled in `tv` layout and its folder has no `poster.jpg`
- **THEN** `poster.jpg` holding the avatar's bytes is written at the root of its output folder

#### Scenario: Poster already present
- **WHEN** the channel's output folder already has `poster.jpg`
- **THEN** it is left unchanged

### Requirement: Season Folders
In `tv` layout, each video's files SHALL be written directly into a season folder at the root of its tracked channel's or playlist's output folder, named `Season <season>` with the season zero-padded to 2 digits: `Season 2026` for a channel video, `Season 01` for a playlist video. There SHALL be no folder per video, so each show follows Plex's documented `<show>/Season <season>/<episode file>` structure and its `tvshow.nfo` sits one folder above every episode file.

#### Scenario: Channel video's files
- **WHEN** a channel video published in 2026 is downloaded in `tv` layout
- **THEN** its media file, thumbnail and episode NFO are directly inside `Season 2026`

### Requirement: Season And Episode Numbering
In `tv` layout, the system SHALL number each video from YouTube's publish timestamp (`publishedAt`, UTC) or playlist position:
- A channel video's season SHALL be its publish year, and its episode SHALL be its publish month, day, hour and minute as the number `MMDDhhmm`.
- A playlist video's season SHALL be `1`, and its episode SHALL be its position in the playlist when its first file is written.

#### Scenario: Channel video numbering
- **WHEN** a channel video published at `2026-01-02T15:30:45Z` is numbered
- **THEN** it is numbered season `2026`, episode `1021530`

#### Scenario: Later video the same day
- **WHEN** another video of the same channel published at `2026-01-02T18:05:00Z` is numbered
- **THEN** it is numbered season `2026`, episode `1021805`, after the earlier one

#### Scenario: Playlist video numbering
- **WHEN** a video at position 7 of a tracked playlist is numbered
- **THEN** it is numbered season `1`, episode `7`

### Requirement: Episode File Naming
In `tv` layout, a video's media file SHALL be named `S<season>E<episode> - <title>`, with the season zero-padded to 2 digits, the episode zero-padded to 8 digits for a channel video and 2 digits for a playlist video, and the title sanitized by the `video-naming` capability's rules. The thumbnail and episode NFO SHALL share the media file's base name.

#### Scenario: Channel video downloaded in TV layout
- **WHEN** a channel video titled "My Video", published at `2026-01-02T15:30:45Z`, is downloaded in `tv` layout
- **THEN** it is saved as `Season 2026/S2026E01021530 - My Video.mp4`, with `S2026E01021530 - My Video.jpg` and `S2026E01021530 - My Video.nfo` beside it

#### Scenario: Playlist video downloaded in TV layout
- **WHEN** the video at position 3 of a tracked playlist, titled "Intro", is downloaded in `tv` layout
- **THEN** it is saved as `Season 01/S01E03 - Intro.mp4`

### Requirement: Episode Name Collisions
When another file in a video's season folder already uses its episode base name (two same-titled videos published in the same minute, or a reordered playlist reusing a position for a same-titled video), the video SHALL use the `video-naming` capability's collision fallback as its base name: `<episode name> [<video id>]`.

#### Scenario: Same minute, same title
- **WHEN** `Season 2026/S2026E01021530 - Live.mp4` already belongs to another video and a channel video titled "Live", published in that same minute, is downloaded
- **THEN** it is saved as `Season 2026/S2026E01021530 - Live [<video id>].mp4`

### Requirement: Episode Thumbnail Fetched Ahead In TV Layout
In `tv` layout, a thumbnail fetched ahead of the video's download SHALL be saved in the video's season folder under the episode name, and the later download SHALL reuse that season folder and base name. When the video's YouTube metadata cannot be fetched for this, the thumbnail fetch SHALL be skipped without error and without creating a folder.

#### Scenario: Thumbnail fetched before the download
- **WHEN** a newly added channel video's thumbnail is fetched ahead of its download in `tv` layout
- **THEN** it is saved as `Season <year>/<episode name>.jpg`, and the download later writes `Season <year>/<episode name>.mp4`

#### Scenario: Metadata unavailable for the thumbnail fetch
- **WHEN** a video's thumbnail fetch runs in `tv` layout and its YouTube metadata cannot be fetched
- **THEN** no thumbnail is recorded and no folder is created

### Requirement: Episode Files Handled As One Unit
In `tv` layout, a video's episode files (every file in its season folder named exactly its base name, or starting with its base name followed by `.`) SHALL be the unit that video deletion, failed-download cleanup and orphan cleanup act on, the way a per-video folder is in `movie` layout. A season folder itself SHALL never be deleted by them.

#### Scenario: Downloaded episode removed
- **WHEN** a downloaded channel video stored as `Season 2026/S2026E01021530 - My Video.mp4` ages out of its channel
- **THEN** its `.mp4`, `.jpg` and `.nfo` are deleted, and `Season 2026` and its other episodes' files are kept

### Requirement: Orphan Cleanup Looks Inside Season Folders
Orphan cleanup SHALL look inside each season folder: it SHALL delete episode files that belong to no stored video, and SHALL keep the files of a download or thumbnail fetch in flight, matched by the title after their `S<season>E<episode> - ` prefix.

#### Scenario: Orphaned episode files
- **WHEN** orphan cleanup finds files in `Season 2026` whose base name belongs to no stored video
- **THEN** those files are deleted

#### Scenario: Download in flight
- **WHEN** orphan cleanup finds `Season 2026/S2026E01021530 - My Video.nfo` while "My Video" is still downloading and its files aren't recorded yet
- **THEN** that file is kept

### Requirement: Whole Show Scanned In Plex After A Download
When the Plex folder scan is configured (`YARRTUBE_PLEX_VIDEOS_PATH`) and a `tv` layout video finishes downloading, the system SHALL ask Plex to scan the video's whole show folder rather than only its season folder, so a new show is matched from its `tvshow.nfo` and poster.

#### Scenario: New channel's first episode downloaded
- **WHEN** the first episode of a newly tracked channel finishes downloading in `tv` layout
- **THEN** Plex is asked to scan that channel's output folder

### Requirement: Show Files Survive Orphan Cleanup
Reconciliation's orphan cleanup SHALL never delete `tvshow.nfo`, a `poster.*` file or a `Season <n>` folder at the root of a tracked channel's or playlist's output folder.

#### Scenario: Orphan sweep in a show folder
- **WHEN** orphan cleanup runs on an output folder containing `tvshow.nfo`, `poster.jpg` and an empty `Season 2025`
- **THEN** all three remain

### Requirement: Episode NFO
In `tv` layout, a video's metadata sidecar SHALL be named after its media file's base name with the `.nfo` extension and placed beside it, instead of `movie.nfo`.

#### Scenario: Episode NFO placed
- **WHEN** a video's metadata is generated in `tv` layout and its media file is `S01E03 - Intro.mp4`
- **THEN** `S01E03 - Intro.nfo` is written in the same folder

### Requirement: Episode NFO Content
An episode NFO SHALL have root element `episodedetails` and contain `title`, `season`, `episode`, `plot`, `aired` (publish date `YYYY-MM-DD`), `director` (channel name), a `uniqueid` of type `youtube` holding the video's YouTube ID and, when a thumbnail was saved, `thumb`. It SHALL follow the `video-metadata` capability's field mapping, plot truncation and XML escaping rules.

#### Scenario: Episode NFO written
- **WHEN** the episode NFO of `S2026E01021530 - My Video.mp4`, published at `2026-01-02T15:30:45Z`, is written
- **THEN** it has root `episodedetails`, `season` `2026`, `episode` `1021530`, `aired` `2026-01-02` and a `youtube` `uniqueid`

### Requirement: Layout Switch Does Not Migrate Existing Files
Changing `YARRTUBE_LIBRARY_LAYOUT` SHALL NOT rename, move or rewrite files already downloaded. When metadata is regenerated for an already-downloaded video, its sidecar SHALL follow how its recorded media file was named: an episode NFO beside it when the file's base name starts with an `S<season>E<episode> - ` prefix, otherwise `movie.nfo`.

#### Scenario: Switching an existing install to TV layout
- **WHEN** the daemon restarts in `tv` layout with videos previously downloaded in `movie` layout
- **THEN** those videos' folders, media files and `movie.nfo` files are left untouched, and only new downloads use the season layout

#### Scenario: Metadata repaired for a movie-layout video in TV layout
- **WHEN** a video downloaded in `movie` layout has its missing metadata regenerated while the layout is `tv`
- **THEN** a `movie.nfo` is written for it, not an episode NFO
