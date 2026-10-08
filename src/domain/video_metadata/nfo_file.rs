use super::{EpisodeNumber, VideoMetadata, render_episode_nfo, render_movie_nfo};
use crate::domain::video::is_season_dir;
use std::path::Path;

pub const MOVIE_NFO_FILENAME: &str = "movie.nfo";

/// A video's metadata sidecar: its filename (`movie.nfo`, or an episode NFO
/// named after the media file) and its rendered XML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NfoFile {
    pub filename: String,
    pub content: String,
}

impl NfoFile {
    /// `movie.nfo`.
    pub fn movie(metadata: &VideoMetadata) -> Self {
        Self {
            filename: MOVIE_NFO_FILENAME.to_string(),
            content: render_movie_nfo(metadata),
        }
    }

    /// `"<media_basename>.nfo"`, rooted at `episodedetails`.
    pub fn episode(metadata: &VideoMetadata, episode: EpisodeNumber, media_basename: &str) -> Self {
        Self {
            filename: format!("{media_basename}.nfo"),
            content: render_episode_nfo(metadata, episode),
        }
    }

    /// An episode NFO when the recorded media file (relative to its show's
    /// output dir) lives in a season folder and its basename has an episode
    /// prefix, else `movie.nfo`, so a sidecar always matches how its file
    /// was named, whatever the current layout.
    pub fn for_media_file(metadata: &VideoMetadata, media_filename: &str) -> Self {
        let in_season_dir = media_filename
            .split_once('/')
            .is_some_and(|(first, _)| is_season_dir(first));
        let basename = Path::new(media_filename)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default();
        match EpisodeNumber::parse_prefix(basename).filter(|_| in_season_dir) {
            Some(episode) => Self::episode(metadata, episode, basename),
            None => Self::movie(metadata),
        }
    }
}
