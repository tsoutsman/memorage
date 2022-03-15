use std::hash::Hash;

use hashbrown::HashMap;

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
    key_overwrites: HashMap<K, usize>,
}

impl<K, V, const N: usize> MaxSizeHashMap<K, V, N>
where
    K: Hash + Eq + Clone + std::fmt::Debug,
{
    pub(crate) fn new() -> Self {
        Self {
            hash_map: HashMap::with_capacity(N),
            ring_buffer: RingBuffer::new(),
            key_overwrites: HashMap::with_capacity(N),
        }
    }

    pub(crate) fn insert(&mut self, key: K, value: V) -> Option<V> {
        // TODO: Do we want this
        // if self.hash_map.get(&key) == Some(&value) {
        //     return None;
        // }
        // TODO: Remove clone?
        if let Some(oldest_key) = self.ring_buffer.push(key.clone()) {
            if let Some(x) = self.key_overwrites.get_mut(&oldest_key) {
                *x -= 1;
                if *x == 0 {
                    self.key_overwrites.remove(&oldest_key);
                    // This may fail if user called Self::remove earlier but we don't
                    // really care.
                    self.hash_map.remove(&oldest_key);
                }
            } else {
                debug_assert!(false, "oldest key not in key overwrites");
            }
        }
        if let Some(x) = self.key_overwrites.get_mut(&key) {
            *x += 1;
        } else {
            // TODO: Remove clone?
            self.key_overwrites.insert(key.clone(), 1);
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
        macro_rules! assert_map_contents {
            ($i:ident;$($k:expr => $v:expr),*$(,)?) => {
                $(
                    assert!($i.contains_key(&$k));
                    assert_eq!($i.hash_map.get(&$k), Some(&$v));
                )*
            }
        }

        let mut hash_map = MaxSizeHashMap::<_, _, 3>::new();
        assert_eq!(hash_map.ring_buffer.cursor, 0);
        assert_eq!(hash_map.ring_buffer.buffer, [None, None, None]);

        assert_eq!(hash_map.insert(1, "When"), None);
        assert_eq!(hash_map.ring_buffer.cursor, 1);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(1), None, None]);
        assert_map_contents! {
            hash_map;
            1 => "When",
        }

        assert_eq!(hash_map.insert(2, "I"), None);
        assert_eq!(hash_map.ring_buffer.cursor, 2);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(1), Some(2), None]);
        assert_map_contents! {
            hash_map;
            1 => "When",
            2 => "I",
        }

        assert_eq!(hash_map.insert(3, "get"), None);
        assert_eq!(hash_map.ring_buffer.cursor, 0);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(1), Some(2), Some(3)]);
        assert_map_contents! {
            hash_map;
            1 => "When",
            2 => "I",
            3 => "get",
        }

        assert_eq!(hash_map.insert(4, "signed,"), None);
        assert_eq!(hash_map.ring_buffer.cursor, 1);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(4), Some(2), Some(3)]);
        assert_map_contents! {
            hash_map;
            2 => "I",
            3 => "get",
            4 => "signed,",
        }

        assert_eq!(hash_map.remove(&4), Some("signed,"));
        assert_eq!(hash_map.ring_buffer.cursor, 1);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(4), Some(2), Some(3)]);
        assert_map_contents! {
            hash_map;
            2 => "I",
            3 => "get",
        }
        assert!(!hash_map.contains_key(&4));

        assert_eq!(hash_map.insert(5, "homie,"), None);
        assert_eq!(hash_map.ring_buffer.cursor, 2);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(4), Some(5), Some(3)]);
        assert_map_contents! {
            hash_map;
            3 => "get",
            5 => "homie,",
        }

        // TODO: Keys are sometimes removed when they don't need to be. This happens
        // after a full rotation of the buffer so it probs isn't important.
        assert_eq!(hash_map.insert(5, "I'ma"), Some("homie,"));
        assert_eq!(hash_map.ring_buffer.cursor, 0);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(4), Some(5), Some(5)]);
        assert_map_contents! {
            hash_map;
            5 => "I'ma",
        }
        assert!(!hash_map.contains_key(&3));

        assert_eq!(hash_map.insert(6, "act"), None);
        assert_eq!(hash_map.ring_buffer.cursor, 1);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(6), Some(5), Some(5)]);
        assert_map_contents! {
            hash_map;
            5 => "I'ma",
            6 => "act",
        }

        assert_eq!(hash_map.insert(7, "a"), None);
        assert_eq!(hash_map.ring_buffer.cursor, 2);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(6), Some(7), Some(5)]);
        assert_map_contents! {
            hash_map;
            5 => "I'ma",
            6 => "act",
            7 => "a",
        }

        assert_eq!(hash_map.insert(8, "fool"), None);
        assert_eq!(hash_map.ring_buffer.cursor, 0);
        assert_eq!(hash_map.ring_buffer.buffer, [Some(6), Some(7), Some(8)]);
        assert_map_contents! {
            hash_map;
            6 => "act",
            7 => "a",
            8 => "fool",
        }
    }
}
