## MODIFIED Requirements

### Requirement: Metadata File Named movie.nfo
In `movie` layout (see the `tv-library-layout` capability), the system SHALL write a downloaded video's metadata sidecar as `movie.nfo` in that video's own output folder. In `tv` layout, the system SHALL instead write it as an episode NFO named after the video's media file, as the `tv-library-layout` capability's "Episode NFO" requirement defines. Wherever this capability refers to `movie.nfo`, it SHALL mean the sidecar for the active layout.

#### Scenario: Metadata generated
- **WHEN** a video's metadata is successfully generated in `movie` layout
- **THEN** it is written to a file named `movie.nfo` in that video's own folder

#### Scenario: Metadata generated in TV layout
- **WHEN** a video's metadata is successfully generated in `tv` layout and its media file is `S01E03 - Intro.mp4`
- **THEN** it is written to `S01E03 - Intro.nfo` in that video's own folder, and no `movie.nfo` is written

### Requirement: Metadata Generation Failure Does Not Affect Download Outcome
In `movie` layout, the system SHALL NOT fail, retry, or otherwise alter a video download's outcome when generating its metadata fails; a video download's success is determined solely by whether its file was downloaded. When the YouTube metadata cannot be fetched before the download, the system SHALL still download the video and SHALL try to generate its metadata once more after the download succeeds. In `tv` layout, the YouTube metadata is needed to name the media file, so when it cannot be fetched before the download, the system SHALL NOT run the download and SHALL fail the attempt retryably, the same way as a failed download. On a metadata generation failure, the system SHALL write no `movie.nfo` file and record no completed metadata for that video. A download that does not succeed SHALL NOT record metadata as generated and SHALL NOT leave a `movie.nfo` file in the video's folder.

#### Scenario: YouTube metadata fetch fails after a successful download
- **WHEN** a video's YouTube metadata cannot be fetched before or after its download in `movie` layout, and its file finishes downloading successfully
- **THEN** the video is still marked as successfully downloaded, no `movie.nfo` file is written, and no metadata is recorded as generated for it

#### Scenario: Metadata fetch fails before the download but succeeds after it
- **WHEN** a video's YouTube metadata cannot be fetched before its download in `movie` layout, its file finishes downloading successfully, and the metadata can be fetched afterwards
- **THEN** its `movie.nfo` file is generated in its own folder and its metadata is recorded as generated

#### Scenario: Metadata fetch fails before the download in TV layout
- **WHEN** a video's YouTube metadata cannot be fetched before its download in `tv` layout
- **THEN** `yt-dlp` is not run, the video is marked errored for retry, and no new folder or sidecar is created for it

#### Scenario: Download fails after movie.nfo was written
- **WHEN** a video's `movie.nfo` was written before its download and the download then fails
- **THEN** no metadata is recorded as generated for that video and no `movie.nfo` file remains in its folder
