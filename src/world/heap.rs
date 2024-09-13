use std::{cmp::Ordering, collections::HashMap};

use bevy::reflect::Map;

// Heap or Priority Queue
// Weak ordered tree
// Min Heap: every child / grandchild is smaller
// Max Heap: every child / grandchild is larger
// Adjust tree on every insert/delete
// No traversal
#[derive(Debug)]
pub enum HeapError {
    LengthIsZero,
    LookupReturnedNone,
}

#[derive(Debug)]
pub struct MinHeapMap<T> {
    data: Vec<T>,
    lookup: HashMap<(u32, u32), usize>,
    length: usize,
}

pub trait Heapable {
    fn x(&self) -> u32;
    fn y(&self) -> u32;
    fn lookup_key(&self) -> (u32, u32) {
        (self.x(), self.y())
    }
}

impl<T: Ord + PartialEq + Clone + std::fmt::Debug + Heapable> From<Vec<T>> for MinHeapMap<T> {
    fn from(value: Vec<T>) -> Self {
        let mut new = Self::new();
        for v in value {
            new.insert(v);
        }
        assert_eq!(new.data.len(), new.lookup.len());
        assert_eq!(new.data.len(), new.length);

        new
    }
}

impl<T: Ord + PartialEq + Clone + std::fmt::Debug + Heapable> MinHeapMap<T> {
    pub fn new() -> Self {
        Self {
            data: vec![],
            lookup: HashMap::new(),
            length: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn insert(&mut self, val: T) {
        self.lookup.insert(val.lookup_key(), self.length + 1);
        self.data.push(val);
        self.heapify_up(self.length);
        self.length += 1;
    }

    pub fn lookup(&self, key: (u32, u32)) -> Option<&T> {
        if let Some(idx) = self.lookup.get(&key) {
            return self.data.get(*idx);
        }
        None
    }

    pub fn lookup_and_mutate(
        &mut self,
        key: (u32, u32),
        f: impl FnOnce(&mut T),
    ) -> Result<(), HeapError> {
        if let Some(idx) = self.lookup.get(&key) {
            if let Some(data) = self.data.get_mut(*idx) {
                f(data);
                self.heapify_up(*idx);
                return Ok(());
            }
        }
        Err(HeapError::LookupReturnedNone)
    }

    pub fn pop(&mut self) -> Result<T, HeapError> {
        if self.length == 0 {
            return Err(HeapError::LengthIsZero);
        }
        let out = self.data.remove(0);
        self.lookup.remove(&out.lookup_key()).unwrap();
        self.length -= 1;

        if self.length == 0 {
            self.data = vec![];
            self.length = 0;
            return Ok(out);
        }
        self.heapify_down(0);
        Ok(out)
    }

    fn swap(&mut self, one: usize, other: usize) {
        let one_val = self.data[one].clone();
        let other_val = self.data[other].clone();

        let one_lookup_key = one_val.lookup_key();
        self.lookup.insert(one_lookup_key, other);
        self.data[other] = one_val;

        let other_lookup_key = other_val.lookup_key();
        self.lookup.insert(other_lookup_key, one);
        self.data[one] = other_val;
    }

    fn heapify_down(&mut self, idx: usize) {
        let (l_index, r_index) = (Self::left_child_idx(idx), Self::right_child_idx(idx));
        if idx >= self.length || l_index >= self.length {
            return;
        }

        let val = &self.data[idx];
        self.lookup.insert(val.lookup_key(), idx);
        let lval = &self.data[l_index];
        self.lookup.insert(lval.lookup_key(), l_index);

        let mut min = val.min(lval);
        if let Some(rval) = self.data.get(r_index) {
            self.lookup.insert(rval.lookup_key(), r_index);
            min = min.min(rval);
        }

        match min {
            _ if min == val => {
                // All is well if parent is min
            }
            _ if min == lval => {
                self.heapify_down(l_index);
                self.swap(idx, l_index);
            }
            _ => {
                // min must be rval
                self.heapify_down(r_index);
                self.swap(idx, r_index);
            }
        }
    }

    fn heapify_up(&mut self, idx: usize) {
        if idx == 0 {
            return;
        }
        let parent_idx = Self::parent_idx(idx);
        let parent_val = self.data[parent_idx].clone();
        self.lookup.insert(parent_val.lookup_key(), parent_idx);
        let val = self.data[idx].clone();
        self.lookup.insert(val.lookup_key(), idx);

        if parent_val > val {
            self.swap(idx, parent_idx);
            self.heapify_up(parent_idx)
        }
    }

    fn parent_idx(idx: usize) -> usize {
        // Numbers that don't evenly go into 2 return the division just without a remainder, not
        // floating point numbers
        (idx - 1) / 2
    }

    fn left_child_idx(idx: usize) -> usize {
        idx * 2 + 1
    }

    fn right_child_idx(idx: usize) -> usize {
        idx * 2 + 2
    }
}

mod tests {
    use std::collections::HashMap;

    use super::{Heapable, MinHeapMap};

    #[derive(Debug, PartialEq, Clone, Eq)]
    struct TestHeapable {
        val: i32,
        x: u32,
        y: u32,
    }

    impl PartialOrd for TestHeapable {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.val.partial_cmp(&other.val)
        }
    }

    impl Ord for TestHeapable {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.val.cmp(&other.val)
        }
    }

    impl Heapable for TestHeapable {
        fn y(&self) -> u32 {
            self.y
        }
        fn x(&self) -> u32 {
            self.x
        }
    }

    pub fn create_heap_map(size: u32, vec: &[i32]) -> MinHeapMap<TestHeapable> {
        assert_eq!(size as usize * size as usize, vec.len());
        let mut vec = vec.to_vec();
        let mut heap_vec = Vec::new();
        for y in 1..=size {
            for x in 1..=size {
                let val = TestHeapable {
                    val: vec[heap_vec.len()],
                    x,
                    y,
                };
                heap_vec.push(val);
            }
        }
        MinHeapMap::from(heap_vec)
    }

    #[test]
    fn swap_works() {
        let input = [0, 32, 65, 16];
        let mut heap_map = create_heap_map(2, &input);

        let key0 = heap_map.lookup.get(&(1, 1)).unwrap().clone();
        let key1 = heap_map.lookup.get(&(1, 2)).unwrap().clone();
        heap_map.swap(key0, key1);

        let mut expected_map = heap_map.lookup.clone();

        expected_map.insert((1, 1), key1);
        expected_map.insert((1, 2), key0);

        assert_eq!(heap_map.lookup, expected_map);
    }

    #[test]
    fn heap_works() {
        let input = [0, 32, 65, 16, 19, 12, 14, 7, 8];
        let mut heap_map = create_heap_map(3, &input);

        assert_eq!(heap_map.pop().unwrap(), TestHeapable { val: 0, x: 1, y: 1 });
        assert_eq!(heap_map.length, 8);
        assert_eq!(heap_map.pop().unwrap(), TestHeapable { val: 7, x: 2, y: 3 });

        heap_map.lookup_and_mutate((2, 2), |v| v.val = 0).unwrap();

        assert_eq!(heap_map.pop().unwrap(), TestHeapable { val: 0, x: 2, y: 2 });

        assert_eq!(
            heap_map.lookup((3, 2)),
            Some(&TestHeapable {
                val: 12,
                x: 3,
                y: 2
            })
        );

        assert_eq!(
            heap_map.lookup((1, 3)),
            Some(&TestHeapable {
                val: 14,
                x: 1,
                y: 3
            })
        );

        assert_eq!(heap_map.pop().unwrap(), TestHeapable { val: 8, x: 3, y: 3 });
        assert_eq!(
            heap_map.pop().unwrap(),
            TestHeapable {
                val: 12,
                x: 3,
                y: 2
            }
        );
        assert_eq!(
            heap_map.pop().unwrap(),
            TestHeapable {
                val: 14,
                x: 1,
                y: 3
            }
        );
        assert_eq!(
            heap_map.pop().unwrap(),
            TestHeapable {
                val: 16,
                x: 1,
                y: 2
            }
        );
        assert_eq!(
            heap_map.pop().unwrap(),
            TestHeapable {
                val: 32,
                x: 2,
                y: 1
            }
        );
        assert_eq!(
            heap_map.pop().unwrap(),
            TestHeapable {
                val: 65,
                x: 3,
                y: 1
            }
        );
    }
}
