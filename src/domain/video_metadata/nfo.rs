use super::{EpisodeNumber, ShowMetadata, VideoMetadata};
use quick_xml::escape::escape;

fn element(tag: &str, value: &str) -> String {
    format!("<{tag}>{}</{tag}>", escape(value))
}

/// Renders `movie.nfo`'s XML content from a video's `VideoMetadata`, with
/// every text value escaped so the result is always well-formed XML
/// regardless of characters present in the source fields.
pub fn render_movie_nfo(metadata: &VideoMetadata) -> String {
    let mut body = String::new();
    body.push_str(&element("title", &metadata.title));
    body.push_str(&element("sorttitle", &metadata.sorttitle));
    body.push_str(&element("plot", &metadata.plot));
    body.push_str(&element("studio", &metadata.studio));
    body.push_str(&element("director", &metadata.director));
    body.push_str(&element("premiered", &metadata.premiered()));
    body.push_str(&element("year", &metadata.year().to_string()));
    if let Some(genre) = &metadata.genre {
        body.push_str(&element("genre", genre));
    }
    for tag in &metadata.tags {
        body.push_str(&element("tag", tag));
    }
    body.push_str(&youtube_uniqueid(&metadata.uniqueid));
    if let Some(thumb) = &metadata.thumb {
        body.push_str(&element("thumb", thumb));
    }

    format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><movie>{body}</movie>"#)
}

/// Renders a TV-layout episode NFO's XML content (root `episodedetails`),
/// escaped like `render_movie_nfo`.
pub fn render_episode_nfo(metadata: &VideoMetadata, episode: EpisodeNumber) -> String {
    let mut body = String::new();
    body.push_str(&element("title", &metadata.title));
    body.push_str(&element("season", &episode.season.to_string()));
    body.push_str(&element("episode", &episode.episode.to_string()));
    body.push_str(&element("plot", &metadata.plot));
    body.push_str(&element("aired", &metadata.premiered()));
    body.push_str(&element("director", &metadata.director));
    body.push_str(&youtube_uniqueid(&metadata.uniqueid));
    if let Some(thumb) = &metadata.thumb {
        body.push_str(&element("thumb", thumb));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><episodedetails>{body}</episodedetails>"#
    )
}

/// Renders a TV-layout show's `tvshow.nfo` XML content (root `tvshow`),
/// escaped like `render_movie_nfo`.
pub fn render_tvshow_nfo(show: &ShowMetadata) -> String {
    let mut body = String::new();
    body.push_str(&element("title", &show.title));
    body.push_str(&element("studio", &show.studio));
    body.push_str(&youtube_uniqueid(&show.uniqueid));

    format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><tvshow>{body}</tvshow>"#)
}

fn youtube_uniqueid(id: &str) -> String {
    format!(r#"<uniqueid type="youtube">{}</uniqueid>"#, escape(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel::{Channel, ChannelHandle, VideoLimit};
    use crate::domain::playlist::PlaylistPath;
    use crate::domain::shared::Quality;
    use crate::domain::video_metadata::NfoFile;
    use chrono::{DateTime, Utc};

    fn metadata() -> VideoMetadata {
        VideoMetadata::new(
            "My Video",
            "A description",
            "My Channel",
            "My Channel",
            DateTime::parse_from_rfc3339("2024-01-02T03:04:05Z")
                .unwrap()
                .with_timezone(&Utc),
            Some("Music".to_string()),
            vec!["tag1".to_string(), "tag2".to_string()],
            "yt1",
            Some("My Video.jpg".to_string()),
            "0001 My Video",
            DateTime::<Utc>::UNIX_EPOCH,
        )
    }

    #[test]
    fn it_should_render_a_well_formed_episode_nfo() {
        let metadata = VideoMetadata {
            title: "A & <b> \"q\" 'a'".to_string(),
            thumb: Some("S2024E01020304 - My Video.jpg".to_string()),
            ..metadata()
        };
        let episode = EpisodeNumber::for_channel_video(metadata.published_at);

        let nfo = NfoFile::episode(&metadata, episode, "S2024E01020304 - My Video");

        parses_as_valid_xml(&nfo.content);
        assert_eq!(
            nfo,
            NfoFile {
                filename: "S2024E01020304 - My Video.nfo".to_string(),
                content: concat!(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><episodedetails>"#,
                    "<title>A &amp; &lt;b&gt; &quot;q&quot; &apos;a&apos;</title>",
                    "<season>2024</season>",
                    "<episode>1020304</episode>",
                    "<plot>A description</plot>",
                    "<aired>2024-01-02</aired>",
                    "<director>My Channel</director>",
                    r#"<uniqueid type="youtube">yt1</uniqueid>"#,
                    "<thumb>S2024E01020304 - My Video.jpg</thumb>",
                    "</episodedetails>"
                )
                .to_string(),
            }
        );
    }

    #[test]
    fn it_should_render_a_well_formed_tvshow_nfo() {
        let channel = Channel::create(
            ChannelHandle::new("@rock").unwrap(),
            "Rock & Roll",
            "UCabc",
            Quality::High,
            VideoLimit::new(10).unwrap(),
            PlaylistPath::new("channels/rock").unwrap(),
            None,
            DateTime::<Utc>::UNIX_EPOCH,
        );

        let xml = render_tvshow_nfo(&ShowMetadata::for_channel(&channel));

        parses_as_valid_xml(&xml);
        assert_eq!(
            xml,
            concat!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><tvshow>"#,
                "<title>Rock &amp; Roll</title>",
                "<studio>Rock &amp; Roll</studio>",
                r#"<uniqueid type="youtube">UCabc</uniqueid>"#,
                "</tvshow>"
            )
        );
    }

    fn parses_as_valid_xml(xml: &str) {
        let mut reader = quick_xml::Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("generated movie.nfo is not well-formed XML: {e}"),
            }
        }
    }

    #[test]
    fn it_should_render_every_field_when_present() {
        let xml = render_movie_nfo(&metadata());

        parses_as_valid_xml(&xml);
        assert!(xml.contains("<title>My Video</title>"));
        assert!(xml.contains("<sorttitle>0001 My Video</sorttitle>"));
        assert!(xml.contains("<plot>A description</plot>"));
        assert!(xml.contains("<studio>My Channel</studio>"));
        assert!(xml.contains("<director>My Channel</director>"));
        assert!(xml.contains("<premiered>2024-01-02</premiered>"));
        assert!(xml.contains("<year>2024</year>"));
        assert!(xml.contains("<genre>Music</genre>"));
        assert!(xml.contains("<tag>tag1</tag>"));
        assert!(xml.contains("<tag>tag2</tag>"));
        assert!(xml.contains(r#"<uniqueid type="youtube">yt1</uniqueid>"#));
        assert!(xml.contains("<thumb>My Video.jpg</thumb>"));
    }

    #[test]
    fn it_should_omit_thumb_when_absent() {
        let metadata = VideoMetadata {
            thumb: None,
            ..metadata()
        };

        let xml = render_movie_nfo(&metadata);

        parses_as_valid_xml(&xml);
        assert!(!xml.contains("<thumb>"));
    }

    #[test]
    fn it_should_omit_tag_elements_when_there_are_no_tags() {
        let metadata = VideoMetadata {
            tags: Vec::new(),
            ..metadata()
        };

        let xml = render_movie_nfo(&metadata);

        parses_as_valid_xml(&xml);
        assert!(!xml.contains("<tag>"));
    }

    #[test]
    fn it_should_omit_genre_when_unmapped() {
        let metadata = VideoMetadata {
            genre: None,
            ..metadata()
        };

        let xml = render_movie_nfo(&metadata);

        parses_as_valid_xml(&xml);
        assert!(!xml.contains("<genre>"));
    }

    #[test]
    fn it_should_escape_xml_significant_characters_and_remain_well_formed() {
        let metadata = VideoMetadata {
            title: "Title & <tags> \"quoted\" 'apos'".to_string(),
            plot: "Plot with & < > \" '".to_string(),
            tags: vec!["a&b".to_string()],
            ..metadata()
        };

        let xml = render_movie_nfo(&metadata);

        parses_as_valid_xml(&xml);
        assert!(!xml.contains("<tags>"));
        assert!(xml.contains("&amp;"));
        assert!(xml.contains("&lt;"));
        assert!(xml.contains("&gt;"));
        assert!(xml.contains("&quot;"));
        assert!(xml.contains("&apos;"));
    }
}
