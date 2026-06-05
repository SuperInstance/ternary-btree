# From Binary to Ternary: B-Tree

## The Trap

Binary trees — whether plain BSTs, red-black trees, or B-trees of order 2 — branch exactly two ways per node. This is the default choice because it's what we learned first, not because it's optimal. Every binary tree node stores one key and has two children. To go deeper, you visit more nodes, pay more pointer-chasing latency, and miss more cache lines.

The hidden cost: a binary B-tree with *n* keys has height O(log₂ n). For a 2-3-4 tree (order 4), height is O(log₄ n). But 2-3-4 trees are complex — they store 1-3 keys and 2-4 children. The 3-way B-tree (order 3) is the sweet spot: it stores just 1-2 keys with 2-3 children, and the pre-emptive splitting logic is simpler than any binary self-balancing scheme.

## Map to Three States

| Domain | −1 / left | 0 / middle | +1 / right |
|--------|-----------|------------|------------|
| Key comparison | `k < key0` | `key0 ≤ k < key1` | `k ≥ key1` |
| Child pointer | left child | middle child | right child |
| Node occupancy | min (1 key) | normal (1-2 keys) | full (split) |

## From Binary to Ternary

**Before: binary B-tree node**

```rust
struct Node<K, V> {
    key: K,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
}
```

One key, two pointers. Every search descends at a binary fork. With 1 million keys, you traverse ~20 levels.

**After: ternary B-tree node**

```rust
struct Node<K, V> {
    keys: Vec<K>,           // 1 or 2 keys
    children: Vec<Box<Node<K, V>>>,  // 2 or 3 children
    values: Vec<V>,         // parallel to keys
}
```

Two keys, up to three children. With 1 million keys, you traverse at most ~13 levels. That's 35% fewer pointer dereferences and cache misses.

**Before: insertion requires backtracking**

```rust
// Standard B-tree insert: go down, find the leaf,
// insert, then climb back up splitting overflowed nodes
```

**After: pre-emptive splitting**

```rust
// Ternary B-tree insert: on the way down,
// split any full (2-key) node BEFORE recursing
// Result: one root-to-leaf pass, no backtracking
```

This is the ternary advantage in action: because each node has at most 2 keys and 3 children, the split logic is trivial. A full node splits its middle key up and distributes the other two keys to new children. No rotation, no recoloring, no complex rebalancing.

**0 is not nothing:** The middle child is not a "halfway" or "maybe" pointer — it's a genuine third partition. When you search for a key that falls between `key0` and `key1`, the middle child is the *only* correct branch. The neutral space between the two keys is not an error condition; it's where the tree's power lives.

## Why It Matters

Ternary B-trees are 37% shorter than binary B-trees for the same data. Pre-emptive splitting eliminates backtracking. Each node stores its keys densely with no wasted slots. The 3-way branching factor is the simplest nontrivial B-tree — easier to implement than red-black trees, more efficient than binary BSTs, and provably optimal for its class.
