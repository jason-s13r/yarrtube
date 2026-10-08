use crate::domain::video_metadata::TVSHOW_NFO_FILENAME;
use std::path::{Path, PathBuf};

/// Writes a TV-layout show's own files at the root of its tracked channel's
/// or playlist's output folder: `tvshow.nfo` and the poster copied from the
/// channel's stored avatar. Injected into `ShowMetadataWriter`.
pub trait ShowMetadataRepository: Send + Sync {
    /// Writes `content` as `output_dir/tvshow.nfo`, overwriting any existing
    /// one.
    fn write_tvshow_nfo(&self, output_dir: &Path, content: &str) -> anyhow::Result<()>;

    /// Copies `avatars_dir/<avatar_filename>` to `output_dir/poster.<ext>`,
    /// keeping the avatar's extension; a no-op if that poster exists.
    fn write_poster(&self, output_dir: &Path, avatar_filename: &str) -> anyhow::Result<()>;
}

pub struct FilesystemShowMetadataRepository {
    avatars_dir: PathBuf,
}

impl FilesystemShowMetadataRepository {
    pub fn new(avatars_dir: PathBuf) -> Self {
        Self { avatars_dir }
    }
}

impl ShowMetadataRepository for FilesystemShowMetadataRepository {
    fn write_tvshow_nfo(&self, output_dir: &Path, content: &str) -> anyhow::Result<()> {
        let path = output_dir.join(TVSHOW_NFO_FILENAME);
        std::fs::create_dir_all(output_dir)
            .and_then(|()| std::fs::write(&path, content))
            .map_err(|e| anyhow::anyhow!("failed to write {path:?}: {e}"))
    }

    fn write_poster(&self, output_dir: &Path, avatar_filename: &str) -> anyhow::Result<()> {
        let avatar = self.avatars_dir.join(avatar_filename);
        let extension = avatar.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
        let poster = output_dir.join(format!("poster.{extension}"));
        if poster.exists() {
            return Ok(());
        }
        std::fs::create_dir_all(output_dir)
            .and_then(|()| std::fs::copy(&avatar, &poster))
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("failed to copy {avatar:?} to {poster:?}: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_write_tvshow_nfo_at_the_output_dir_root() {
        let root = tempfile::tempdir().unwrap();
        let output_dir = root.path().join("channels/somechannel");
        let repository = FilesystemShowMetadataRepository::new(root.path().join("avatars"));

        let first = repository.write_tvshow_nfo(&output_dir, "<tvshow>old</tvshow>");
        let second = repository.write_tvshow_nfo(&output_dir, "<tvshow>new</tvshow>");

        assert_eq!(
            (
                first.is_ok(),
                second.is_ok(),
                std::fs::read_to_string(output_dir.join("tvshow.nfo")).ok()
            ),
            (true, true, Some("<tvshow>new</tvshow>".to_string()))
        );
    }

    #[test]
    fn it_should_copy_the_avatar_as_poster_keeping_its_extension() {
        let root = tempfile::tempdir().unwrap();
        let avatars_dir = root.path().join("avatars");
        let output_dir = root.path().join("channels/somechannel");
        std::fs::create_dir_all(&avatars_dir).unwrap();
        std::fs::write(avatars_dir.join("@somechannel.png"), "png bytes").unwrap();
        let repository = FilesystemShowMetadataRepository::new(avatars_dir);

        let result = repository.write_poster(&output_dir, "@somechannel.png");

        assert_eq!(
            (
                result.is_ok(),
                std::fs::read_to_string(output_dir.join("poster.png")).ok()
            ),
            (true, Some("png bytes".to_string()))
        );
    }

    #[test]
    fn it_should_leave_an_existing_poster_unchanged() {
        let root = tempfile::tempdir().unwrap();
        let avatars_dir = root.path().join("avatars");
        let output_dir = root.path().join("channels/somechannel");
        std::fs::create_dir_all(&avatars_dir).unwrap();
        std::fs::create_dir_all(&output_dir).unwrap();
        std::fs::write(avatars_dir.join("@somechannel.jpg"), "new avatar").unwrap();
        std::fs::write(output_dir.join("poster.jpg"), "user's poster").unwrap();
        let repository = FilesystemShowMetadataRepository::new(avatars_dir);

        let result = repository.write_poster(&output_dir, "@somechannel.jpg");

        assert_eq!(
            (
                result.is_ok(),
                std::fs::read_to_string(output_dir.join("poster.jpg")).ok()
            ),
            (true, Some("user's poster".to_string()))
        );
    }

    #[test]
    fn it_should_fail_when_the_avatar_file_is_missing() {
        let root = tempfile::tempdir().unwrap();
        let output_dir = root.path().join("channels/somechannel");
        let repository = FilesystemShowMetadataRepository::new(root.path().join("avatars"));

        let result = repository.write_poster(&output_dir, "@somechannel.jpg");

        assert_eq!(
            (result.is_err(), output_dir.join("poster.jpg").exists()),
            (true, false)
        );
    }
}

/// State-based stand-in for the show folders: the `tvshow.nfo` contents and
/// posters written, each with its output dir. `failing()` makes every write
/// return `Err`, recording nothing.
#[cfg(test)]
#[derive(Default)]
pub struct FakeShowMetadataRepository {
    pub(crate) tvshow_nfos: std::sync::Mutex<Vec<(PathBuf, String)>>,
    pub(crate) posters: std::sync::Mutex<Vec<(PathBuf, String)>>,
    failing: bool,
}

#[cfg(test)]
impl FakeShowMetadataRepository {
    pub fn failing() -> Self {
        Self {
            failing: true,
            ..Self::default()
        }
    }

    fn fail_if_failing(&self) -> anyhow::Result<()> {
        if self.failing {
            return Err(anyhow::anyhow!("fake show metadata write error"));
        }
        Ok(())
    }
}

#[cfg(test)]
impl ShowMetadataRepository for FakeShowMetadataRepository {
    fn write_tvshow_nfo(&self, output_dir: &Path, content: &str) -> anyhow::Result<()> {
        self.fail_if_failing()?;
        self.tvshow_nfos
            .lock()
            .unwrap()
            .push((output_dir.to_path_buf(), content.to_string()));
        Ok(())
    }

    fn write_poster(&self, output_dir: &Path, avatar_filename: &str) -> anyhow::Result<()> {
        self.fail_if_failing()?;
        self.posters
            .lock()
            .unwrap()
            .push((output_dir.to_path_buf(), avatar_filename.to_string()));
        Ok(())
    }
}
