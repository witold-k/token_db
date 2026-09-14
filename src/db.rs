use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{Error, Result};

/// Stable numeric identifier assigned to a token.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TokenId(u32);

impl TokenId {
    /// Returns the underlying integer ID.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    pub(crate) fn from_index(index: usize) -> Result<Self> {
        u32::try_from(index).map(Self).map_err(|_| Error::TooManyTokens)
    }

    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A token and its observed frequency.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenEntry {
    pub(crate) text: Arc<str>,
    pub(crate) count: u64,
}

impl TokenEntry {
    /// Returns the token text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns how many times the token was inserted.
    #[must_use]
    pub const fn count(&self) -> u64 {
        self.count
    }
}

/// Stores unique tokens, stable IDs, and frequency counts.
#[derive(Clone, Debug, Default)]
pub struct TokenDb {
    pub(crate) entries: Vec<TokenEntry>,
    pub(crate) index: FxHashMap<Arc<str>, TokenId>,
}

impl TokenDb {
    /// Creates an empty token database.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of unique tokens.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` when the database contains no tokens.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Inserts one occurrence of `token` and returns its stable ID.
    ///
    /// Existing tokens are not reallocated.
    pub fn insert(&mut self, token: &str) -> Result<TokenId> {
        if let Some(&id) = self.index.get(token) {
            let entry = &mut self.entries[id.index()];
            entry.count = entry.count.checked_add(1).ok_or(Error::CountOverflow)?;
            return Ok(id);
        }

        let id = TokenId::from_index(self.entries.len())?;
        let text: Arc<str> = Arc::from(token);
        self.entries.push(TokenEntry {
            text: Arc::clone(&text),
            count: 1,
        });
        self.index.insert(text, id);
        Ok(id)
    }

    /// Returns the entry for `id`, or `None` when the ID is out of range.
    #[must_use]
    pub fn get(&self, id: TokenId) -> Option<&TokenEntry> {
        self.entries.get(id.index())
    }

    /// Returns the ID assigned to `token`.
    #[must_use]
    pub fn id(&self, token: &str) -> Option<TokenId> {
        self.index.get(token).copied()
    }

    /// Returns the entry associated with `token`.
    #[must_use]
    pub fn get_by_token(&self, token: &str) -> Option<&TokenEntry> {
        self.id(token).and_then(|id| self.get(id))
    }

    /// Iterates over entries in stable ID order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (TokenId, &TokenEntry)> + '_ {
        self.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (TokenId(index as u32), entry))
    }

    /// Merges `other` into this database.
    ///
    /// The returned vector maps each ID in `other` to its final ID in `self`.
    /// The operation validates all count additions before changing `self`.
    pub fn merge(&mut self, other: &Self) -> Result<Vec<TokenId>> {
        let new_tokens = other
            .entries
            .iter()
            .filter(|entry| !self.index.contains_key(entry.text.as_ref()))
            .count();

        if new_tokens > 0 {
            let last_new_index = self
                .entries
                .len()
                .checked_add(new_tokens - 1)
                .ok_or(Error::TooManyTokens)?;
            TokenId::from_index(last_new_index)?;
        }

        for entry in &other.entries {
            if let Some(&id) = self.index.get(entry.text.as_ref()) {
                self.entries[id.index()]
                    .count
                    .checked_add(entry.count)
                    .ok_or(Error::CountOverflow)?;
            }
        }

        self.entries.reserve(new_tokens);
        self.index.reserve(new_tokens);

        let mut mapping = Vec::with_capacity(other.len());
        for entry in &other.entries {
            let id = if let Some(&id) = self.index.get(entry.text.as_ref()) {
                self.entries[id.index()].count += entry.count;
                id
            } else {
                let id = TokenId::from_index(self.entries.len())?;
                let text = Arc::clone(&entry.text);
                self.entries.push(TokenEntry {
                    text: Arc::clone(&text),
                    count: entry.count,
                });
                self.index.insert(text, id);
                id
            };
            mapping.push(id);
        }

        Ok(mapping)
    }

    pub(crate) fn from_entries(entries: Vec<TokenEntry>) -> Result<Self> {
        let mut index = FxHashMap::with_capacity_and_hasher(entries.len(), Default::default());

        for (position, entry) in entries.iter().enumerate() {
            if entry.count == 0 {
                return Err(Error::InvalidFormat("token count must be greater than zero"));
            }

            let id = TokenId::from_index(position)?;
            if index.insert(Arc::clone(&entry.text), id).is_some() {
                return Err(Error::DuplicateToken(entry.text.to_string()));
            }
        }

        Ok(Self { entries, index })
    }
}

