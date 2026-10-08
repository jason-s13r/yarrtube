use chrono::{DateTime, Datelike, Timelike, Utc};

/// A video's season and episode in the TV library layout: from its publish
/// time for a channel video (season = year, episode = `MMDDhhmm`), or from
/// its playlist position for a playlist video (season 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpisodeNumber {
    pub season: u32,
    pub episode: u32,
    episode_width: usize,
}

impl EpisodeNumber {
    /// season = UTC year, episode = `MMDDhhmm` (width 8).
    pub fn for_channel_video(published_at: DateTime<Utc>) -> Self {
        Self {
            season: published_at.year() as u32,
            episode: published_at.month() * 1_000_000
                + published_at.day() * 10_000
                + published_at.hour() * 100
                + published_at.minute(),
            episode_width: 8,
        }
    }

    /// season = 1, episode = position (width 2).
    pub fn for_playlist_video(position: i64) -> Self {
        Self {
            season: 1,
            episode: position.max(0) as u32,
            episode_width: 2,
        }
    }

    /// The playlist position when `Some`, else the publish time — the same
    /// rule as `resolve_sorttitle`.
    pub fn resolve(published_at: DateTime<Utc>, sort_position: Option<i64>) -> Self {
        match sort_position {
            Some(position) => Self::for_playlist_video(position),
            None => Self::for_channel_video(published_at),
        }
    }

    /// `"Season 2026"` / `"Season 01"`.
    pub fn season_dir(&self) -> String {
        format!("Season {:02}", self.season)
    }

    /// `"S2026E01021530"` / `"S01E03"`.
    pub fn code(&self) -> String {
        format!(
            "S{:02}E{:0width$}",
            self.season,
            self.episode,
            width = self.episode_width
        )
    }

    /// Reads a folder/file name's `"S<n>E<n> - "` prefix; `None` for a
    /// movie-layout name.
    pub fn parse_prefix(name: &str) -> Option<Self> {
        let (code, _) = name.split_once(EPISODE_PREFIX_SEPARATOR)?;
        let (season, episode) = code.strip_prefix('S')?.split_once('E')?;
        Some(Self {
            season: parse_digits(season)?,
            episode: parse_digits(episode)?,
            episode_width: episode.len(),
        })
    }
}

/// Separates an episode name's `S…E…` code from its title.
pub const EPISODE_PREFIX_SEPARATOR: &str = " - ";

/// A non-empty run of ASCII digits as a number; `None` for anything else
/// (including a sign, which `str::parse` would accept).
fn parse_digits(digits: &str) -> Option<u32> {
    (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
        .then(|| digits.parse().ok())
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::video::strip_episode_prefix;

    fn published_at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-01-02T15:30:45Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn it_should_number_a_channel_video_by_publish_year_and_minute() {
        let episode = EpisodeNumber::resolve(published_at(), None);

        assert_eq!(
            (
                episode.season,
                episode.episode,
                episode.season_dir(),
                episode.code()
            ),
            (
                2026,
                1_021_530,
                "Season 2026".to_string(),
                "S2026E01021530".to_string()
            )
        );
    }

    #[test]
    fn it_should_number_a_playlist_video_by_its_position() {
        let episode = EpisodeNumber::resolve(published_at(), Some(3));

        assert_eq!(
            (
                episode.season,
                episode.episode,
                episode.season_dir(),
                episode.code()
            ),
            (1, 3, "Season 01".to_string(), "S01E03".to_string())
        );
    }

    #[test]
    fn it_should_parse_the_episode_prefix_of_a_name() {
        assert_eq!(
            (
                EpisodeNumber::parse_prefix("S2026E01021530 - My Video"),
                EpisodeNumber::parse_prefix("S01E03 - Intro [vid1]"),
                EpisodeNumber::parse_prefix("My Video"),
                EpisodeNumber::parse_prefix("S01E03 Intro"),
                strip_episode_prefix("S2026E01021530 - My Video"),
                strip_episode_prefix("My Video"),
            ),
            (
                Some(EpisodeNumber::resolve(published_at(), None)),
                Some(EpisodeNumber::for_playlist_video(3)),
                None,
                None,
                Some("My Video"),
                None,
            )
        );
    }
}
