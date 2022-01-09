use std::hash::Hash;

use bimap::BiMap;
use hashbrown::HashMap;

pub(crate) use bimap::Overwritten;

// The approach used isn't perfect (see comment in max_size_bi_map test).

struct RingBuffer<K, const N: usize> {
    cursor: usize,
    // TODO: Use maybeunint rather than option?
    buffer: [Option<K>; N],
}

impl<K, const N: usize> RingBuffer<K, N> {
    // https://github.com/rust-lang/rust/issues/44796
    const INIT: Option<K> = None;

    fn new() -> Self {
        Self {
            cursor: 0,
            buffer: [Self::INIT; N],
        }
    }

    fn push(&mut self, value: K) -> Option<K> {
        let result = unsafe { self.buffer.get_unchecked_mut(self.cursor).replace(value) };
        if self.cursor + 1 == N {
            self.cursor = 0;
        } else {
            self.cursor += 1;
        }
        result
    }
}

pub(crate) struct MaxSizeHashMap<K, V, const N: usize>
where
    K: Hash + Eq + Clone,
{
    hash_map: HashMap<K, V>,
    ring_buffer: RingBuffer<K, N>,
}

impl<K, V, const N: usize> MaxSizeHashMap<K, V, N>
where
    K: Hash + Eq + Clone,
{
    pub(crate) fn new() -> Self {
        Self {
            hash_map: HashMap::with_capacity(N),
            ring_buffer: RingBuffer::new(),
        }
    }

    pub(crate) fn insert(&mut self, key: K, value: V) -> Option<V> {
        // TODO: Remove clone?
        if let Some(oldest_key) = self.ring_buffer.push(key.clone()) {
            // This may fail if we called Self::remove earlier but we don't really care.
            self.hash_map.remove(&oldest_key);
        }
        self.hash_map.insert(key, value)
    }

    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.hash_map.contains_key(key)
    }

    pub(crate) fn remove(&mut self, key: &K) -> Option<V> {
        self.hash_map.remove(key)
    }
}

pub(crate) struct MaxSizeBiMap<L, R, const N: usize>
where
    L: Hash + Eq + Clone,
    R: Hash + Eq + Clone,
{
    bi_map: BiMap<L, R>,
    // Ideally we wouldn't just blindly use the left one, but the one with the smallest
    // size.
    ring_buffer: RingBuffer<L, N>,
}

impl<L, R, const N: usize> MaxSizeBiMap<L, R, N>
where
    L: Hash + Eq + Clone,
    R: Hash + Eq + Clone,
{
    pub(crate) fn new() -> Self {
        Self {
            bi_map: BiMap::with_capacity(N),
            ring_buffer: RingBuffer::new(),
        }
    }

    pub(crate) fn insert(&mut self, left: L, right: R) -> Overwritten<L, R> {
        // TODO: Remove clone?
        if let Some(oldest_key) = self.ring_buffer.push(left.clone()) {
            // This may fail if we called Self::remove earlier but we don't really care.
            self.bi_map.remove_by_left(&oldest_key);
        }
        self.bi_map.insert(left, right)
    }

    pub(crate) fn get_by_left(&self, key: &L) -> Option<&R> {
        self.bi_map.get_by_left(key)
    }

    pub(crate) fn get_by_right(&self, value: &R) -> Option<&L> {
        self.bi_map.get_by_right(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer() {
        let mut ring_buffer = RingBuffer::<_, 3>::new();
        assert_eq!(ring_buffer.cursor, 0);
        assert_eq!(ring_buffer.buffer, [None, None, None]);

        assert_eq!(ring_buffer.push(1), None);
        assert_eq!(ring_buffer.cursor, 1);
        assert_eq!(ring_buffer.buffer, [Some(1), None, None]);

        assert_eq!(ring_buffer.push(2), None);
        assert_eq!(ring_buffer.cursor, 2);
        assert_eq!(ring_buffer.buffer, [Some(1), Some(2), None]);

        assert_eq!(ring_buffer.push(3), None);
        assert_eq!(ring_buffer.cursor, 0);
        assert_eq!(ring_buffer.buffer, [Some(1), Some(2), Some(3)]);

        assert_eq!(ring_buffer.push(4), Some(1));
        assert_eq!(ring_buffer.cursor, 1);
        assert_eq!(ring_buffer.buffer, [Some(4), Some(2), Some(3)]);

        assert_eq!(ring_buffer.push(5), Some(2));
        assert_eq!(ring_buffer.cursor, 2);
        assert_eq!(ring_buffer.buffer, [Some(4), Some(5), Some(3)]);

        assert_eq!(ring_buffer.push(6), Some(3));
        assert_eq!(ring_buffer.cursor, 0);
        assert_eq!(ring_buffer.buffer, [Some(4), Some(5), Some(6)]);

        assert_eq!(ring_buffer.push(7), Some(4));
        assert_eq!(ring_buffer.cursor, 1);
        assert_eq!(ring_buffer.buffer, [Some(7), Some(5), Some(6)]);
    }

    #[test]
    fn max_size_hash_map() {
        let mut hash_map = MaxSizeHashMap::<_, _, 3>::new();
        assert_eq!(hash_map.ring_buffer.cursor, 0);
        assert_eq!(hash_map.ring_buffer.buffer, [None, None, None]);

        assert_eq!(hash_map.insert(1, "When"), None);
        assert!(hash_map.contains_key(&1));
        assert_eq!(hash_map.ring_buffer.cursor, 1);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(1), None, None]);

        assert_eq!(hash_map.insert(2, "I"), None);
        assert!(hash_map.contains_key(&2));
        assert_eq!(hash_map.ring_buffer.cursor, 2);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(1), Some(2), None]);

        assert_eq!(hash_map.insert(3, "get"), None);
        assert!(hash_map.contains_key(&3));
        assert_eq!(hash_map.ring_buffer.cursor, 0);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(1), Some(2), Some(3)]);

        assert_eq!(hash_map.insert(4, "signed,"), None);
        assert!(!hash_map.contains_key(&1));
        assert_eq!(hash_map.ring_buffer.cursor, 1);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(4), Some(2), Some(3)]);

        assert_eq!(hash_map.remove(&4), Some("signed,"));
        assert!(!hash_map.contains_key(&4));
        assert_eq!(hash_map.ring_buffer.cursor, 1);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(4), Some(2), Some(3)]);

        assert_eq!(hash_map.insert(5, "homie,"), None);
        assert_eq!(hash_map.ring_buffer.cursor, 2);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(4), Some(5), Some(3)]);
        assert_eq!(hash_map.insert(5, "I'ma"), Some("homie,"));
        assert_eq!(hash_map.ring_buffer.cursor, 0);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(4), Some(5), Some(5)]);

        assert_eq!(hash_map.insert(6, "act"), None);
        assert_eq!(hash_map.ring_buffer.cursor, 1);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(6), Some(5), Some(5)]);
    }

    #[test]
    fn max_size_bi_map() {
        let mut bi_map = MaxSizeBiMap::<_, _, 3>::new();
        assert_eq!(bi_map.ring_buffer.cursor, 0);
        assert_eq!(bi_map.ring_buffer.buffer, [None, None, None]);

        assert_eq!(bi_map.insert(1, "a"), Overwritten::Neither);
        assert_eq!(bi_map.ring_buffer.cursor, 1);
        assert_eq!(bi_map.ring_buffer.buffer, [Some(1), None, None]);

        assert_eq!(bi_map.get_by_left(&1), Some(&"a"));
        assert_eq!(bi_map.get_by_right(&"a"), Some(&1));

        assert_eq!(bi_map.insert(2, "fool"), Overwritten::Neither);
        assert_eq!(bi_map.ring_buffer.cursor, 2);
        assert_eq!(bi_map.ring_buffer.buffer, [Some(1), Some(2), None]);

        assert_eq!(bi_map.insert(3, "Hit"), Overwritten::Neither);
        assert_eq!(bi_map.ring_buffer.cursor, 0);
        assert_eq!(bi_map.ring_buffer.buffer, [Some(1), Some(2), Some(3)]);

        assert_eq!(bi_map.insert(4, "the"), Overwritten::Neither);
        assert_eq!(bi_map.ring_buffer.cursor, 1);
        assert_eq!(bi_map.ring_buffer.buffer, [Some(4), Some(2), Some(3)]);

        assert_eq!(bi_map.get_by_left(&1), None);

        assert_eq!(bi_map.get_by_left(&4), Some(&"the"));
        assert_eq!(bi_map.get_by_right(&"the"), Some(&4));

        assert_eq!(bi_map.insert(3, "dance"), Overwritten::Left(3, "Hit"));
        assert_eq!(bi_map.ring_buffer.cursor, 2);
        // TODO: is this really what we want? "dance" would be removed on the next push
        // as it's place in the ring buffer before was not removed. The client and protocol
        // are pretty resilient and there shouldn't be many overwrites in actual use so this
        // shouldn't be a big issue.
        // We could avoid this issue by also associating each key (in a normal hashmap) with
        // an integer, incrementing when it gets inserted, decrementing when it gets popped
        // from the buffer, and deleting when == 0.
        assert_eq!(bi_map.ring_buffer.buffer, [Some(4), Some(3), Some(3)]);
    }
}
