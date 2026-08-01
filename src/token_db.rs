use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use struct_extractors::extract_accessors;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenDBEntry {
    pub text: Arc<str>,
    pub count: usize,
    pub index: usize,
}

#[extract_accessors]
#[derive(Default, Debug, Serialize)]
pub struct TokenDB {
    #[serde(skip)]
    map: FxHashMap<Arc<str>, (usize, usize)>, // (count, index)
    #[access(get_ref)]
    list: Vec<TokenDBEntry>,
}

impl TokenDB {
    #[inline(always)]
    pub fn get_by_index(&self, index: usize) -> &TokenDBEntry {
        &self.list[index]
    }

    /// Looks up a token via a `&str` reference without any heap allocation.
    #[inline(always)]
    pub fn get_by_name(&self, name: &str) -> Option<&TokenDBEntry> {
        // OPTIMIZATION: Leverages HashMap's Borrow trait for allocation-free lookups.
        self.map.get(name).map(|(_, idx)| &self.list[*idx])
    }

    #[doc(hidden)]
    pub fn clear_internal_map_for_testing(&mut self) {
        self.map.clear();
    }

    /// Pushes a token into the database. Guaranteed to be allocation-free if the token exists.
    pub fn push(&mut self, item: &str) -> usize {
        // OPTIMIZATION: First check the map without allocating an Arc.
        if let Some((count, index)) = self.map.get_mut(item) {
            *count += 1;
            self.list[*index].count = *count;
            return *index;
        }

        // Allocate only if the token is brand new.
        let key: Arc<str> = Arc::from(item);
        let index = self.list.len();

        self.map.insert(key.clone(), (1, index));
        self.list.push(TokenDBEntry {
            text: key,
            count: 1,
            index,
        });

        index
    }

    /// Merges another TokenDB into self. Reuses the optimized `join_with_map` logic.
    pub fn join(&mut self, other: &TokenDB) {
        // OPTIMIZATION: Prevents code duplication and reduces maintenance overhead.
        let _ = self.join_with_map(other);
    }

    /// Merges all entries from `other` into `self` and returns an index‑mapping.
    ///
    /// # Description
    /// For every `TokenDBEntry` in `other`, this function:
    /// - Checks whether the token text already exists in `self`.
    /// - If it exists:
    ///     - The count is increased by `entry.count`.
    ///     - The existing entry in `self` is updated.
    ///     - The index of the entry in `other` is mapped to the existing index in `self`.
    /// - If it does not exist:
    ///     - A new entry is appended to `self.list`.
    ///     - The token is inserted into `self.map`.
    ///     - The index of the entry in `other` is mapped to the newly created index.
    ///
    /// # Returns
    /// A mapping from `other`'s indices to the final indices in `self`.
    /// This allows callers to track how entries from `other` were merged or deduplicated.
    ///
    /// # Example
    /// If `other.list[3]` corresponds to a token that already exists at `self.list[10]`,
    /// the returned map will contain: `3 -> 10`.
    pub fn join_with_map(&mut self, other: &TokenDB) -> Vec<usize> {
        let mut index_map = Vec::with_capacity(other.list.len());

        // OPTIMIZATION: Pre-allocates memory to avoid expensive re-allocations during large merges.
        self.list.reserve(other.list.len());
        self.map.reserve(other.list.len());

        for entry in &other.list {
            // OPTIMIZATION: Allocation-free map lookup using a borrowed reference.
            let final_index = if let Some((count, self_idx)) = self.map.get_mut(&*entry.text) {
                *count += entry.count;
                self.list[*self_idx].count = *count;
                *self_idx
            } else {
                let new_self_idx = self.list.len();
                let key = entry.text.clone();
                self.map.insert(key.clone(), (entry.count, new_self_idx));
                self.list.push(TokenDBEntry {
                    text: key,
                    count: entry.count,
                    index: new_self_idx,
                });
                new_self_idx
            };

            index_map.push(final_index);
        }

        index_map
    }

    /// Rebuilds the internal lookup map and synchronizes internal entry indices.
    pub fn rebuild_map(&mut self) {
        self.map.clear();
        self.map.reserve(self.list.len());
        for idx in 0..self.list.len() {
            let entry = &mut self.list[idx];
            // BUGFIX: Updates the inner entry's index field to match its real vector position,
            // in case the file data was corrupted or out of sync before deserialization.
            entry.index = idx;
            self.map.insert(entry.text.clone(), (entry.count, idx));
        }
    }

    pub fn save(&self, output: &Path) -> std::io::Result<()> {
        // OPTIMIZATION: Clean error handling instead of panicking via `.unwrap()`.
        let encoded = postcard::to_allocvec(&self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(output, encoded)
    }

    pub fn load(input: &Path) -> std::io::Result<TokenDB> {
        let data = fs::read(input)?;
        let decoded: TokenDB = postcard::from_bytes(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(decoded)
    }
}

impl<'de> Deserialize<'de> for TokenDB {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let list = Vec::<TokenDBEntry>::deserialize(deserializer)?;
        let mut tmp = TokenDB {
            map: FxHashMap::default(),
            list,
        };
        tmp.rebuild_map();
        Ok(tmp)
    }
}

