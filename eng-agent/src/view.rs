use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum SplitDirection {
    Horizontal, // 左右に分割
    Vertical,   // 上下に分割
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutSize {
    Fixed(u32), // ピクセル固定
    Stretch,    // 残り全部 (可変)
}

#[derive(Debug, Clone)]
pub enum ViewNode {
    Leaf {
        id: String,
        buffer_id: Option<String>,
    },
    Parent {
        direction: SplitDirection,
        children: Vec<ViewNode>,
        sizes: Vec<LayoutSize>,
    },
}

#[derive(Debug)]
pub struct ViewTree {
    pub root: ViewNode,
    pub active_view_id: Option<String>,
}

impl Default for ViewTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewTree {
    pub fn new() -> Self {
        let initial_view_id = Uuid::new_v4().to_string();
        Self {
            root: ViewNode::Leaf {
                id: initial_view_id.clone(),
                buffer_id: None,
            },
            active_view_id: Some(initial_view_id),
        }
    }

    // 今後の拡張用: View分割、サイズ変更などのメソッドをここに追加
}
