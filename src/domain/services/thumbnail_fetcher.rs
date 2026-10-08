use crate::domain::services::{MetadataGenerator, MetadataGeneratorApi};
use crate::domain::shared::LibraryLayout;
use crate::domain::task::Task;
use crate::domain::video::Video;
use crate::domain::video::VideoRecordId;
use crate::domain::video::VideoStatus;
use crate::domain::video::entry_location;
use crate::domain::video::top_level_entry;
use crate::domain::video::video_filename::{VideoFilename, episode_folder_name};
use crate::domain::video_metadata::EpisodeNumber;
use crate::infrastructure::repositories::sqlite_playlist_video_repository::PlaylistVideoRepository;
use crate::infrastructure::repositories::sqlite_task_repository::TaskRepository;
use crate::infrastructure::repositories::sqlite_video_repository::VideoRepository;
use crate::infrastructure::repositories::youtube_video_downloader_repository::{
    FetchedThumbnail, ThumbnailFetch, VideoDownloaderRepository,
};
use crate::infrastructure::shared::system_clock::Clock;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tracing::warn;

/// Best-effort thumbnail fetch, independent of and ahead of a video's full
/// download — see the `video-thumbnails` capability. The fetch itself runs
/// from the `fetch_thumbnail` task and never returns an error to its caller;
/// each reconciler's missing-thumbnail recovery pass only schedules it.
#[derive(Clone)]
pub struct ThumbnailFetcher {
    video_repository: Arc<dyn VideoRepository>,
    video_downloader_repository: Arc<dyn VideoDownloaderRepository>,
    task_repository: Arc<dyn TaskRepository>,
    clock: Arc<dyn Clock>,
    metadata_generator: Arc<MetadataGenerator>,
    playlist_video_repository: Arc<dyn PlaylistVideoRepository>,
    layout: LibraryLayout,
}

impl ThumbnailFetcher {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        video_repository: Arc<dyn VideoRepository>,
        video_downloader_repository: Arc<dyn VideoDownloaderRepository>,
        task_repository: Arc<dyn TaskRepository>,
        clock: Arc<dyn Clock>,
        metadata_generator: Arc<MetadataGenerator>,
        playlist_video_repository: Arc<dyn PlaylistVideoRepository>,
        layout: LibraryLayout,
    ) -> Self {
        Self {
            video_repository,
            video_downloader_repository,
            task_repository,
            clock,
            metadata_generator,
            playlist_video_repository,
            layout,
        }
    }
}

pub trait ThumbnailFetcherApi: Send + Sync {
    /// No-ops if `video` already has a recorded thumbnail or is no longer
    /// thumbnail-fetchable (`Video::is_thumbnail_fetchable`). Otherwise fetches
    /// one into `output_dir` and persists it via `Video::with_thumbnail` on
    /// success; any failure (a clean "no thumbnail available" outcome, or a
    /// systemic error) is logged and swallowed, leaving `video` untouched.
    /// A `Downloaded` video already has its own folder recorded via
    /// `filename` (e.g. a missing-thumbnail recovery pass running against a
    /// video whose full download already ran) — that folder is reused
    /// verbatim instead of resolving a fresh, collision-suffixed one.
    fn fetch(&self, video: &Video, output_dir: &Path);

    /// Missing-thumbnail recovery pass over `videos`, run by
    /// `InternalVideoReconciler`: schedules a `FetchThumbnail` for
    /// each video with no thumbnail, except one in `skip_ids` (just reset for
    /// redownload this same pass, so its download writes its own thumbnail)
    /// or one whose download is `InProgress` (it records its own thumbnail),
    /// or one that isn't thumbnail-fetchable (`Video::is_thumbnail_fetchable`).
    /// A video that already has a fetch pending or running gets no second
    /// one: `TaskRepository::schedule` dedupes it.
    fn schedule_missing(
        &self,
        videos: &[Video],
        skip_ids: &HashSet<&VideoRecordId>,
        output_dir: &Path,
    ) -> anyhow::Result<()>;
}

impl ThumbnailFetcherApi for ThumbnailFetcher {
    fn fetch(&self, video: &Video, output_dir: &Path) {
        if video.thumbnail_filename.is_some() || !video.is_thumbnail_fetchable() {
            return;
        }

        match self.fetch_thumbnail(video, output_dir) {
            Ok(ThumbnailFetch::Fetched(fetched)) => self.record_thumbnail(video, fetched),
            Ok(ThumbnailFetch::Unavailable { reason }) => warn!(
                video_id = %video.id,
                reason = reason.as_deref(),
                "no thumbnail available for video"
            ),
            Err(e) => {
                warn!(video_id = %video.id, error = %e, "failed to fetch video thumbnail");
            }
        }
    }

    fn schedule_missing(
        &self,
        videos: &[Video],
        skip_ids: &HashSet<&VideoRecordId>,
        output_dir: &Path,
    ) -> anyhow::Result<()> {
        videos
            .iter()
            .filter(|v| {
                v.thumbnail_filename.is_none()
                    && !skip_ids.contains(&v.id)
                    && v.status != VideoStatus::InProgress
                    && v.is_thumbnail_fetchable()
            })
            .try_for_each(|video| self.schedule_fetch(video, output_dir))
    }
}

impl ThumbnailFetcher {
    fn fetch_thumbnail(&self, video: &Video, output_dir: &Path) -> anyhow::Result<ThumbnailFetch> {
        match self.layout {
            LibraryLayout::Tv => self.fetch_episode_thumbnail(video, output_dir),
            LibraryLayout::Movie => self.fetch_movie_thumbnail(video, output_dir),
        }
    }

    /// Reuses `video`'s already-recorded folder, if any.
    fn fetch_movie_thumbnail(
        &self,
        video: &Video,
        output_dir: &Path,
    ) -> anyhow::Result<ThumbnailFetch> {
        let existing_folder = video.filename.as_deref().map(top_level_entry);
        let filename = VideoFilename::from_title(&video.title);
        self.video_downloader_repository.fetch_thumbnail(
            &video.youtube_id.to_url(),
            filename.as_str(),
            video.youtube_id.as_str(),
            output_dir,
            existing_folder,
        )
    }

    /// Saves the thumbnail under the episode name in the video's episode
    /// folder: the one its files already live in, else a fresh one in its
    /// season folder, numbered from its YouTube metadata, which the
    /// download later reuses.
    fn fetch_episode_thumbnail(
        &self,
        video: &Video,
        output_dir: &Path,
    ) -> anyhow::Result<ThumbnailFetch> {
        let Some(entry) = self.episode_entry(video, output_dir)? else {
            return Ok(ThumbnailFetch::Unavailable {
                reason: Some(
                    "YouTube metadata unavailable, needed to name the episode".to_string(),
                ),
            });
        };
        let (folder, basename) = entry_location(&entry);
        self.video_downloader_repository.fetch_thumbnail(
            &video.youtube_id.to_url(),
            basename,
            video.youtube_id.as_str(),
            output_dir,
            Some(folder),
        )
    }

    /// The entry `video`'s files already live in, else a fresh one claimed
    /// in its season folder; `None` (no folder created) when it can't be
    /// numbered.
    fn episode_entry(&self, video: &Video, output_dir: &Path) -> anyhow::Result<Option<String>> {
        if let Some(folder) = video.recorded_video_entry() {
            return Ok(Some(folder.to_string()));
        }
        self.episode_number(video)
            .map(|episode| self.prepare_episode_folder(video, output_dir, episode))
            .transpose()
    }

    /// From the video's YouTube metadata (its publish time) and playlist
    /// position; `None` when the metadata can't be fetched.
    fn episode_number(&self, video: &Video) -> Option<EpisodeNumber> {
        let sort_position = self.playlist_position(video);
        self.metadata_generator
            .generate(video, sort_position, None)
            .map(|metadata| EpisodeNumber::resolve(metadata.published_at, sort_position))
    }

    fn playlist_position(&self, video: &Video) -> Option<i64> {
        match self.playlist_video_repository.find_by_video(&video.id) {
            Ok(playlist_video) => playlist_video.map(|pv| pv.position),
            Err(e) => {
                warn!(video_id = %video.id, error = %e, "failed to look up playlist position, numbering by publish time");
                None
            }
        }
    }

    fn prepare_episode_folder(
        &self,
        video: &Video,
        output_dir: &Path,
        episode: EpisodeNumber,
    ) -> anyhow::Result<String> {
        let name = episode_folder_name(episode, &video.title);
        let season_dir = episode.season_dir();
        let basename = self.video_downloader_repository.prepare_episode(
            &output_dir.join(&season_dir),
            name.as_str(),
            video.youtube_id.as_str(),
        )?;
        Ok(format!("{season_dir}/{basename}"))
    }

    fn schedule_fetch(&self, video: &Video, output_dir: &Path) -> anyhow::Result<()> {
        self.task_repository.schedule(
            &Task::FetchThumbnail {
                video_id: video.id.as_str().to_string(),
                output_dir: output_dir.to_string_lossy().to_string(),
            },
            self.clock.now(),
        )
    }

    /// Writes only the thumbnail: a download of the same video may have
    /// changed its status since `video` was read.
    fn record_thumbnail(&self, video: &Video, fetched: FetchedThumbnail) {
        let thumbnail_filename = format!("{}/{}", fetched.folder, fetched.filename);
        if let Err(e) =
            self.video_repository
                .update_thumbnail(&video.id, &thumbnail_filename, self.clock.now())
        {
            warn!(video_id = %video.id, error = %e, "failed to persist fetched thumbnail");
        }
    }
}
