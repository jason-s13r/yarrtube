use super::errors::ValidationError;

/// How downloaded videos are laid out on disk for a media server: `Movie`
/// (one `movie.nfo` folder per video, the default) or `Tv` (each tracked
/// channel/playlist is a show, each video an episode in a season folder).
/// One daemon-wide choice, read from `YARRTUBE_LIBRARY_LAYOUT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryLayout {
    #[default]
    Movie,
    Tv,
}

impl LibraryLayout {
    /// `None`/empty → `Movie`; case-insensitive `"movie"`/`"tv"`; anything
    /// else is rejected with an error naming the variable.
    pub fn parse(value: Option<&str>) -> Result<Self, ValidationError> {
        let value = value.unwrap_or_default();
        match value.to_lowercase().as_str() {
            "" | "movie" => Ok(Self::Movie),
            "tv" => Ok(Self::Tv),
            _ => Err(ValidationError(format!(
                "YARRTUBE_LIBRARY_LAYOUT must be \"movie\" or \"tv\" (got \"{value}\")"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_default_to_the_movie_layout_when_unset() {
        assert_eq!(
            [LibraryLayout::parse(None), LibraryLayout::parse(Some(""))],
            [Ok(LibraryLayout::Movie), Ok(LibraryLayout::Movie)]
        );
    }

    #[test]
    fn it_should_parse_the_tv_layout_case_insensitively() {
        assert_eq!(
            [
                LibraryLayout::parse(Some("tv")),
                LibraryLayout::parse(Some("TV")),
                LibraryLayout::parse(Some("Movie")),
            ],
            [
                Ok(LibraryLayout::Tv),
                Ok(LibraryLayout::Tv),
                Ok(LibraryLayout::Movie)
            ]
        );
    }

    #[test]
    fn it_should_reject_an_unknown_layout() {
        assert_eq!(
            LibraryLayout::parse(Some("series")),
            Err(ValidationError(
                "YARRTUBE_LIBRARY_LAYOUT must be \"movie\" or \"tv\" (got \"series\")".to_string()
            ))
        );
    }
}
