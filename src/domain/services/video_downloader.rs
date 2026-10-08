use crate::domain::event::DomainEvent;
use crate::domain::services::delete_video_entry;
use crate::domain::services::{MetadataGenerator, MetadataGeneratorApi};
use crate::domain::shared::{LibraryLayout, Quality};
use crate::domain::video::Video;
use crate::domain::video::VideoDownloaded;
use crate::domain::video::VideoRecordId;
use crate::domain::video::thumbnail_filename::expected_thumbnail_filename;
use crate::domain::video::top_level_entry;
use crate::domain::video::video_filename::{VideoFilename, episode_folder_name};
use crate::domain::video::{entry_location, video_entry};
use crate::domain::video_metadata::{EpisodeNumber, NfoFile, VideoMetadata};
use crate::infrastructure::repositories::filesystem_video_file_repository::VideoFileRepository;
use crate::infrastructure::repositories::sqlite_playlist_video_repository::PlaylistVideoRepository;
use crate::infrastructure::repositories::sqlite_video_metadata_repository::VideoMetadataRepository;
use crate::infrastructure::repositories::sqlite_video_repository::VideoRepository;
use crate::infrastructure::repositories::youtube_video_downloader_repository::{
    DownloadAttempt, DownloadedVideo, VideoDownloaderRepository,
};
use crate::infrastructure::shared::domain_events::event_publisher::EventPublisher;
use crate::infrastructure::shared::system_clock::Clock;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Substrings that, found (case-insensitively) in a diagnosed failure reason,
/// mark a video permanently unavailable — see design.md's token table. This
/// is an allowlist of *permanent* reasons, never an inverted "everything
/// except transient" rule, so an unseen `yt-dlp`/YouTube wording is never
/// silently excluded forever.
const PERMANENT_UNAVAILABILITY_TOKENS: [&str; 8] = [
    "members-only",
    "claimed content",
    "copyright",
    "in your country",
    "private video",
    "has been removed",
    "no longer available",
    "account associated with this video has been terminated",
];

/// Whether a diagnosed failure `reason` names a block that can never succeed
/// on a later attempt, so the video should be excluded rather than retried.
fn is_permanently_unavailable_reason(reason: &str) -> bool {
    let reason = reason.to_lowercase();
    PERMANENT_UNAVAILABILITY_TOKENS
        .iter()
        .any(|token| reason.contains(token))
}

/// Joins the diagnosed reason (the richer signal) with the download's own
/// stderr (the fallback) for logging and classification, skipping whichever
/// is absent or empty.
fn combined_reason(diagnosed: Option<&str>, stderr: Option<&str>) -> String {
    [diagnosed, stderr]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Emits the dedicated, greppable SABR event when `yt-dlp` reported YouTube's
/// SABR-only streaming experiment for this download (see the `video-download`
/// capability's "SABR-Only Streaming Is Surfaced" requirement). It is purely
/// observational: it fires on both a failed and a degraded-successful download
/// and never changes the video's status, recorded reason, or retry flow.
fn log_sabr(video: &Video, sabr_notice: Option<&str>) {
    if let Some(reason) = sabr_notice {
        warn!(
            video_id = %video.id,
            reason = %reason,
            "SABR-only streaming experiment reported by yt-dlp; higher-quality formats were skipped"
        );
    }
}

/// The folder a download writes into, relative to the output dir, the base
/// filename it asks `yt-dlp` for, and whether the folder was created for
/// this attempt (and so is removed again if the attempt fails).
struct PreparedFolder {
    folder: String,
    desired_filename: String,
    /// The base name the media file, thumbnail and NFO share.
    basename: String,
    /// The video's entry (see `video_entry`), what cleanup deletes.
    entry: String,
    fresh: bool,
}

impl PreparedFolder {
    /// Where the video's files already live (e.g. a thumbnail fetched ahead
    /// of the download), keeping their base name: a TV-layout entry
    /// `"Season N/<base>"` reuses its season folder and base name, any other
    /// entry its own folder, named after it.
    fn reused(entry: &str) -> Self {
        let (folder, basename) = entry_location(entry);
        Self {
            folder: folder.to_string(),
            desired_filename: basename.to_string(),
            basename: basename.to_string(),
            entry: entry.to_string(),
            fresh: false,
        }
    }
}

/// Downloads one video via `yt-dlp`, transitioning it through in-progress to
/// downloaded/errored. Container-agnostic: the caller (a task handler)
/// already resolved the video's owning container's quality and output
/// directory before invoking this.
#[derive(Clone)]
pub struct VideoDownloader {
    video_repository: Arc<dyn VideoRepository>,
    video_downloader_repository: Arc<dyn VideoDownloaderRepository>,
    video_file_repository: Arc<dyn VideoFileRepository>,
    playlist_video_repository: Arc<dyn PlaylistVideoRepository>,
    metadata_generator: Arc<MetadataGenerator>,
    video_metadata_repository: Arc<dyn VideoMetadataRepository>,
    event_publisher: Arc<dyn EventPublisher>,
    clock: Arc<dyn Clock>,
    layout: LibraryLayout,
}

impl VideoDownloader {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        video_repository: Arc<dyn VideoRepository>,
        video_downloader_repository: Arc<dyn VideoDownloaderRepository>,
        video_file_repository: Arc<dyn VideoFileRepository>,
        playlist_video_repository: Arc<dyn PlaylistVideoRepository>,
        metadata_generator: Arc<MetadataGenerator>,
        video_metadata_repository: Arc<dyn VideoMetadataRepository>,
        event_publisher: Arc<dyn EventPublisher>,
        clock: Arc<dyn Clock>,
        layout: LibraryLayout,
    ) -> Self {
        Self {
            video_repository,
            video_downloader_repository,
            video_file_repository,
            playlist_video_repository,
            metadata_generator,
            video_metadata_repository,
            event_publisher,
            clock,
            layout,
        }
    }
}

pub trait VideoDownloaderApi: Send + Sync {
    /// No-ops (without touching status) if the video no longer exists,
    /// since that means the download no longer needs to happen. A video
    /// deleted while its download ran gets nothing recorded, and the folder
    /// the download wrote is removed. Returns
    /// `Err` on a failed download so the task queue retries/dead-letters it.
    fn download(
        &self,
        video_id: VideoRecordId,
        quality: Quality,
        output_dir: &Path,
        is_last_attempt: bool,
    ) -> anyhow::Result<()>;
}

impl VideoDownloaderApi for VideoDownloader {
    fn download(
        &self,
        video_id: VideoRecordId,
        quality: Quality,
        output_dir: &Path,
        is_last_attempt: bool,
    ) -> anyhow::Result<()> {
        let Some(video) = self.video_repository.find(&video_id)? else {
            debug!(video_id = %video_id, "video no longer exists, skipping download");
            return Ok(());
        };
        if video.is_download_settled() {
            debug!(video_id = %video_id, status = video.status.as_str(), "video already settled, skipping download");
            return Ok(());
        }

        let started = video.start_download(self.clock.now());
        self.video_repository.update(&started)?;
        let (metadata, sort_position) = self.fetch_metadata(&started);
        if self.layout == LibraryLayout::Tv && metadata.is_none() {
            return self.fail_without_metadata(started, is_last_attempt);
        }
        match self.run_download(
            &started,
            quality,
            output_dir,
            metadata.as_ref(),
            sort_position,
        ) {
            Ok(DownloadAttempt::Succeeded(downloaded)) => {
                log_sabr(&started, downloaded.sabr_notice.as_deref());
                self.record_unless_deleted(started, downloaded, quality, output_dir, metadata)
            }
            Ok(DownloadAttempt::Failed {
                stderr,
                sabr_notice,
            }) => {
                log_sabr(&started, sabr_notice.as_deref());
                self.record_failed(started, stderr, is_last_attempt)
            }
            Err(e) => self.record_errored(started, e, is_last_attempt),
        }
    }
}

impl VideoDownloader {
    /// Resolves the video's folder before `yt-dlp` runs, reusing the one a
    /// thumbnail fetched ahead of the download already created, if any, and
    /// writes its NFO into it first, so a media scanner never sees the
    /// video without it. When the download doesn't succeed, that NFO is
    /// removed, and so is a folder freshly created for this
    /// attempt, so a retry's collision check reuses the same folder name
    /// instead of suffixing it.
    fn run_download(
        &self,
        video: &Video,
        quality: Quality,
        output_dir: &Path,
        metadata: Option<&VideoMetadata>,
        sort_position: Option<i64>,
    ) -> anyhow::Result<DownloadAttempt> {
        let prepared = self.prepare_video_folder(video, output_dir, metadata, sort_position)?;
        let folder = prepared.folder.as_str();
        let video_dir = output_dir.join(folder);
        let basename = prepared.basename.as_str();
        let nfo = metadata.map(|metadata| {
            let ahead = metadata.clone().with_thumb(Some(format!("{basename}.jpg")));
            self.nfo_file(&ahead, basename)
        });
        if let Some(nfo) = &nfo {
            self.write_nfo_ahead(video, nfo, &video_dir);
        }
        info!(video_id = %video.id, "downloading video");
        let attempt = self.video_downloader_repository.download(
            &video.youtube_id.to_url(),
            &prepared.desired_filename,
            video.youtube_id.as_str(),
            quality,
            output_dir,
            Some(folder),
        );
        if !matches!(attempt, Ok(DownloadAttempt::Succeeded(_))) {
            if let Some(nfo) = &nfo {
                self.remove_nfo(video, &nfo.filename, &video_dir);
            }
            if prepared.fresh {
                self.remove_fresh_entry(video, output_dir, &prepared.entry);
            }
        }
        attempt
    }

    /// In the TV layout (once the metadata gives the publish time) the
    /// video's episode folder inside its season folder; otherwise its
    /// title-named folder, reusing one a thumbnail fetched ahead of the
    /// download already created.
    fn prepare_video_folder(
        &self,
        video: &Video,
        output_dir: &Path,
        metadata: Option<&VideoMetadata>,
        sort_position: Option<i64>,
    ) -> anyhow::Result<PreparedFolder> {
        match (self.layout, metadata, video.recorded_video_entry()) {
            (LibraryLayout::Tv, Some(_), Some(folder)) => Ok(PreparedFolder::reused(folder)),
            (LibraryLayout::Tv, Some(metadata), None) => self.prepare_episode_folder(
                video,
                output_dir,
                EpisodeNumber::resolve(metadata.published_at, sort_position),
            ),
            _ => self.prepare_movie_folder(video, output_dir),
        }
    }

    fn prepare_movie_folder(
        &self,
        video: &Video,
        output_dir: &Path,
    ) -> anyhow::Result<PreparedFolder> {
        let filename = VideoFilename::from_title(&video.title);
        let existing_folder = video.thumbnail_filename.as_deref().map(top_level_entry);
        let folder = self.video_downloader_repository.prepare_folder(
            filename.as_str(),
            video.youtube_id.as_str(),
            output_dir,
            existing_folder,
        )?;
        Ok(PreparedFolder {
            desired_filename: filename.as_str().to_string(),
            basename: folder.clone(),
            entry: folder.clone(),
            folder,
            fresh: existing_folder.is_none(),
        })
    }

    fn prepare_episode_folder(
        &self,
        video: &Video,
        output_dir: &Path,
        episode: EpisodeNumber,
    ) -> anyhow::Result<PreparedFolder> {
        let name = episode_folder_name(episode, &video.title);
        let season_dir = episode.season_dir();
        let basename = self.video_downloader_repository.prepare_episode(
            &output_dir.join(&season_dir),
            name.as_str(),
            video.youtube_id.as_str(),
        )?;
        Ok(PreparedFolder {
            entry: format!("{season_dir}/{basename}"),
            folder: season_dir,
            desired_filename: basename.clone(),
            basename,
            fresh: true,
        })
    }

    /// In the TV layout, an episode NFO named after the media file's
    /// `basename` when it carries an episode number; otherwise `movie.nfo`.
    fn nfo_file(&self, metadata: &VideoMetadata, basename: &str) -> NfoFile {
        match (self.layout, EpisodeNumber::parse_prefix(basename)) {
            (LibraryLayout::Tv, Some(episode)) => NfoFile::episode(metadata, episode, basename),
            _ => NfoFile::movie(metadata),
        }
    }

    /// Best-effort: without it the video still downloads, and Plex then
    /// imports it before its NFO exists, as before this step.
    fn write_nfo_ahead(&self, video: &Video, nfo: &NfoFile, video_dir: &Path) {
        if let Err(e) = self.video_metadata_repository.write_nfo(nfo, video_dir) {
            warn!(video_id = %video.id, error = %e, "failed to write the NFO ahead of the download");
        }
    }

    /// Best-effort: a stale NFO next to no media is ignored by Plex and
    /// overwritten by the next attempt.
    fn remove_nfo(&self, video: &Video, nfo_filename: &str, video_dir: &Path) {
        if let Err(e) = self
            .video_metadata_repository
            .remove_nfo(nfo_filename, video_dir)
        {
            warn!(video_id = %video.id, error = %e, "failed to remove movie.nfo of a download that did not succeed");
        }
    }

    /// Best-effort: leftovers only cost the retry a suffixed name.
    fn remove_fresh_entry(&self, video: &Video, output_dir: &Path, entry: &str) {
        if let Err(e) = delete_video_entry(self.video_file_repository.as_ref(), output_dir, entry) {
            warn!(video_id = %video.id, error = %e, "failed to remove the folder of a download that did not succeed");
        }
    }

    /// The video (or its whole playlist/channel) may have been deleted while
    /// it downloaded: then the folder the download wrote is removed instead
    /// of recorded, so no stray folder outlives it.
    fn record_unless_deleted(
        &self,
        started: Video,
        downloaded: DownloadedVideo,
        quality: Quality,
        output_dir: &Path,
        metadata: Option<VideoMetadata>,
    ) -> anyhow::Result<()> {
        if self.video_repository.find(&started.id)?.is_none() {
            self.discard_download(&started, &downloaded, output_dir);
            return Ok(());
        }
        self.record_downloaded(started, downloaded, quality, output_dir, metadata)
    }

    /// Best-effort: a folder that can't be removed is logged, and the next
    /// reconcile's orphan sweep (if the container still exists) removes it.
    fn discard_download(&self, video: &Video, downloaded: &DownloadedVideo, output_dir: &Path) {
        let recorded = format!("{}/{}", downloaded.folder, downloaded.filename);
        let entry = video_entry(&recorded);
        info!(video_id = %video.id, entry = %entry, "video deleted during its download, removing the downloaded files");
        if let Err(e) = delete_video_entry(self.video_file_repository.as_ref(), output_dir, entry) {
            warn!(video_id = %video.id, error = %e, "failed to remove the folder of a download whose video was deleted");
        }
    }

    fn record_downloaded(
        &self,
        started: Video,
        downloaded: DownloadedVideo,
        quality: Quality,
        output_dir: &Path,
        metadata: Option<VideoMetadata>,
    ) -> anyhow::Result<()> {
        let video_dir = output_dir.join(&downloaded.folder);
        let thumbnail_filename = self.find_downloaded_thumbnail(&video_dir, &downloaded)?;
        let filename = format!("{}/{}", downloaded.folder, downloaded.filename);
        let downloaded_video = started.mark_downloaded(
            quality,
            filename,
            thumbnail_filename.clone(),
            downloaded.duration_seconds,
            self.clock.now(),
        );
        self.video_repository.update(&downloaded_video)?;
        info!(video_id = %downloaded_video.id, "video downloaded");
        let thumb_basename = thumbnail_filename
            .as_deref()
            .and_then(|f| Path::new(f).file_name())
            .and_then(|f| f.to_str());
        self.save_metadata(&downloaded_video, &video_dir, thumb_basename, metadata);
        self.publish_downloaded(&downloaded_video, output_dir, &downloaded.folder);
        Ok(())
    }

    /// Best-effort: the download is already recorded, and losing the event
    /// only skips reacting to it (e.g. asking Plex to scan the folder).
    fn publish_downloaded(&self, video: &Video, output_dir: &Path, folder: &str) {
        let event = DomainEvent::VideoDownloaded(VideoDownloaded {
            video_id: video.id.as_str().to_string(),
            output_dir: output_dir.to_string_lossy().to_string(),
            folder: folder.to_string(),
        });
        if let Err(e) = self.event_publisher.publish(&event) {
            warn!(video_id = %video.id, error = %e, "failed to publish that the video was downloaded");
        }
    }

    /// The `<folder>/<thumbnail>` path of the thumbnail `yt-dlp` wrote next
    /// to the downloaded video, if it wrote one.
    fn find_downloaded_thumbnail(
        &self,
        video_dir: &Path,
        downloaded: &DownloadedVideo,
    ) -> anyhow::Result<Option<String>> {
        let expected_thumbnail = expected_thumbnail_filename(&downloaded.filename);
        Ok(self
            .video_file_repository
            .list(video_dir)?
            .iter()
            .any(|f| f == &expected_thumbnail)
            .then(|| format!("{}/{}", downloaded.folder, expected_thumbnail)))
    }

    /// A clean `yt-dlp` failure: diagnoses the precise reason (see design.md),
    /// logs it, and either excludes a permanently-unavailable video (settling
    /// the task with `Ok(())`, so it is neither retried nor dead-lettered) or
    /// keeps the current errored/retry behavior and returns `Err`.
    fn record_failed(
        &self,
        started: Video,
        stderr: Option<String>,
        is_last_attempt: bool,
    ) -> anyhow::Result<()> {
        let diagnosed = self.diagnose_failure(&started.youtube_id.to_url());
        let reason = combined_reason(diagnosed.as_deref(), stderr.as_deref());
        if is_permanently_unavailable_reason(&reason) {
            self.exclude(started, &reason)
        } else {
            self.fail_retryably(started, reason, is_last_attempt)
        }
    }

    /// Diagnoses a failed download's precise reason, swallowing a probe error
    /// (logged) as undetermined so the probe failing never fails the handling.
    fn diagnose_failure(&self, video_url: &str) -> Option<String> {
        match self.video_downloader_repository.diagnose(video_url) {
            Ok(reason) => reason,
            Err(e) => {
                warn!(error = %e, "diagnostic probe failed; treating the failure reason as undetermined");
                None
            }
        }
    }

    /// Marks a permanently-unavailable video excluded and settles the task
    /// without error, so no retry is scheduled and it is not dead-lettered.
    fn exclude(&self, started: Video, reason: &str) -> anyhow::Result<()> {
        let video_id = started.id.clone();
        let excluded = started.mark_excluded(self.clock.now());
        self.video_repository.update(&excluded)?;
        info!(video_id = %video_id, reason = %reason, "excluding permanently-unavailable video");
        Ok(())
    }

    /// Marks the video errored (retrying, or permanently on the last attempt)
    /// and returns the precise reason so the task queue retries/dead-letters
    /// it.
    fn fail_retryably(
        &self,
        started: Video,
        reason: String,
        is_last_attempt: bool,
    ) -> anyhow::Result<()> {
        let video_id = started.id.clone();
        self.mark_errored(started, is_last_attempt)?;
        let error_message = if reason.is_empty() {
            format!("yt-dlp failed to download video {video_id}")
        } else {
            reason
        };
        warn!(video_id = %video_id, error = %error_message, "yt-dlp reported a failed download");
        Err(anyhow::anyhow!(error_message))
    }

    /// In the TV layout the publish time names the episode, so without the
    /// video's YouTube metadata `yt-dlp` isn't run: the video is marked
    /// errored and the task retried, like a failed download.
    fn fail_without_metadata(&self, started: Video, is_last_attempt: bool) -> anyhow::Result<()> {
        let video_id = started.id.clone();
        self.mark_errored(started, is_last_attempt)?;
        warn!(video_id = %video_id, "YouTube metadata unavailable, not downloading the episode");
        Err(anyhow::anyhow!(
            "YouTube metadata unavailable, needed to name the episode"
        ))
    }

    /// A systemic download error: marks the video errored and propagates
    /// the error so the task queue retries/dead-letters it.
    fn record_errored(
        &self,
        started: Video,
        error: anyhow::Error,
        is_last_attempt: bool,
    ) -> anyhow::Result<()> {
        let video_id = started.id.clone();
        self.mark_errored(started, is_last_attempt)?;
        error!(video_id = %video_id, error = %error, "video download errored");
        Err(error)
    }

    fn mark_errored(&self, started: Video, is_last_attempt: bool) -> anyhow::Result<()> {
        let updated = if is_last_attempt {
            started.mark_errored(self.clock.now())
        } else {
            started.mark_errored_retrying(self.clock.now())
        };
        self.video_repository.update(&updated)
    }

    /// Builds `video`'s metadata with its playlist position (no `thumb` yet)
    /// — see design.md's "Failure handling: skip the save entirely, never
    /// fail the download" decision. Any failure (the playlist-position lookup
    /// or the YouTube fetch) is logged and swallowed rather than propagated:
    /// metadata generation never fails or retries the download itself, and a
    /// skipped/failed attempt self-heals on the next reconcile pass (see
    /// `InternalVideoReconciler`).
    fn fetch_metadata(&self, video: &Video) -> (Option<VideoMetadata>, Option<i64>) {
        let playlist_position = match self.playlist_video_repository.find_by_video(&video.id) {
            Ok(playlist_video) => playlist_video.map(|pv| pv.position),
            Err(e) => {
                warn!(video_id = %video.id, error = %e, "failed to look up playlist position, falling back to publish-date sorttitle");
                None
            }
        };
        let metadata = self
            .metadata_generator
            .generate(video, playlist_position, None);
        (metadata, playlist_position)
    }

    /// Saves the downloaded `video`'s NFO and records its metadata,
    /// referencing the thumbnail actually downloaded. When the fetch before
    /// the download failed, it is attempted once more here. A failure is
    /// logged and swallowed, never failing the download.
    fn save_metadata(
        &self,
        video: &Video,
        video_dir: &Path,
        thumbnail_filename: Option<&str>,
        metadata: Option<VideoMetadata>,
    ) {
        let Some(metadata) = metadata.or_else(|| self.fetch_metadata(video).0) else {
            return;
        };
        let video_metadata = metadata.with_thumb(thumbnail_filename.map(str::to_string));
        let media_stem = video
            .filename
            .as_deref()
            .and_then(|f| Path::new(f).file_stem())
            .and_then(|f| f.to_str())
            .unwrap_or_default();
        let nfo = self.nfo_file(&video_metadata, media_stem);
        if let Err(e) =
            self.video_metadata_repository
                .save(&video.id, &video_metadata, &nfo, video_dir)
        {
            warn!(video_id = %video.id, error = %e, "failed to save video metadata");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_permanently_unavailable_reason;

    #[test]
    fn it_should_classify_permanently_unavailable_reasons() {
        let reasons = [
            "Join this channel to get access to members-only content like this video, and other exclusive perks.",
            "It was blocked due to the claimed content by Mediatoon.",
            "Video unavailable. This video contains content that has been blocked on copyright grounds.",
            "The uploader has not made this video available in your country.",
            "Private video. Sign in if you've been granted access to this video.",
            "This video has been removed by the uploader.",
            "This video is no longer available.",
            "This video is not available because the YouTube account associated with this video has been terminated.",
        ];

        assert_eq!(
            reasons.map(is_permanently_unavailable_reason),
            [true, true, true, true, true, true, true, true]
        );
    }

    #[test]
    fn it_should_classify_permanently_unavailable_reasons_case_insensitively() {
        let reasons = [
            "JOIN THIS CHANNEL FOR MEMBERS-ONLY CONTENT",
            "It was blocked due to the CLAIMED CONTENT by X.",
        ];

        assert_eq!(reasons.map(is_permanently_unavailable_reason), [true, true]);
    }

    #[test]
    fn it_should_not_classify_generic_or_transient_reasons() {
        let reasons = [
            "Video unavailable",
            "HTTP Error 403: Forbidden",
            "Unable to download webpage: The read operation timed out",
            "HTTP Error 429: Too Many Requests",
            "Sign in to confirm you're not a bot",
            "Unable to download video data: fragment 3 not found",
            "Connection reset by peer",
        ];

        assert_eq!(reasons.map(is_permanently_unavailable_reason), [false; 7]);
    }
}
