use tui::{
    buffer::Buffer,
    layout::Rect,
    text::Spans,
    widgets::{Block, Widget},
};

use crate::ui::styles;

#[derive(Debug, Clone)]
pub struct ListWidget<'a> {
    /// A block to wrap this widget in
    block: Option<Block<'a>>,
    /// Items
    items: Vec<String>,
    /// Item to highlight
    selected: Option<usize>,
    /// Bullet point for items
    bullet: char,
    /// Bullet point for selected item
    bullet_selected: char,
    pub focus: bool,
}

impl<'a> ListWidget<'a> {
    pub fn new(items: Vec<String>, highlighted: Option<usize>) -> ListWidget<'a> {
        ListWidget {
            block: None,
            items,
            selected: highlighted,
            bullet: '•',
            bullet_selected: '►',
            focus: true,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> ListWidget<'a> {
        self.block = Some(block);
        self
    }

    pub fn focus(mut self, focus: bool) -> ListWidget<'a> {
        self.focus = focus;
        self
    }
}

impl<'a> Widget for ListWidget<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        // buf.set_style(area, styles::default);
        let area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if area.height < 1 {
            return;
        }

        let style_normal = match self.focus {
            true => styles::list_text(),
            false => styles::list_text_dim(),
        };
        let style_selected = match self.focus {
            true => styles::list_text_highlight(),
            false => styles::list_text_dim(),
        };

        let x = area.left();
        let mut y = area.top();
        let width = area.width;
        for (i, text) in self.items.iter().enumerate() {
            let mut style = style_normal;
            let mut text = text.clone();
            if self.selected == Some(i) {
                style = style_selected;
                text = format!("{} {}", self.bullet_selected, text);
            } else {
                text = format!("{} {}", self.bullet, text);
            }
            buf.set_spans(x, y, &Spans::from(text), width);
            buf.set_style(Rect::new(x, y, width, 1), style);
            y += 1;
        }
    }
}
