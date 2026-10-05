## ADDED Requirements

### Requirement: YouTube API Key Never Logged
The system SHALL NOT include the YouTube Data API key in any log output or in
any error recorded for a task, including errors raised when a YouTube API
request fails before receiving a response (for example a DNS, connection or
timeout failure) and errors carrying a YouTube API error response.

#### Scenario: YouTube API request fails to connect
- **WHEN** a YouTube Data API request fails before a response arrives, for example because the host name cannot be resolved
- **THEN** the logged error and the task's recorded error describe the failure without containing the API key

#### Scenario: YouTube API returns an error response
- **WHEN** a YouTube Data API request receives a non-success response
- **THEN** the logged error and the task's recorded error contain the status and response body without containing the API key
