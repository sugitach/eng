use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub color: u32,      // 0xRRGGBB
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone)]
pub struct StyleSpan {
    pub range: Range<usize>,
    pub style: Style,
}

#[derive(Debug, Default, Clone)]
pub struct StyleMap {
    spans: Vec<StyleSpan>,
}

impl StyleMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// テキスト挿入に合わせてスパンをずらす
    pub fn on_insert(&mut self, char_idx: usize, len: usize) {
        for span in &mut self.spans {
            if span.range.start >= char_idx {
                span.range.start += len;
                span.range.end += len;
            } else if span.range.end > char_idx {
                // 挿入位置がスパンの途中だった場合
                span.range.end += len;
            }
        }
    }

    /// テキスト削除に合わせてスパンをずらす/削除する
    pub fn on_delete(&mut self, range: Range<usize>) {
        let len = range.end - range.start;
        self.spans.retain_mut(|span| {
            if span.range.start >= range.end {
                // 削除範囲より後ろにある場合
                span.range.start -= len;
                span.range.end -= len;
                true
            } else if span.range.end <= range.start {
                // 削除範囲より前にある場合
                true
            } else {
                // 削除範囲と重なる場合
                if span.range.start >= range.start && span.range.end <= range.end {
                    false // 完全に削除される
                } else {
                    // 部分的に重なる場合
                    if span.range.start < range.start {
                        // スパンの後半が削られる
                        span.range.end = range.start + (span.range.end.saturating_sub(range.end));
                    } else {
                        // スパンの前半が削られる
                        span.range.start = range.start;
                        span.range.end = span.range.end.saturating_sub(len);
                    }
                    span.range.start < span.range.end
                }
            }
        });
    }

    pub fn add_span(&mut self, range: Range<usize>, style: Style) {
        self.spans.push(StyleSpan { range, style });
    }

    pub fn get_spans(&self) -> &[StyleSpan] {
        &self.spans
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_map_insert() {
        let mut map = StyleMap::new();
        let style = Style { color: 0xFF0000, bold: true, italic: false };
        map.add_span(5..10, style);

        // 0文字目に1文字挿入
        map.on_insert(0, 1);
        assert_eq!(map.get_spans()[0].range, 6..11);

        // 7文字目（スパン内）に2文字挿入
        map.on_insert(7, 2);
        assert_eq!(map.get_spans()[0].range, 6..13);
    }

    #[test]
    fn test_style_map_delete() {
        let mut map = StyleMap::new();
        let style = Style { color: 0x00FF00, bold: false, italic: true };
        map.add_span(10..20, style);

        // スパンより前を削除
        map.on_delete(0..5);
        assert_eq!(map.get_spans()[0].range, 5..15);

        // スパンの半分を削除
        map.on_delete(10..15);
        assert_eq!(map.get_spans()[0].range, 5..10);
    }
}
