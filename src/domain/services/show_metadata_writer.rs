use crate::domain::channel::Channel;
use crate::domain::playlist::Playlist;
use crate::domain::shared::LibraryLayout;
use crate::domain::video::resolve_output_dir;
use crate::domain::video_metadata::{ShowMetadata, render_tvshow_nfo};
use crate::infrastructure::repositories::filesystem_show_metadata_repository::ShowMetadataRepository;
use std::path::Path;
use std::sync::Arc;
use tracing::warn;

/// Writes a TV-layout show's own files (`tvshow.nfo`, and a channel's
/// poster) at the root of its output folder (`videos_path` joined with the
/// channel's or playlist's path), once per reconcile pass. A no-op in the
/// movie layout.
pub struct ShowMetadataWriter {
    layout: LibraryLayout,
    show_metadata_repository: Arc<dyn ShowMetadataRepository>,
    videos_path: String,
}

impl ShowMetadataWriter {
    pub fn new(
        layout: LibraryLayout,
        show_metadata_repository: Arc<dyn ShowMetadataRepository>,
        videos_path: impl Into<String>,
    ) -> Self {
        Self {
            layout,
            show_metadata_repository,
            videos_path: videos_path.into(),
        }
    }
}

pub trait ShowMetadataWriterApi: Send + Sync {
    /// Writes `tvshow.nfo` and, when the channel has a stored avatar, its
    /// poster. A no-op in the movie layout. Best-effort: failures are
    /// logged, never returned.
    fn write_for_channel(&self, channel: &Channel);

    /// Writes `tvshow.nfo` only: a playlist has no stored image. A no-op in
    /// the movie layout. Best-effort, like `write_for_channel`.
    fn write_for_playlist(&self, playlist: &Playlist);
}

impl ShowMetadataWriterApi for ShowMetadataWriter {
    fn write_for_channel(&self, channel: &Channel) {
        self.write_show(
            channel.path.as_str(),
            &ShowMetadata::for_channel(channel),
            channel.avatar_filename.as_deref(),
        );
    }

    fn write_for_playlist(&self, playlist: &Playlist) {
        self.write_show(
            playlist.path.as_str(),
            &ShowMetadata::for_playlist(playlist),
            None,
        );
    }
}

impl ShowMetadataWriter {
    /// In the TV layout, writes `show`'s `tvshow.nfo` and, given an avatar,
    /// its poster into the show folder at `path` under the videos root.
    fn write_show(&self, path: &str, show: &ShowMetadata, avatar_filename: Option<&str>) {
        if self.layout != LibraryLayout::Tv {
            return;
        }
        let output_dir = resolve_output_dir(&self.videos_path, path);
        self.write_tvshow_nfo(&output_dir, &render_tvshow_nfo(show));
        if let Some(avatar_filename) = avatar_filename {
            self.write_poster(&output_dir, avatar_filename);
        }
    }

    fn write_tvshow_nfo(&self, output_dir: &Path, content: &str) {
        if let Err(e) = self
            .show_metadata_repository
            .write_tvshow_nfo(output_dir, content)
        {
            warn!(output_dir = ?output_dir, error = %e, "failed to write tvshow.nfo");
        }
    }

    fn write_poster(&self, output_dir: &Path, avatar_filename: &str) {
        if let Err(e) = self
            .show_metadata_repository
            .write_poster(output_dir, avatar_filename)
        {
            warn!(output_dir = ?output_dir, error = %e, "failed to write the show poster");
        }
    }
}
