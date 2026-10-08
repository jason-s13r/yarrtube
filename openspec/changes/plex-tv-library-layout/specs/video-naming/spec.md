## MODIFIED Requirements

### Requirement: Video ID Omitted By Default
The system SHALL NOT include the video's YouTube ID in the folder or filenames derived from its title.

#### Scenario: No naming collision
- **WHEN** a video is downloaded and its sanitized title does not collide with an entry already present in the container's output directory
- **THEN** the video's own output folder, and the file saved inside it, are named based solely on the sanitized title, without the video ID

#### Scenario: No naming collision in TV layout
- **WHEN** a video is downloaded in `tv` layout (see the `tv-library-layout` capability) and no file in its season folder already uses its episode name
- **THEN** its files are named `S<season>E<episode> - <sanitized title>`, without the video ID

### Requirement: Collision Fallback Appends Video ID
The system SHALL detect when a derived name is already taken and in that case SHALL append the video's YouTube ID to it, as `<name> [<video id>]`, to disambiguate it. In `movie` layout the derived name is the video's folder name, taken when an entry (file or folder) with that name exists in the container's output directory. In `tv` layout it is the episode base name, taken when a file in the video's season folder is named after it (that name itself, or that name followed by `.` and an extension).

#### Scenario: Two videos sanitize to the same filename in one playlist
- **WHEN** a video's sanitized title matches the name of an entry already present in the same playlist's or channel's output directory
- **THEN** the system appends the video's YouTube ID to its own output folder's name before saving into it, so the two videos' folders do not collide

#### Scenario: Two episodes share a name in one season folder
- **WHEN** in `tv` layout a video's episode name is `S2026E01021530 - Live` and `Season 2026/S2026E01021530 - Live.mp4` already belongs to another video
- **THEN** the video's files are named `S2026E01021530 - Live [<video id>]`, so the two episodes' files do not collide
