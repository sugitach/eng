use ropey::Rope;
use std::ops::Range;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::style::StyleMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    text: Rope,
    version: u64,
    pub name: String,
    pub path: Option<PathBuf>,
    pub modified: bool,
    pub read_only: bool,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    pub styles: StyleMap,
}

impl Buffer {
    pub fn new(name: String, text: &str) -> Self {
        Self {
            text: Rope::from_str(text),
            version: 0,
            name,
            path: None,
            modified: false,
            read_only: false,
            encoding: Encoding::Utf8,
            line_ending: LineEnding::Lf,
            styles: StyleMap::new(),
        }
    }

    pub fn insert(&mut self, char_idx: usize, text: &str) -> Result<(), String> {
        if self.read_only {
            return Err("Buffer is read-only".into());
        }
        let char_len = self.text.len_chars();
        if char_idx > char_len {
             return Err(format!("Index out of bounds: {} > {}", char_idx, char_len));
        }
        let insert_len = text.chars().count();
        self.text.insert(char_idx, text);
        self.styles.on_insert(char_idx, insert_len);
        self.version += 1;
        self.modified = true;
        Ok(())
    }

    pub fn delete(&mut self, range: Range<usize>) -> Result<(), String> {
         if self.read_only {
             return Err("Buffer is read-only".into());
         }
         let char_len = self.text.len_chars();
         if range.end > char_len || range.start > range.end {
             return Err(format!("Invalid range: {:?} (len: {})", range, char_len));
         }
         self.text.remove(range.clone());
         self.styles.on_delete(range);
         self.version += 1;
         self.modified = true;
         Ok(())
    }

    pub fn set_path(&mut self, path: PathBuf) {
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                self.name = name_str.to_string();
            }
        }
        self.path = Some(path);
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

    pub async fn create_buffer(&self, name: String, text: &str) -> String {
        let buffer = Buffer::new(name, text);
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
    use crate::style::Style;

    #[test]
    fn test_buffer_styles_sync() {
        let mut buf = Buffer::new("test".into(), "Hello");
        buf.styles.add_span(0..5, Style { color: 0, bold: true, italic: false });
        
        // 先頭に挿入
        buf.insert(0, ">").unwrap();
        assert_eq!(buf.styles.get_spans()[0].range, 1..6);
    }

    #[test]
    fn test_buffer_properties() {
        let mut buf = Buffer::new("scratch".into(), "Hello");
        assert_eq!(buf.name, "scratch");
        assert_eq!(buf.modified, false);
        assert_eq!(buf.read_only, false);

        buf.insert(5, "!").unwrap();
        assert_eq!(buf.modified, true);
    }

    #[test]
    fn test_buffer_read_only() {
        let mut buf = Buffer::new("readonly".into(), "Hello");
        buf.read_only = true;
        let result = buf.insert(0, "X");
        assert!(result.is_err());
        assert_eq!(buf.to_string(), "Hello");
    }

    #[test]
    fn test_buffer_set_path() {
        let mut buf = Buffer::new("old".into(), "");
        buf.set_path(PathBuf::from("/tmp/test.rs"));
        assert_eq!(buf.name, "test.rs");
        assert_eq!(buf.path, Some(PathBuf::from("/tmp/test.rs")));
    }

    #[test]
    fn test_buffer_insert_delete() {
        let mut buf = Buffer::new("test".into(), "Hello");
        buf.insert(5, " World").unwrap();
        assert_eq!(buf.to_string(), "Hello World");

        buf.delete(0..6).unwrap();
        assert_eq!(buf.to_string(), "World");
    }

    #[test]
    fn test_lines() {
        let buf = Buffer::new("lines".into(), "One\nTwo\nThree");
        assert_eq!(buf.len_lines(), 3);
        assert_eq!(buf.char_to_line(4), 1); 
    }

    #[tokio::test]
    async fn test_editor_state() {
        let state = EditorState::new();
        let id = state.create_buffer("test.txt".into(), "Content").await;
        
        let buf_arc = state.get_buffer(&id).await.expect("Buffer should exist");
        let buf = buf_arc.read().await;
        assert_eq!(buf.name, "test.txt");
        assert_eq!(buf.to_string(), "Content");
    }
}