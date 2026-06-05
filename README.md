# ternary-btree

B-tree with ternary (left/middle/right) branching over keys ordered as {-1, 0, +1}.

Each internal node holds up to 2 keys and 3 child pointers using pre-emptive top-down splitting.

## Features
- `TernaryBTree<K, V>` — generic map
- `insert`, `search`, `delete` with rebalancing (borrow/merge)
- `range(lo, hi)` — sorted range query
- `iter()` — full sorted traversal
- 15 tests

## Usage
```rust
let mut tree = TernaryBTree::new();
tree.insert(-1i32, "neg");
tree.insert(0, "zero");
tree.insert(1, "pos");
assert_eq!(tree.search(&0), Some(&"zero"));
let range = tree.range(&-1, &0);
```
