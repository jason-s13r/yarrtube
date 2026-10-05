/// The request header the YouTube Data API reads the API key from. Sending
/// the key here rather than as a `key` query parameter keeps it out of every
/// request URL, and so out of any error message (and log line, and recorded
/// task error) built from one.
pub const YOUTUBE_API_KEY_HEADER: &str = "X-Goog-Api-Key";
