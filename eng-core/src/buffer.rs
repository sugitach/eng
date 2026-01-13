use ropey::Rope;
use std::ops::Range;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Buffer {
    text: Rope,
    version: u64,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new("")
    }
}

impl Buffer {
    pub fn new(text: &str) -> Self {
        Self {
            text: Rope::from_str(text),
            version: 0,
        }
    }

    pub fn insert(&mut self, char_idx: usize, text: &str) {
        let char_len = self.text.len_chars();
        if char_idx > char_len {
             panic!("Index out of bounds: {} > {}", char_idx, char_len);
        }
        self.text.insert(char_idx, text);
        self.version += 1;
    }

    pub fn delete(&mut self, range: Range<usize>) {
         let char_len = self.text.len_chars();
         if range.end > char_len || range.start > range.end {
             panic!("Invalid range: {:?} (len: {})", range, char_len);
         }
         self.text.remove(range);
         self.version += 1;
    }

    pub fn len_chars(&self) -> usize {
        self.text.len_chars()
    }

    pub fn len_lines(&self) -> usize {
        self.text.len_lines()
    }
    
    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.text.char_to_line(char_idx)
    }

    pub fn line_to_char(&self, line_idx: usize) -> usize {
        self.text.line_to_char(line_idx)
    }

    pub fn to_string(&self) -> String {
        String::from(&self.text)
    }
}

#[derive(Debug, Default)]
pub struct EditorState {
    buffers: RwLock<HashMap<String, Arc<RwLock<Buffer>>>>,
}

impl EditorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_buffer(&self, text: &str) -> String {
        let buffer = Buffer::new(text);
        let id = Uuid::new_v4().to_string();
        
        let mut buffers = self.buffers.write().await;
        buffers.insert(id.clone(), Arc::new(RwLock::new(buffer)));
        
        id
    }

    pub async fn get_buffer(&self, id: &str) -> Option<Arc<RwLock<Buffer>>> {
        let buffers = self.buffers.read().await;
        buffers.get(id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_insert_delete() {
        let mut buf = Buffer::new("Hello");
        buf.insert(5, " World");
        assert_eq!(buf.to_string(), "Hello World");

        buf.delete(0..6);
        assert_eq!(buf.to_string(), "World");
    }

    #[test]
    fn test_buffer_multibyte() {
        let mut buf = Buffer::new("こんにちは");
        assert_eq!(buf.len_chars(), 5);
        
        buf.insert(5, "世界");
        assert_eq!(buf.to_string(), "こんにちは世界");
        
        buf.delete(1..3);
        assert_eq!(buf.to_string(), "こちは世界");
    }

    #[test]
    fn test_lines() {
        let buf = Buffer::new("One\nTwo\nThree");
        assert_eq!(buf.len_lines(), 3);
        assert_eq!(buf.char_to_line(0), 0);
        assert_eq!(buf.char_to_line(4), 1); 
    }

    #[tokio::test]
    async fn test_editor_state() {
        let state = EditorState::new();
        let id = state.create_buffer("Test Content").await;
        
        let buf_arc = state.get_buffer(&id).await.expect("Buffer should exist");
        let buf = buf_arc.read().await;
        assert_eq!(buf.to_string(), "Test Content");
    }
}