// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Write};
use std::path::Path;
use std::sync::Arc;

use crate::{Error, Result, TokenDb, TokenEntry};

const MAGIC: &[u8; 8] = b"TOKENDB1";

/// Prevent corrupted/untrusted files from requesting absurd allocations.
///
/// This is intentionally very generous for something that is supposed to
/// represent a token.
const MAX_TOKEN_BYTES: usize = 16 * 1024 * 1024;

impl TokenDb {
    /// Serializes the database into memory.
    ///
    /// Prefer [`TokenDb::write_to`] or [`TokenDb::save`] for large databases,
    /// because this method necessarily allocates the complete encoded database.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        self.write_to(&mut bytes)?;
        Ok(bytes)
    }

    /// Deserializes a database from bytes created by [`TokenDb::to_bytes`].
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::read_from(Cursor::new(bytes))
    }

    /// Writes the database to `writer`.
    ///
    /// Serialization is streamed directly to the writer and does not allocate
    /// a second copy of the database.
    pub fn write_to<W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_all(MAGIC)?;

        // usize -> u64 is safe on all currently supported Rust platforms.
        writer.write_all(&(self.entries.len() as u64).to_le_bytes())?;

        for entry in &self.entries {
            let text = entry.text().as_bytes();

            let text_len =
                u32::try_from(text.len()).map_err(|_| Error::InvalidFormat("token too large"))?;

            writer.write_all(&text_len.to_le_bytes())?;
            writer.write_all(text)?;
            writer.write_all(&entry.count().to_le_bytes())?;
        }

        Ok(())
    }

    /// Reads a database from `reader`.
    ///
    /// The input is validated before a usable `TokenDb` is returned.
    pub fn read_from<R: Read>(mut reader: R) -> Result<Self> {
        let mut magic = [0_u8; MAGIC.len()];
        reader.read_exact(&mut magic)?;

        if &magic != MAGIC {
            return Err(Error::InvalidFormat(
                "missing or unsupported format header",
            ));
        }

        let entry_count = read_u64(&mut reader)?;

        // At most u32::MAX + 1 entries can exist because IDs are u32 and start
        // at zero.
        if entry_count > u64::from(u32::MAX) + 1 {
            return Err(Error::TooManyTokens);
        }

        let mut entries = Vec::new();

        for _ in 0..entry_count {
            let text_len = read_u32(&mut reader)? as usize;

            if text_len > MAX_TOKEN_BYTES {
                return Err(Error::InvalidFormat("token exceeds maximum size"));
            }

            let mut text = vec![0_u8; text_len];
            reader.read_exact(&mut text)?;

            let text = String::from_utf8(text)
                .map_err(|_| Error::InvalidFormat("token is not valid UTF-8"))?;

            let count = read_u64(&mut reader)?;

            entries.push(TokenEntry {
                text: Arc::from(text),
                count,
            });
        }

        ensure_eof(&mut reader)?;

        // This performs the semantic validation in one central place:
        //
        // - count != 0
        // - no duplicate tokens
        // - every position fits in TokenId
        // - index is constructed consistently
        Self::from_entries(entries)
    }

    /// Saves the database atomically to `path`.
    ///
    /// The database is streamed to a temporary file in the destination
    /// directory. The destination is replaced only after the new file has been
    /// completely written and synchronized.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));

        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;

        {
            let mut writer = BufWriter::new(temporary.as_file_mut());

            self.write_to(&mut writer)?;
            writer.flush()?;
        }

        // Ensure file contents have reached the filesystem before publishing
        // the new file under the final name.
        temporary.as_file().sync_all()?;

        temporary
            .persist(path)
            .map_err(|error| Error::Io(error.error))?;

        sync_directory(parent)?;

        Ok(())
    }

    /// Loads a database from `path`.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        Self::read_from(reader)
    }
}

fn read_u32<R: Read>(reader: &mut R) -> Result<u32> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64<R: Read>(reader: &mut R) -> Result<u64> {
    let mut bytes = [0_u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

fn ensure_eof<R: Read>(reader: &mut R) -> Result<()> {
    let mut byte = [0_u8; 1];

    match reader.read(&mut byte) {
        Ok(0) => Ok(()),
        Ok(_) => Err(Error::InvalidFormat("trailing bytes after database")),
        Err(error) => Err(Error::Io(error)),
    }
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)?.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<()> {
    Ok(())
}
