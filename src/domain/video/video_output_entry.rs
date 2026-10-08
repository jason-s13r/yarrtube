use crate::domain::video_metadata::EpisodeNumber;
use crate::domain::video_metadata::episode_number::EPISODE_PREFIX_SEPARATOR;
use std::path::{Path, PathBuf};

/// Returns the first path component of a stored `filename`/`thumbnail_filename`:
/// the video's own output folder name for a new-style `"folder/file.ext"` path
/// (one video per folder, alongside its thumbnail and `movie.nfo`), or the bare
/// filename unchanged for a legacy flat path with no `/` (downloaded before
/// per-video folders were introduced). Either way, this is "the entry that
/// owns this stored path, at the container's output directory level" — what
/// deletion and reconciliation need to delete or protect as a unit.
pub fn top_level_entry(relative_path: &str) -> &str {
    relative_path
        .split_once('/')
        .map_or(relative_path, |(first, _)| first)
}

/// The per-video unit a stored path belongs to, at the level deletion and
/// reconciliation act on: `"Season 2026/F.mp4"` → `"Season 2026/F"` (a
/// TV-layout episode's base name, owning every `F.*` beside it, see
/// `entry_owns`), `"Season 2026/F/F.mp4"` → `"Season 2026/F"` (a
/// first-build episode folder), `"F/F.mp4"` → `"F"`, `"F.mp4"` → `"F.mp4"`.
pub fn video_entry(relative_path: &str) -> &str {
    match relative_path.split_once('/') {
        Some((season, rest)) if is_season_dir(season) => {
            let base = match rest.split_once('/') {
                Some((folder, _)) => folder,
                None => Path::new(rest)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or(rest),
            };
            &relative_path[..season.len() + 1 + base.len()]
        }
        _ => top_level_entry(relative_path),
    }
}

/// The media file's base name inside a per-video `folder`: its last path
/// component, so a TV-layout `"Season N/<episode>"` folder yields
/// `"<episode>"` and a top-level `"F"` yields `"F"`.
pub fn media_basename(folder: &str) -> &str {
    folder.rsplit('/').next().unwrap_or(folder)
}

/// Where a video `entry`'s files live and the base name they share: a
/// TV-layout `"Season N/<base>"` entry → (`"Season N"`, `"<base>"`); any
/// other entry is its own folder, named after it (`"F"` → (`"F"`, `"F"`)).
pub fn entry_location(entry: &str) -> (&str, &str) {
    match entry.split_once('/') {
        Some((season, base)) if is_season_dir(season) => (season, base),
        _ => (entry, media_basename(entry)),
    }
}

/// Whether the listed `file` (relative to the output dir) belongs to the
/// video `entry`: the entry itself, or for a season-folder entry
/// (`"Season N/<base>"`) any file named `<base>.<anything>` beside it.
pub fn entry_owns(entry: &str, file: &str) -> bool {
    let in_season_dir = entry
        .split_once('/')
        .is_some_and(|(first, _)| is_season_dir(first));
    file == entry || (in_season_dir && is_named_after(file, entry))
}

/// Whether a file `name` is `base` itself or `base` plus extensions
/// (`"T.mp4"`, `"T.f137.mp4"`), the way a TV-layout episode's files share
/// its base name.
pub fn is_named_after(name: &str, base: &str) -> bool {
    name.strip_prefix(base)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
}

/// Whether `name` is a TV-layout season folder, `"Season <digits>"`.
pub fn is_season_dir(name: &str) -> bool {
    name.strip_prefix("Season ")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// The title part of a TV-layout episode name: `"S2026E01021530 - T"` →
/// `"T"`; `None` for a name without an episode prefix.
pub fn strip_episode_prefix(name: &str) -> Option<&str> {
    EpisodeNumber::parse_prefix(name)?;
    name.split_once(EPISODE_PREFIX_SEPARATOR)
        .map(|(_, title)| title)
}

/// Resolves the directory a video's NFO belongs in for a video whose stored
/// `filename` is relative to `output_dir`: the file's parent dir, i.e. its
/// own per-video folder for a `"folder/file.ext"` or
/// `"Season N/folder/file.ext"` path, or `output_dir` itself for a legacy
/// flat path (there is no per-video folder to place it in).
pub fn video_dir_for_filename(output_dir: &Path, filename: &str) -> PathBuf {
    match filename.rsplit_once('/') {
        Some((folder, _)) => output_dir.join(folder),
        None => output_dir.to_path_buf(),
    }
}

/// Resolves a playlist's/channel's output directory: `videos_path` joined
/// with its recorded `path`. Shared by every service that reads or writes
/// to a container's directory on disk.
pub fn resolve_output_dir(videos_path: &str, path: &str) -> PathBuf {
    Path::new(videos_path).join(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_resolve_the_video_entry_of_a_flat_episode_file() {
        let entry = video_entry("Season 2026/S2026E01021530 - v1.2 Out.mp4");

        assert_eq!(
            (
                entry,
                video_entry("Season 2026/S2026E01021530 - v1.2 Out.jpg"),
                entry_owns(entry, "Season 2026/S2026E01021530 - v1.2 Out.nfo"),
                entry_owns(entry, "Season 2026/S2026E01021530 - v1.2 Out.logo.txt"),
                entry_owns(entry, "Season 2026/S2026E01021530 - v1.2 Out Two.mp4"),
                entry_owns(entry, "Season 2025/S2026E01021530 - v1.2 Out.mp4"),
                entry_owns("My Video", "My Video"),
                entry_owns("My Video", "My Video.nfo"),
            ),
            (
                "Season 2026/S2026E01021530 - v1.2 Out",
                "Season 2026/S2026E01021530 - v1.2 Out",
                true,
                true,
                false,
                false,
                true,
                false,
            )
        );
    }

    #[test]
    fn it_should_resolve_the_video_entry_of_a_recorded_path() {
        assert_eq!(
            (
                video_entry("Season 2026/S2026E01021530 - T/S2026E01021530 - T.mp4"),
                video_entry("My Video/My Video.mp4"),
                video_entry("My Video.mp4"),
                is_season_dir("Season 2026"),
                is_season_dir("Season 01"),
                is_season_dir("Season"),
                is_season_dir("Season 1a"),
                is_season_dir("My Season 1"),
                video_dir_for_filename(
                    Path::new("/videos/c"),
                    "Season 01/S01E03 - T/S01E03 - T.mp4"
                ),
            ),
            (
                "Season 2026/S2026E01021530 - T",
                "My Video",
                "My Video.mp4",
                true,
                true,
                false,
                false,
                false,
                PathBuf::from("/videos/c/Season 01/S01E03 - T"),
            )
        );
    }

    #[test]
    fn it_should_return_the_folder_name_for_a_nested_path() {
        assert_eq!(top_level_entry("My Video/My Video.mp4"), "My Video");
    }

    #[test]
    fn it_should_return_the_filename_unchanged_for_a_legacy_flat_path() {
        assert_eq!(top_level_entry("My Video.mp4"), "My Video.mp4");
    }

    #[test]
    fn it_should_only_split_on_the_first_separator() {
        assert_eq!(
            top_level_entry("My Video [vid1]/nested/file.mp4"),
            "My Video [vid1]"
        );
    }

    #[test]
    fn it_should_resolve_the_video_dir_as_the_owning_folder_for_a_new_style_path() {
        assert_eq!(
            video_dir_for_filename(Path::new("/videos/my-playlist"), "My Video/My Video.mp4"),
            Path::new("/videos/my-playlist/My Video")
        );
    }

    #[test]
    fn it_should_resolve_the_video_dir_as_the_output_dir_for_a_legacy_flat_path() {
        assert_eq!(
            video_dir_for_filename(Path::new("/videos/my-playlist"), "My Video.mp4"),
            Path::new("/videos/my-playlist")
        );
    }
}
