use std::collections::HashMap;

/// Protocol detection cache: (port, process_name) → protocol
pub struct ProtocolCache {
    entries: HashMap<(u16, String), String>,
}

impl ProtocolCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, port: u16, process_name: &str) -> Option<&String> {
        self.entries.get(&(port, process_name.to_string()))
    }

    pub fn insert(&mut self, port: u16, process_name: &str, protocol: &str) {
        self.entries
            .insert((port, process_name.to_string()), protocol.to_string());
    }

    pub fn remove(&mut self, port: u16, process_name: &str) {
        self.entries.remove(&(port, process_name.to_string()));
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for ProtocolCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operations() {
        let mut cache = ProtocolCache::new();
        assert!(cache.get(3000, "node").is_none());

        cache.insert(3000, "node", "http");
        assert_eq!(cache.get(3000, "node"), Some(&"http".to_string()));

        cache.remove(3000, "node");
        assert!(cache.get(3000, "node").is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = ProtocolCache::new();
        cache.insert(3000, "node", "http");
        cache.insert(5432, "postgres", "postgres");
        cache.clear();
        assert!(cache.get(3000, "node").is_none());
        assert!(cache.get(5432, "postgres").is_none());
    }
}
