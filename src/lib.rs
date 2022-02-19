use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::slice::{Iter, IterMut};

pub struct HashMap<K, V> {
    // buckets: [Entry<K,V>; capacity],
    // Once initialized, it‘s capacity will not change, which is guaranteed by program logic
    buckets: Vec<Entry<K, V>>,
    capacity: usize,
    // always <= capacity
    length: usize,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    // create a HashMap with default capacity 100
    pub fn new() -> HashMap<K, V> {
        HashMap::with_capacity(100)
    }

    // create a HashMap with capacity
    pub fn with_capacity(capacity: usize) -> HashMap<K, V> {
        let mut buckets = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buckets.push(Entry::Empty);
        }
        HashMap {
            buckets: buckets,
            capacity: capacity,
            length: 0,
        }
    }

    #[inline]
    pub fn cap(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn clear(&mut self) {
        for i in 0..self.capacity {
            self.buckets[i] = Entry::Empty;
        }
    }

    // Hash the key to get a bucket index, index < self.capacity
    fn find_bucket<Q: ?Sized>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.cap()
    }

    // Returns true when the bucket not overflow, otherwise returns false
    pub fn insert(&mut self, key: K, val: V) -> (bool, Option<V>) {
        let mut index = self.find_bucket(&key);
        let old_entry = &mut self.buckets[index];
        if let Entry::Empty = old_entry {
            *old_entry = Entry::KeyPair(key, val);
            self.length += 1;
            return (true, None);
        }
        if let Entry::KeyPair(k, ref mut v) = old_entry {
            if k == &key {
                return (true, Some(std::mem::replace(v, val)));
            }
        }
        // When no bucket is available, inserts are not allowed
        if self.len() >= self.cap() {
            return (false, None);
        }
        // Resolve hash collision
        loop {
            index += 1;
            index = index % self.cap();
            let old_entry = &self.buckets[index];
            if let Entry::Empty = old_entry {
                self.buckets[index] = Entry::KeyPair(key, val);
                self.length += 1;
                break;
            }
        }
        (true, None)
    }

    // Calculate hash of the key, and if there is a conflict, search backward in turn,
    // return a `None` if there is no space else return a bucket index
    fn probe_key_bucket<Q: ?Sized>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut index = self.find_bucket(key);
        let entry = &self.buckets[index];

        if let Entry::Empty = entry {
            return None;
        }
        if entry.key().unwrap().borrow() == key {
            return Some(index);
        }
        // hash collision
        let start_index = index;
        loop {
            index += 1;
            index = index % self.cap();
            if index == start_index {
                break None;
            }
            let entry = &self.buckets[index];
            if entry.key().unwrap().borrow() == key {
                break Some(index);
            }
        }
    }

    // Returns a reference to the value corresponding to the key
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let pi = self.probe_key_bucket(key);
        match pi {
            None => None,
            Some(i) => match &self.buckets[i] {
                &Entry::Empty => None,
                &Entry::KeyPair(_, ref val) => Some(val),
            },
        }
    }

    // Returns a mutable reference to the value corresponding to the key
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let pi = self.probe_key_bucket(key);
        match pi {
            None => None,
            Some(i) => match &mut self.buckets[i] {
                &mut Entry::Empty => None,
                &mut Entry::KeyPair(_, ref mut val) => Some(val),
            },
        }
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get(key).is_some()
    }

    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys { inner: self.iter() }
    }

    pub fn values(&self) -> Values<'_, K, V> {
        Values { inner: self.iter() }
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let pi = self.probe_key_bucket(key);
        match pi {
            None => false,
            Some(i) => {
                self.buckets[i] = Entry::Empty;
                self.length -= 1;
                true
            }
        }
    }

    // An iterator visiting all key-value pairs, the iterator element type is `(&'a K, &'a V)`
    #[inline]
    pub fn iter(&self) -> HashMapIter<K, V> {
        HashMapIter {
            inner: self.buckets.iter(),
        }
    }

    // An iterator visiting all key-value pairs in arbitrary order, with mutable references to the values,
    // the iterator element type is `(&'a K, &'a mut V)`
    #[inline]
    pub fn iter_mut(&mut self) -> HashMapIterMut<K, V> {
        HashMapIterMut {
            inner: self.buckets.iter_mut(),
        }
    }
}

impl<K, V> PartialEq for HashMap<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    fn eq(&self, other: &HashMap<K, V>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V> Eq for HashMap<K, V>
where
    K: Eq + Hash,
    V: Eq,
{
}

pub struct HashMapIter<'a, K: 'a, V: 'a> {
    inner: Iter<'a, Entry<K, V>>,
}

impl<'a, K, V> Iterator for HashMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        let mut n = self.inner.next();
        loop {
            match n {
                Some(entry) if !entry.is_empty() => {
                    return Some((entry.key().unwrap(), entry.value().unwrap()))
                }
                Some(..) => {
                    n = self.inner.next();
                }
                None => {
                    return None;
                }
            }
        }
    }
}

pub struct HashMapIterMut<'a, K: 'a, V: 'a> {
    inner: IterMut<'a, Entry<K, V>>,
}

impl<'a, K, V> Iterator for HashMapIterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        let mut n = self.inner.next();
        loop {
            match n {
                Some(&mut Entry::KeyPair(ref key, ref mut val)) => return Some((key, val)),
                Some(..) => {
                    n = self.inner.next();
                }
                None => {
                    return None;
                }
            }
        }
    }
}

pub struct Keys<'a, K: 'a, V: 'a> {
    inner: HashMapIter<'a, K, V>,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<&'a K> {
        self.inner.next().map(|(k, _)| k)
    }
}

pub struct Values<'a, K: 'a, V: 'a> {
    inner: HashMapIter<'a, K, V>,
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<&'a V> {
        self.inner.next().map(|(_, v)| v)
    }
}

pub enum Entry<K, V> {
    Empty,
    KeyPair(K, V),
}

impl<K, V> Entry<K, V> {
    #[inline]
    pub fn key(&self) -> Option<&K> {
        match *self {
            Entry::KeyPair(ref k, _) => Some(k),
            _ => None,
        }
    }

    #[inline]
    pub fn value(&self) -> Option<&V> {
        match *self {
            Entry::KeyPair(_, ref v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn value_mut(&mut self) -> Option<&mut V> {
        match *self {
            Entry::KeyPair(_, ref mut v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        match *self {
            Entry::Empty => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::HashMap;

    #[test]
    fn default_new() {
        let m: HashMap<usize, usize> = HashMap::new();
        assert_eq!(m.len(), 0);
        assert_eq!(m.cap(), 100);
        assert_eq!(m.get(&1), None);

        let mut count = 0;
        for (_, _) in m.iter() {
            count += 1;
        }
        assert_eq!(count, 0);
    }

    #[test]
    fn test_common() {
        let mut m = HashMap::with_capacity(3);
        assert_eq!(m.cap(), 3);
        assert_eq!(m.len(), 0);

        // insert key 1, get result
        assert_eq!(m.insert(1, 100), (true, None));
        match m.get(&1) {
            Some(v) => assert_eq!(*v, 100),
            None => panic!("panicure!"),
        }

        // insert more keys
        assert_eq!(m.insert(2, 200), (true, None));
        assert_eq!(m.insert(3, 300), (true, None));
        assert_eq!(m.len(), 3);

        // our hashmap capacity is 3, now it's filled, can't be insert anymore
        assert_eq!(m.insert(4, 400), (false, None));

        // although it's filled, but we can update the existing key, return true
        assert_eq!(m.insert(1, 1000), (true, Some(100)));
        // assert the new value
        match m.get(&1) {
            Some(v) => assert_eq!(*v, 1000),
            None => panic!("panicure!"),
        }

        // remove key 1, vacated a bucket
        assert_eq!(m.remove(&1), true);
        match m.get(&1) {
            Some(y) => panic!("key {} must not exists!", y),
            None => {}
        }
        assert_eq!(m.contains_key(&1), false);
        assert_eq!(m.len(), 2);
        for (&k, &v) in m.iter() {
            match k {
                2 => assert_eq!(v, 200),
                3 => assert_eq!(v, 300),
                _ => {}
            }
        }

        // we can insert a key/value now
        assert_eq!(m.insert(4, 400), (true, None));
        assert_eq!(m.len(), 3);
        for (&k, &v) in m.iter() {
            match k {
                2 => assert_eq!(v, 200),
                3 => assert_eq!(v, 300),
                4 => assert_eq!(v, 400),
                _ => {}
            }
        }
    }

    #[test]
    fn test_get_mut() {
        let mut m = HashMap::new();
        m.insert("foo", 42);
        m.insert("bar", 43);
        match m.get("foo") {
            Some(v) => assert_eq!(*v, 42),
            None => panic!("panicure!"),
        }

        // change value of key `foo`
        match m.get_mut("foo") {
            Some(v) => *v = 40,
            None => panic!("panicure!"),
        }

        // get the changed value
        match m.get("foo") {
            Some(v) => assert_eq!(*v, 40),
            None => panic!("panicure!"),
        }
    }

    #[test]
    fn test_iter_mut() {
        let mut m = HashMap::new();
        m.insert("foo", 42);
        m.insert("bar", 43);
        for (_, v) in m.iter_mut() {
            *v += 1;
        }

        for (&k, &v) in m.iter() {
            match k {
                "foo" => assert_eq!(v, 43),
                "bar" => assert_eq!(v, 44),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_keys_values() {
        let mut map = HashMap::new();
        map.insert("foo", 42);
        map.insert("bar", 43);

        for key in map.keys() {
            println!("{}", key);
        }

        for val in map.values() {
            println!("{}", val);
        }
    }

    #[test]
    fn test_eq() {
        let mut map1 = HashMap::new();
        map1.insert("foo", 42);
        map1.insert("bar", 43);

        let mut map2 = HashMap::new();
        map2.insert("foo", 42);
        map2.insert("bar", 43);

        let eq = map1 == map2;
        assert_eq!(true, eq)
    }
}
