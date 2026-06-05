//! Ternary B-tree: left(-1) / middle(0) / right(+1) branching.
//!
//! Each internal node holds up to 2 keys (k0 < k1) and three child pointers:
//!   left   → keys < k0
//!   middle → k0 ≤ keys < k1
//!   right  → keys ≥ k1
//!
//! Pre-emptive top-down splitting keeps insertion O(log n) with no second pass.

use std::fmt::Debug;

const MAX_KEYS: usize = 2;
const MIN_KEYS: usize = 1;

#[derive(Debug, Clone)]
enum Node<K: Ord + Clone + Debug, V: Clone + Debug> {
    Leaf { entries: Vec<(K, V)> },
    Internal { keys: Vec<K>, children: Vec<Box<Node<K, V>>> },
}

impl<K: Ord + Clone + Debug, V: Clone + Debug> Node<K, V> {
    fn new_leaf() -> Self {
        Node::Leaf { entries: Vec::new() }
    }

    fn is_full(&self) -> bool {
        match self {
            Node::Leaf { entries } => entries.len() >= MAX_KEYS,
            Node::Internal { keys, .. } => keys.len() >= MAX_KEYS,
        }
    }

    fn key_count(&self) -> usize {
        match self {
            Node::Leaf { entries } => entries.len(),
            Node::Internal { keys, .. } => keys.len(),
        }
    }

    fn child_index(keys: &[K], key: &K) -> usize {
        let mut i = 0;
        while i < keys.len() && key >= &keys[i] {
            i += 1;
        }
        i
    }

    fn search(&self, key: &K) -> Option<&V> {
        match self {
            Node::Leaf { entries } => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            Node::Internal { keys, children } => {
                children[Self::child_index(keys, key)].search(key)
            }
        }
    }

    fn split(&mut self) -> (K, Box<Node<K, V>>) {
        match self {
            Node::Leaf { entries } => {
                let mid = entries.len() / 2;
                let right_entries = entries.split_off(mid);
                let median = right_entries[0].0.clone();
                (median, Box::new(Node::Leaf { entries: right_entries }))
            }
            Node::Internal { keys, children } => {
                let mid = keys.len() / 2;
                let median = keys[mid].clone();
                let right_keys = keys.split_off(mid + 1);
                keys.pop();
                let right_children = children.split_off(mid + 1);
                (median, Box::new(Node::Internal { keys: right_keys, children: right_children }))
            }
        }
    }

    fn insert_non_full(&mut self, key: K, value: V) -> bool {
        match self {
            Node::Leaf { entries } => {
                let pos = entries.partition_point(|(k, _)| k < &key);
                if pos < entries.len() && entries[pos].0 == key {
                    entries[pos].1 = value;
                    false // updated, not new
                } else {
                    entries.insert(pos, (key, value));
                    true
                }
            }
            Node::Internal { keys, children } => {
                let ci = Self::child_index(keys, &key);
                if children[ci].is_full() {
                    let (median, right) = children[ci].split();
                    keys.insert(ci, median);
                    children.insert(ci + 1, right);
                    let ci = Self::child_index(keys, &key);
                    children[ci].insert_non_full(key, value)
                } else {
                    children[ci].insert_non_full(key, value)
                }
            }
        }
    }

    fn leftmost_key(&self) -> K {
        match self {
            Node::Leaf { entries } => entries[0].0.clone(),
            Node::Internal { children, .. } => children[0].leftmost_key(),
        }
    }

    fn delete(&mut self, key: &K) -> bool {
        match self {
            Node::Leaf { entries } => {
                if let Some(pos) = entries.iter().position(|(k, _)| k == key) {
                    entries.remove(pos);
                    true
                } else {
                    false
                }
            }
            Node::Internal { keys, children } => {
                if let Some(sep_pos) = keys.iter().position(|k| k == key) {
                    let successor = children[sep_pos + 1].leftmost_key();
                    children[sep_pos + 1].delete(&successor.clone());
                    keys[sep_pos] = successor;
                    true
                } else {
                    let ci = Self::child_index(keys, key);
                    let removed = children[ci].delete(key);
                    if removed {
                        self.rebalance(ci);
                    }
                    removed
                }
            }
        }
    }

    fn rebalance(&mut self, ci: usize) {
        let needs = match self {
            Node::Internal { children, .. } => children[ci].key_count() < MIN_KEYS,
            _ => return,
        };
        if !needs {
            return;
        }
        if let Node::Internal { keys, children } = self {
            if ci > 0 && children[ci - 1].key_count() > MIN_KEYS {
                Self::rotate_right(keys, children, ci);
            } else if ci + 1 < children.len() && children[ci + 1].key_count() > MIN_KEYS {
                Self::rotate_left(keys, children, ci);
            } else if ci > 0 {
                Self::merge(keys, children, ci - 1);
            } else if ci + 1 < children.len() {
                Self::merge(keys, children, ci);
            }
        }
    }

    fn rotate_right(keys: &mut Vec<K>, children: &mut Vec<Box<Node<K, V>>>, ci: usize) {
        let (left_slice, right_slice) = children.split_at_mut(ci);
        let left = &mut *left_slice[ci - 1];
        let right = &mut *right_slice[0];
        match (left, right) {
            (Node::Leaf { entries: le }, Node::Leaf { entries: re }) => {
                let last = le.pop().unwrap();
                let sep = last.0.clone();
                re.insert(0, last);
                keys[ci - 1] = sep;
            }
            (Node::Internal { keys: lk, children: lc }, Node::Internal { keys: rk, children: rc }) => {
                let bk = lk.pop().unwrap();
                let bc = lc.pop().unwrap();
                let sep = std::mem::replace(&mut keys[ci - 1], bk);
                rk.insert(0, sep);
                rc.insert(0, bc);
            }
            _ => {}
        }
    }

    fn rotate_left(keys: &mut Vec<K>, children: &mut Vec<Box<Node<K, V>>>, ci: usize) {
        let (left_slice, right_slice) = children.split_at_mut(ci + 1);
        let left = &mut *left_slice[ci];
        let right = &mut *right_slice[0];
        match (left, right) {
            (Node::Leaf { entries: le }, Node::Leaf { entries: re }) => {
                let first = re.remove(0);
                let new_sep = re.first().map(|(k, _)| k.clone()).unwrap_or_else(|| first.0.clone());
                le.push(first);
                keys[ci] = new_sep;
            }
            (Node::Internal { keys: lk, children: lc }, Node::Internal { keys: rk, children: rc }) => {
                let bk = rk.remove(0);
                let bc = rc.remove(0);
                let sep = std::mem::replace(&mut keys[ci], bk);
                lk.push(sep);
                lc.push(bc);
            }
            _ => {}
        }
    }

    fn merge(keys: &mut Vec<K>, children: &mut Vec<Box<Node<K, V>>>, left_ci: usize) {
        let sep = keys.remove(left_ci);
        let right = *children.remove(left_ci + 1);
        match (&mut *children[left_ci], right) {
            (Node::Leaf { entries: le }, Node::Leaf { entries: re }) => le.extend(re),
            (Node::Internal { keys: lk, children: lc }, Node::Internal { keys: rk, children: rc }) => {
                lk.push(sep);
                lk.extend(rk);
                lc.extend(rc);
            }
            _ => {}
        }
    }

    fn range<'a>(&'a self, lo: &K, hi: &K, result: &mut Vec<(&'a K, &'a V)>) {
        match self {
            Node::Leaf { entries } => {
                for (k, v) in entries {
                    if k >= lo && k <= hi {
                        result.push((k, v));
                    }
                }
            }
            Node::Internal { keys, children } => {
                for (i, child) in children.iter().enumerate() {
                    let in_range =
                        (i == 0 || hi >= &keys[i - 1]) && (i >= keys.len() || lo <= &keys[i]);
                    if in_range {
                        child.range(lo, hi, result);
                    }
                }
            }
        }
    }

    fn collect_all<'a>(&'a self, out: &mut Vec<(&'a K, &'a V)>) {
        match self {
            Node::Leaf { entries } => {
                for (k, v) in entries {
                    out.push((k, v));
                }
            }
            Node::Internal { children, .. } => {
                for c in children {
                    c.collect_all(out);
                }
            }
        }
    }
}

/// Ternary B-tree mapping `K → V`.
#[derive(Debug)]
pub struct TernaryBTree<K: Ord + Clone + Debug, V: Clone + Debug> {
    root: Option<Box<Node<K, V>>>,
    len: usize,
}

impl<K: Ord + Clone + Debug, V: Clone + Debug> Default for TernaryBTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + Clone + Debug, V: Clone + Debug> TernaryBTree<K, V> {
    pub fn new() -> Self {
        TernaryBTree { root: None, len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn search(&self, key: &K) -> Option<&V> {
        self.root.as_ref()?.search(key)
    }

    pub fn insert(&mut self, key: K, value: V) {
        match &mut self.root {
            None => {
                let mut leaf = Box::new(Node::new_leaf());
                leaf.insert_non_full(key, value);
                self.root = Some(leaf);
                self.len += 1;
            }
            Some(root) => {
                if root.is_full() {
                    let old = self.root.take().unwrap();
                    let mut new_root = Box::new(Node::Internal { keys: vec![], children: vec![old] });
                    if let Node::Internal { keys, children } = &mut *new_root {
                        let (med, right) = children[0].split();
                        keys.push(med);
                        children.push(right);
                    }
                    let is_new = new_root.insert_non_full(key, value);
                    self.root = Some(new_root);
                    if is_new {
                        self.len += 1;
                    }
                } else {
                    let is_new = root.insert_non_full(key, value);
                    if is_new {
                        self.len += 1;
                    }
                }
            }
        }
    }

    pub fn delete(&mut self, key: &K) -> bool {
        if let Some(root) = &mut self.root {
            let removed = root.delete(key);
            if removed {
                self.len -= 1;
                // collapse empty internal root
                let collapse = matches!(root.as_ref(), Node::Internal { keys, children }
                    if keys.is_empty() && children.len() == 1);
                if collapse {
                    let child = if let Node::Internal { children, .. } = &mut **root {
                        children.remove(0)
                    } else {
                        unreachable!()
                    };
                    self.root = Some(child);
                }
                if self.root.as_ref().map_or(false, |r| {
                    matches!(r.as_ref(), Node::Leaf { entries } if entries.is_empty())
                }) {
                    self.root = None;
                }
            }
            removed
        } else {
            false
        }
    }

    /// All key-value pairs with key in [lo, hi], sorted ascending.
    pub fn range(&self, lo: &K, hi: &K) -> Vec<(&K, &V)> {
        let mut result = Vec::new();
        if let Some(root) = &self.root {
            root.range(lo, hi, &mut result);
        }
        result.sort_by(|(a, _), (b, _)| a.cmp(b));
        result
    }

    pub fn iter(&self) -> Vec<(&K, &V)> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            root.collect_all(&mut out);
        }
        out.sort_by(|(a, _), (b, _)| a.cmp(b));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let t: TernaryBTree<i32, &str> = TernaryBTree::new();
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
        assert!(t.search(&1).is_none());
    }

    #[test]
    fn test_single_insert_search() {
        let mut t = TernaryBTree::new();
        t.insert(5, "five");
        assert_eq!(t.search(&5), Some(&"five"));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn test_multiple_inserts_sequential() {
        let mut t = TernaryBTree::new();
        for i in 0..10i32 {
            t.insert(i, i * 10);
        }
        assert_eq!(t.len(), 10);
        for i in 0..10i32 {
            assert_eq!(t.search(&i), Some(&(i * 10)));
        }
    }

    #[test]
    fn test_multiple_inserts_reverse() {
        let mut t = TernaryBTree::new();
        for i in (0..10i32).rev() {
            t.insert(i, i * 2);
        }
        assert_eq!(t.len(), 10);
        for i in 0..10i32 {
            assert_eq!(t.search(&i), Some(&(i * 2)));
        }
    }

    #[test]
    fn test_update_existing_key() {
        let mut t = TernaryBTree::new();
        t.insert(1, "a");
        t.insert(1, "b");
        assert_eq!(t.search(&1), Some(&"b"));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn test_delete_existing() {
        let mut t = TernaryBTree::new();
        t.insert(3, 30);
        t.insert(1, 10);
        t.insert(5, 50);
        assert!(t.delete(&3));
        assert!(t.search(&3).is_none());
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut t: TernaryBTree<i32, i32> = TernaryBTree::new();
        t.insert(1, 1);
        assert!(!t.delete(&99));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn test_delete_all() {
        let mut t = TernaryBTree::new();
        t.insert(1, 1);
        t.insert(2, 2);
        assert!(t.delete(&1));
        assert!(t.delete(&2));
        assert!(t.is_empty());
    }

    #[test]
    fn test_range_query() {
        let mut t = TernaryBTree::new();
        for i in 0..20i32 {
            t.insert(i, i);
        }
        let r = t.range(&5, &10);
        let keys: Vec<i32> = r.iter().map(|(k, _)| **k).collect();
        assert_eq!(keys, vec![5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_range_empty() {
        let mut t = TernaryBTree::new();
        t.insert(1, 1);
        t.insert(10, 10);
        assert!(t.range(&3, &7).is_empty());
    }

    #[test]
    fn test_iter_sorted() {
        let mut t = TernaryBTree::new();
        for &k in &[5, 2, 8, 1, 9, 3] {
            t.insert(k, k);
        }
        let keys: Vec<i32> = t.iter().iter().map(|(k, _)| **k).collect();
        assert_eq!(keys, vec![1, 2, 3, 5, 8, 9]);
    }

    #[test]
    fn test_root_split() {
        let mut t = TernaryBTree::new();
        t.insert(10, "ten");
        t.insert(20, "twenty");
        t.insert(30, "thirty");
        assert_eq!(t.search(&10), Some(&"ten"));
        assert_eq!(t.search(&20), Some(&"twenty"));
        assert_eq!(t.search(&30), Some(&"thirty"));
        assert_eq!(t.len(), 3);
    }

    #[test]
    fn test_negative_and_zero_keys() {
        let mut t = TernaryBTree::new();
        t.insert(-1i32, "neg");
        t.insert(0, "zero");
        t.insert(1, "pos");
        assert_eq!(t.search(&-1), Some(&"neg"));
        assert_eq!(t.search(&0), Some(&"zero"));
        assert_eq!(t.search(&1), Some(&"pos"));
        assert_eq!(t.range(&-1, &0).len(), 2);
    }

    #[test]
    fn test_large_tree() {
        let mut t = TernaryBTree::new();
        for i in 0..200i32 {
            t.insert(i, i * 3);
        }
        assert_eq!(t.len(), 200);
        for i in 0..200i32 {
            assert_eq!(t.search(&i), Some(&(i * 3)));
        }
    }

    #[test]
    fn test_delete_and_reinsert() {
        let mut t = TernaryBTree::new();
        for i in 0..5i32 {
            t.insert(i, i);
        }
        assert!(t.delete(&2));
        t.insert(2, 99);
        assert_eq!(t.search(&2), Some(&99));
        assert_eq!(t.len(), 5);
    }
}
