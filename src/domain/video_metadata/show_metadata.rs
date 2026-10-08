use crate::domain::channel::Channel;
use crate::domain::playlist::Playlist;

/// The file a TV-layout show's metadata is written to, at the root of its
/// output folder.
pub const TVSHOW_NFO_FILENAME: &str = "tvshow.nfo";

/// What a TV-layout show's `tvshow.nfo` holds, for a tracked channel or
/// playlist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowMetadata {
    pub title: String,
    pub studio: String,
    pub uniqueid: String,
}

impl ShowMetadata {
    /// `uniqueid` is the YouTube channel ID.
    pub fn for_channel(channel: &Channel) -> Self {
        Self {
            title: channel.name.clone(),
            studio: channel.name.clone(),
            uniqueid: channel.youtube_channel_id.clone(),
        }
    }

    /// `uniqueid` is the YouTube playlist ID.
    pub fn for_playlist(playlist: &Playlist) -> Self {
        Self {
            title: playlist.name.as_str().to_string(),
            studio: playlist.name.as_str().to_string(),
            uniqueid: playlist.id.as_str().to_string(),
        }
    }
}
