# ternary-btree

A pedagogical implementation of the **ternary B-tree** — a balanced search tree where every internal node stores up to two keys and branches three ways.

---

## What Problem Does This Solve?

Binary search trees branch twice per node. A ternary B-tree branches **three** times, letting us pack more keys into each node and reducing tree height.

### Why Ternary?

In a B-tree of order *m*, each node holds at most *m* − 1 keys and has *m* children. Here *m* = 3, so:
- **Maximum keys per node:** 2
- **Maximum children per node:** 3

**Height bound:** A ternary B-tree storing *n* keys has height

$$h \leq \log_3(n + 1)$$

Compare this to a binary search tree or binary B-tree (2-3 tree) with height

$$h \leq \log_2(n + 1)$$

Because $\log_3 n = \frac{\log_2 n}{\log_2 3} \approx 0.63 \cdot \log_2 n$, the ternary tree is roughly **37 % shorter** in the worst case. Fewer levels means fewer cache misses and fewer pointer traversals during search.

### Pre-emptive Splitting

Insertion uses **top-down pre-emptive splitting**: any full node encountered on the descent is split before recursion continues. This guarantees:
- **No backtracking** — a single root-to-leaf pass
- **O(log₃ n)** time per insertion
- **No recursive rebalancing** on the way back up

Deletion uses **rotation** (borrowing from siblings) or **merging** when a node underflows below the minimum key count (1), keeping the tree balanced.

---

## Mathematical Complexity Analysis

| Operation | Binary BST (worst) | Binary B-tree (2-3) | **Ternary B-tree** |
|-----------|-------------------|---------------------|-------------------|
| Search    | O(n)              | O(log₂ n)           | **O(log₃ n)**     |
| Insert    | O(n)              | O(log₂ n)           | **O(log₃ n)**     |
| Delete    | O(n)              | O(log₂ n)           | **O(log₃ n)**     |
| Range     | O(n)              | O(log₂ n + k)       | **O(log₃ n + k)** |

**Space:** O(n) — each key-value pair is stored exactly once.

### Proving O(log₃ n)

At height *h*, a ternary B-tree with minimum occupancy (every non-root node has at least 1 key, at least 2 children) contains at least:

$$N(h) \geq 1 + 2 + 2 \cdot 3 + 2 \cdot 3^2 + \dots + 2 \cdot 3^{h-1} = 3^h$$

Taking logarithms:

$$h \leq \log_3 N$$

Each operation walks a single root-to-leaf path, so the running time is bounded by the height: **Θ(log₃ n)**.

---

## Architecture

### Node Layout

```text
         Internal Node
   ┌─────────────────────┐
   │   k0      │   k1    │   (keys, sorted)
   ├───────────┼─────────┤
   │ left │ middle │ right │  (3 child pointers)
   └──────┴────────┴───────┘

   left   → keys < k0
   middle → k0 ≤ keys < k1
   right  → keys ≥ k1
```

### Tree Example

```text
                    [50, 80]
                   /    |    \
                 /      |      \
           [20,30]   [60,70]   [90]
           / | \      / | \     / | \
         10 25 40   55 65 75  85 95 100
```

### Invariants

1. Every node has **1 or 2 keys** (root may have 1).
2. Every internal node has **exactly `keys.len() + 1` children**.
3. All keys in a child subtree satisfy the ternary partition relative to the parent keys.
4. All leaf nodes reside at the **same depth** (perfect balance).

---

## Getting Started

```rust
use ternary_btree::TernaryBTree;

fn main() {
    let mut tree = TernaryBTree::new();

    // Insert some key-value pairs
    tree.insert("ferris", 42);
    tree.insert("crab", 7);
    tree.insert("rust", 2021);

    // Search
    assert_eq!(tree.search(&"ferris"), Some(&42));
    assert_eq!(tree.search(&"missing"), None);

    // Range query
    let r = tree.range(&"crab", &"rust");
    println!("{:?}", r); // [("crab", 7), ("ferris", 42), ("rust", 2021)]

    // Iterate in sorted order
    for (k, v) in tree.iter() {
        println!("{} → {}", k, v);
    }

    // Delete
    assert!(tree.delete(&"crab"));
    assert!(tree.search(&"crab").is_none());
}
```

---

## Running the Tests

Run the full suite with:

```bash
cargo test
```

There are **15 tests**, each verifying a critical invariant:

| Test | What It Verifies |
|------|-----------------|
| `test_empty_tree` | An empty tree has length 0 and returns `None` on search. |
| `test_single_insert_search` | One insertion can be found immediately. |
| `test_multiple_inserts_sequential` | Sequential keys (0..10) are all retrievable. |
| `test_multiple_inserts_reverse` | Reverse-order insertion still produces a valid tree. |
| `test_update_existing_key` | Re-inserting the same key overwrites the value without growing `len`. |
| `test_delete_existing` | Removing a key makes it unreachable and decrements `len`. |
| `test_delete_nonexistent` | Deleting a missing key returns `false` and leaves the tree untouched. |
| `test_delete_all` | Removing every entry yields an empty tree. |
| `test_range_query` | `range(5, 10)` returns exactly the keys 5 through 10 inclusive. |
| `test_range_empty` | A range with no matching keys returns an empty vector. |
| `test_iter_sorted` | `iter()` always yields keys in strictly ascending order. |
| `test_root_split` | Inserting a third key triggers the first root split, proving pre-emptive splitting works. |
| `test_negative_and_zero_keys` | Signed integers (including −1 and 0) are handled correctly. |
| `test_large_tree` | 200 insertions and lookups exercise deep tree structures. |
| `test_delete_and_reinsert` | Deleting a key and re-inserting it restores the exact same mapping. |

---

## Related Crates

Explore the broader ternary ecosystem on crates.io:

- [`ternary-tree`](https://crates.io/crates/ternary-tree) — General-purpose ternary tree structures.
- [`ternary-compression`](https://crates.io/crates/ternary-compression) — Data compression using ternary alphabets.
- [`ternary-memory`](https://crates.io/crates/ternary-memory) — Ternary-addressable memory abstractions.
- [`ternary-tensor`](https://crates.io/crates/ternary-tensor) — Ternary-valued tensors for machine learning.

---

## License

MIT
