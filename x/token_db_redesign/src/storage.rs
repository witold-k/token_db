use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{Result, TokenDb, TokenEntry};

const MAGIC: &[u8; 4] = b"TDB1";

#[derive(Serialize)]
struct StoredDb<'a> {
    entries: Vec<StoredEntryRef<'a>>,
}

#[derive(Serialize)]
struct StoredEntryRef<'a> {
    text: &'a str,
    count: u64,
}

#[derive(Deserialize)]
struct StoredDbOwned {
    entries: Vec<StoredEntryOwned>,
}

#[derive(Deserialize)]
struct StoredEntryOwned {
    text: String,
    count: u64,
}

impl TokenDb {
    /// Serializes the database to the versioned binary format used by `token_db`.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let stored = StoredDb {
            entries: self
                .entries
                .iter()
                .map(|entry| StoredEntryRef {
                    text: entry.text(),
                    count: entry.count(),
                })
                .collect(),
        };

        let payload = postcard::to_allocvec(&stored)?;
        let mut output = Vec::with_capacity(MAGIC.len() + payload.len());
        output.extend_from_slice(MAGIC);
        output.extend_from_slice(&payload);
        Ok(output)
    }

    /// Deserializes a database from bytes created by [`TokenDb::to_bytes`].
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if !bytes.starts_with(MAGIC) {
            return Err(crate::Error::InvalidFormat("missing or unsupported format header"));
        }

        let (stored, remainder): (StoredDbOwned, &[u8]) =
            postcard::take_from_bytes(&bytes[MAGIC.len()..])?;
        if !remainder.is_empty() {
            return Err(crate::Error::InvalidFormat("trailing bytes after payload"));
        }
        let entries = stored
            .entries
            .into_iter()
            .map(|entry| TokenEntry {
                text: Arc::from(entry.text),
                count: entry.count,
            })
            .collect();

        Self::from_entries(entries)
    }

    /// Saves the database to `path` using a temporary file in the same directory.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(Path::new("."));
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(&self.to_bytes()?)?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(path)
            .map_err(|error| crate::Error::Io(error.error))?;
        Ok(())
    }

    /// Loads a database from `path`.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_bytes(&fs::read(path)?)
    }
}
