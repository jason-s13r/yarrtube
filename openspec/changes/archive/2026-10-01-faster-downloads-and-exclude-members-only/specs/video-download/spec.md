## MODIFIED Requirements

### Requirement: Video Status Reflects Download Progress
The system SHALL track a video's status through its download lifecycle: pending, in progress, downloaded, errored-but-retrying, permanently errored, or excluded. A video becomes excluded when a download attempt fails for a reason that can never succeed on a later attempt (see "Permanently Unavailable Videos Are Excluded"); excluded is a terminal status that is never retried or recovered.

#### Scenario: Download starts
- **WHEN** the system begins downloading a pending video
- **THEN** the video's status becomes in-progress

#### Scenario: Download succeeds
- **WHEN** a video download completes successfully
- **THEN** the video's status becomes downloaded

#### Scenario: Download fails with attempts remaining
- **WHEN** a video download attempt fails and the system will still retry it
- **THEN** the video's status becomes errored-but-retrying

#### Scenario: Download fails permanently
- **WHEN** a video download has failed and the system will not retry it again
- **THEN** the video's status becomes permanently errored

#### Scenario: Download fails for a diagnosed permanent reason
- **WHEN** a video download attempt fails and the diagnosed reason (see "Failed Downloads Are Diagnosed") is one that can never succeed later — members-only content, a copyright/claimed-content block, a region block, a private video, a removed video, or a terminated account
- **THEN** the video's status becomes excluded, regardless of how many attempts remained

### Requirement: Automatic Retry On Download Failure
The system SHALL automatically retry a failed video download a bounded number of times before giving up on that attempt sequence, without any manual action. Exhausting that bounded sequence SHALL NOT prevent the video from being attempted again later by playlist or channel reconciliation. A failure that marks the video excluded (see "Permanently Unavailable Videos Are Excluded") SHALL NOT be retried within the attempt sequence and SHALL NOT be re-attempted later.

#### Scenario: Transient download failure
- **WHEN** a video download attempt fails and retries remain
- **THEN** the system automatically attempts the download again after a delay

#### Scenario: Retries exhausted
- **WHEN** a video download has failed on every allowed attempt
- **THEN** the system stops attempting that download within that attempt sequence, though the video may be attempted again later by playlist or channel reconciliation

#### Scenario: Permanently-unavailable failure is not retried
- **WHEN** a video download attempt fails for a permanently-unavailable reason
- **THEN** the system does not retry that download within the attempt sequence and does not re-attempt the video later

## ADDED Requirements

### Requirement: Failed Downloads Are Diagnosed
The system SHALL, whenever a video download attempt fails cleanly (`yt-dlp` ran and exited non-zero), determine a precise reason for the failure beyond the generic message the download itself reported, and record it so an operator can see why the download failed. The system SHALL obtain this reason even when the download's own error was generic (e.g. a bare "Video unavailable"). When no precise reason can be determined, the system SHALL record that the reason is undetermined rather than inventing one.

#### Scenario: A failed download's precise reason is recorded
- **WHEN** a video download attempt fails cleanly
- **THEN** the system determines and records a precise failure reason for that attempt

#### Scenario: A generic download error is still diagnosed
- **WHEN** a video download attempt fails with only a generic error (such as a bare "Video unavailable")
- **THEN** the system still determines and records the precise underlying reason when one is available

#### Scenario: No precise reason can be determined
- **WHEN** a video download attempt fails and no precise reason can be determined
- **THEN** the system records the reason as undetermined and does not treat the failure as permanent

### Requirement: Permanently Unavailable Videos Are Excluded
The system SHALL mark a video excluded, instead of errored, when the diagnosed reason for its failed download (see "Failed Downloads Are Diagnosed") is one that can never succeed on a later attempt. The reasons the system SHALL treat as permanently unavailable are: members-only content (requiring a paid channel membership `yt-dlp` cannot satisfy), a copyright or claimed-content block, a region/country block, a private video, a removed or no-longer-available video, and a video whose associated account has been terminated. An excluded video's download attempt SHALL be treated as a settled, non-failing outcome: the download task SHALL complete without error so the task queue neither retries nor dead-letters it. A failure whose diagnosed reason is NOT one of these — including an undetermined or merely generic reason (such as a bare "Video unavailable" with no further detail), or a network, timeout, HTTP server, rate-limit, bot-verification, or fragment/merge error — SHALL NOT be treated as permanently unavailable, and SHALL remain errored and subject to the normal retry and reconcile-recovery behavior.

#### Scenario: A diagnosed permanent reason excludes the video
- **WHEN** a video download attempt fails and the diagnosed reason is one of the permanently-unavailable reasons (members-only, copyright/claimed-content block, region block, private, removed, or terminated account)
- **THEN** the system marks the video excluded and completes the download task without a failure, so no retry is scheduled and the task is not dead-lettered

#### Scenario: A generic "Video unavailable" with no diagnosed cause is not excluded
- **WHEN** a video download attempt fails and the diagnosed reason is undetermined or merely a generic "Video unavailable" with no specific permanent cause
- **THEN** the system marks the video errored (retrying or permanently, per the retry policy) and does not exclude it, so it remains eligible for retry and 24h reconcile recovery

#### Scenario: A recoverable failure is not excluded
- **WHEN** a video download attempt fails for a transient reason such as a network, timeout, HTTP server, rate-limit, bot-verification, or fragment error
- **THEN** the system marks the video errored (retrying or permanently, per the retry policy) and does not exclude it
