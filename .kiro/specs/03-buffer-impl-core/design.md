# 03-buffer-impl-core: Design

## 1. データ構造
`eng-core/src/buffer.rs` モジュールを作成する。

```rust
use ropey::Rope;

#[derive(Debug, Clone)]
pub struct Buffer {
    text: Rope,
    version: u64,
}

impl Buffer {
    pub fn new(text: &str) -> Self { ... }
    pub fn insert(&mut self, char_idx: usize, text: &str) { ... }
    pub fn delete(&mut self, start_char_idx: usize, end_char_idx: usize) { ... }
    pub fn len_chars(&self) -> usize { ... }
    pub fn char_to_line(&self, char_idx: usize) -> usize { ... }
    // その他のRopeyラッパーメソッド
}
```

## 2. EditorState
複数のバッファを管理するためのコンテナ。まずはスレッドセーフな設計を考慮しつつ、基本的なCRUDを用意する。

```rust
use std::collections::HashMap;

pub struct EditorState {
    buffers: HashMap<String, Buffer>, // Key: BufferId (UUID)
}
```
