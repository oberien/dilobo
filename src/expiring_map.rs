use std::collections::{VecDeque, HashMap};
use std::time::{Instant, Duration};

#[derive(Debug, Clone)]
pub struct ExpiringMap<K, V> where K: ::std::cmp::Eq + ::std::hash::Hash {
    map: HashMap<K, V>,
    queue: VecDeque<(K, Instant)>,
    ttl: Duration,
}

impl<K, V> ExpiringMap<K, V> where K: ::std::cmp::Eq + ::std::hash::Hash + Clone {
    pub fn new(ttl: Duration) -> ExpiringMap<K, V> {
        ExpiringMap {
            map: HashMap::new(),
            queue: VecDeque::new(),
            ttl: ttl,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.cleanup();
        self.map.insert(key.clone(), value);
        self.queue.push_back((key, Instant::now() + self.ttl));
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }

    fn cleanup(&mut self) {
        let now = Instant::now();
        while self.queue.front().is_some() && self.queue.front().unwrap().1 < now {
            let (key, _) = self.queue.pop_front().unwrap();
            self.map.remove(&key);
        }
    }
}
