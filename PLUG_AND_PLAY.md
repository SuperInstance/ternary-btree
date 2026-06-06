# PLUG_AND_PLAY — Btree

> B-tree with ternary branching factor for balanced storage

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ternary-btree = { git = "https://github.com/SuperInstance/ternary-btree" }
```

Use in your code:

```rust
use ternary_btree::TernaryBTree;

let mut tree = TernaryBTree::new();
tree.insert("key", 42);
assert_eq!(tree.get("key"), Some(&42));
```

## 📚 Available Documentation

| Document | Description |
|----------|-------------|
| `docs/FROM_BINARY.md` | Understanding ternary concepts as a binary programmer |
| `docs/MIGRATION.md` | Version migration guide |
| `docs/FUTURE-INTEGRATION.md` | Planned features and roadmap |

## 🔗 Integration

This crate is part of the [SuperInstance ternary fleet](https://github.com/SuperInstance). It uses the canonical `Ternary` type from `ternary-types` for cross-crate compatibility.

## 📄 License

MIT
