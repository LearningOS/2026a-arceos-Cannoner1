//! A minimal `HashMap` for no_std: chaining + FNV-1a + doubling rehash.
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};

pub struct FnvHasher(u64);

impl Hasher for FnvHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    len: usize,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    pub fn with_capacity(cap: usize) -> Self {
        let cap = cap.max(4).next_power_of_two();
        let mut buckets = Vec::with_capacity(cap);
        for _ in 0..cap {
            buckets.push(Vec::new());
        }
        Self { buckets, len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    fn hash(&self, k: &K) -> usize
    where
        K: Hash,
    {
        let mut h = FnvHasher(0xcbf2_9ce4_8422_2325); // FNV offset basis
        k.hash(&mut h);
        (h.finish() as usize) & (self.buckets.len() - 1)
    }

    fn grow(&mut self)
    where
        K: Hash,
    {
        let new_len = self.buckets.len() * 2;
        let old = core::mem::replace(&mut self.buckets, Vec::with_capacity(new_len));
        for _ in 0..new_len {
            self.buckets.push(Vec::new());
        }
        for bucket in old {
            for (k, v) in bucket {
                let idx = self.hash(&k);
                self.buckets[idx].push((k, v));
            }
        }
    }
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let idx = self.hash(&k);
        for entry in self.buckets[idx].iter_mut() {
            if entry.0 == k {
                return Some(core::mem::replace(&mut entry.1, v));
            }
        }
        self.buckets[idx].push((k, v));
        self.len += 1;
        // 负载因子 ~1 时翻倍
        if self.len > self.buckets.len() {
            self.grow();
        }
        None
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        let idx = self.hash(k);
        self.buckets[idx]
            .iter()
            .find(|(ek, _)| ek == k)
            .map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        let idx = self.hash(k);
        self.buckets[idx]
            .iter_mut()
            .find(|(ek, _)| ek == k)
            .map(|(_, v)| v)
    }

    pub fn contains_key(&self, k: &K) -> bool {
        self.get(k).is_some()
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        let idx = self.hash(k);
        let bucket = &mut self.buckets[idx];
        if let Some(pos) = bucket.iter().position(|(ek, _)| ek == k) {
            self.len -= 1;
            Some(bucket.swap_remove(pos).1)
        } else {
            None
        }
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            buckets: self.buckets.iter(),
            cur: None,
        }
    }
}

impl<K, V> Default for HashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

// ---- iterator ----

pub struct Iter<'a, K, V> {
    buckets: core::slice::Iter<'a, Vec<(K, V)>>,
    cur: Option<core::slice::Iter<'a, (K, V)>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(c) = self.cur.as_mut() {
                if let Some((k, v)) = c.next() {
                    return Some((k, v));
                }
            }
            match self.buckets.next() {
                Some(b) => self.cur = Some(b.iter()),
                None => return None,
            }
        }
    }
}