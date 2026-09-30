use crate::infrastructure::repositories::plex_collection_repository::PlexCollectionRepository;
use std::sync::Arc;
use tracing::info;

/// Deletes the Plex collections left behind by a deleted playlist/channel,
/// scoped by kind: a deleted playlist's collection is removed only from
/// configured playlist sections and a deleted channel's only from channel
/// sections, so a deleted channel never deletes a same-named collection
/// sitting in a playlist library (or vice versa).
#[derive(Clone)]
pub struct PlexCollectionDeleter {
    playlist_section_ids: Vec<String>,
    channel_section_ids: Vec<String>,
    plex_collection_repository: Arc<dyn PlexCollectionRepository>,
}

impl PlexCollectionDeleter {
    pub fn new(
        playlist_section_ids: Vec<String>,
        channel_section_ids: Vec<String>,
        plex_collection_repository: Arc<dyn PlexCollectionRepository>,
    ) -> Self {
        Self {
            playlist_section_ids,
            channel_section_ids,
            plex_collection_repository,
        }
    }
}

pub trait PlexCollectionDeleterApi: Send + Sync {
    /// Deletes the collection with this title from every configured playlist
    /// section where one exists; missing everywhere is a no-op.
    fn delete_playlist_collection(&self, name: &str) -> anyhow::Result<()>;
    /// Deletes the collection with this title from every configured channel
    /// section where one exists; missing everywhere is a no-op.
    fn delete_channel_collection(&self, name: &str) -> anyhow::Result<()>;
}

impl PlexCollectionDeleterApi for PlexCollectionDeleter {
    fn delete_playlist_collection(&self, name: &str) -> anyhow::Result<()> {
        self.delete_in_sections(&self.playlist_section_ids, name)
    }

    fn delete_channel_collection(&self, name: &str) -> anyhow::Result<()> {
        self.delete_in_sections(&self.channel_section_ids, name)
    }
}

impl PlexCollectionDeleter {
    fn delete_in_sections(&self, section_ids: &[String], name: &str) -> anyhow::Result<()> {
        for section_id in section_ids {
            let collection = self
                .plex_collection_repository
                .list_collections(section_id)?
                .into_iter()
                .find(|collection| collection.title == name);

            if let Some(collection) = collection {
                self.plex_collection_repository
                    .delete_collection(&collection.rating_key)?;
                info!(
                    section = section_id,
                    collection = name,
                    "deleted Plex collection"
                );
            }
        }
        Ok(())
    }
}
