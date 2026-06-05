# ternary-btree

**B-tree with ternary branching: 3-way comparison (less/equal/greater) at every node.**

Binary search trees have 2 children per node. Ternary B-trees have 3 — left (less than), middle (equal), right (greater than). This means keys with the same value accumulate in the middle child, handling duplicates naturally without extra bookkeeping.

---

## Why Ternary Branching?

**Fewer levels**: Height = ⌈log₃(n)⌉ vs ⌈log₂(n)⌉. For 1 million keys:
- Binary: 20 levels
- Ternary: 13 levels (35% fewer)

**Natural duplicate handling**: Equal keys go to the middle child. No need for secondary indices or count fields.

**3-way comparison**: Rust's `Ord` trait already returns `Ordering::{Less, Equal, Greater}` — a ternary result! Ternary B-trees use this directly.

---

## Architecture

- **`TernaryBTree<K, V>`** — Generic B-tree with ternary branching
  - `insert(key, value)` — Insert with 3-way routing
  - `search(key)` → `Option<&V>` — Lookup by key
  - `remove(key)` → `Option<V>` — Delete with rebalancing
  - `range(min, max)` → Range query iterator
  - `len()`, `is_empty()`, `height()` — Tree statistics

---

## Quick Start

```rust
use ternary_btree::TernaryBTree;

let mut tree = TernaryBTree::new();
tree.insert(5, "five");
tree.insert(3, "three");
tree.insert(7, "seven");
tree.insert(5, "five-dup"); // duplicate → middle child

assert_eq!(tree.search(&5), Some(&"five-dup"));
assert_eq!(tree.search(&3), Some(&"three"));
assert_eq!(tree.height(), 1); // shallow!
```

---

## Ecosystem

- **ternary-heap** — Ternary priority queue (complementary data structure)
- **ternary-sort** — Sorting optimized for ternary data
- **ternary-search** — Search algorithms on ternary structures
- **ternary-index** — Inverted index with ternary routing

## License

MIT
