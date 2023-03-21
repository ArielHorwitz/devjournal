use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Spans,
    widgets::{Block, Widget},
};

#[derive(Debug, Clone)]
pub struct ListWidget<'a> {
    /// A block to wrap this widget in
    block: Option<Block<'a>>,
    /// Items
    items: Vec<String>,
    /// Item to highlight
    selected: Option<usize>,
    /// Style to for item text
    style: Style,
    /// Style for selected item text
    style_selected: Style,
    /// Bullet point for items
    bullet: char,
    /// Bullet point for selected item
    bullet_selected: char,
}

#[allow(dead_code)]
impl<'a> ListWidget<'a> {
    pub fn new(items: Vec<String>, highlighted: Option<usize>) -> ListWidget<'a> {
        ListWidget {
            block: None,
            style: Default::default(),
            items,
            selected: highlighted,
            style_selected: Default::default(),
            bullet: '•',
            bullet_selected: '►',
        }
    }

    pub fn block(mut self, block: Block<'a>) -> ListWidget<'a> {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> ListWidget<'a> {
        self.style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> ListWidget<'a> {
        self.style_selected = style;
        self
    }

    pub fn bullet(mut self, bullet: char) -> ListWidget<'a> {
        self.bullet = bullet;
        self
    }

    pub fn bullet_selected(mut self, bullet: char) -> ListWidget<'a> {
        self.bullet_selected = bullet;
        self
    }
}

impl<'a> Widget for ListWidget<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
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

        let x = area.left();
        let mut y = area.top();
        let width = area.width;
        for (i, text) in self.items.iter().enumerate() {
            let mut style = self.style;
            let mut text = text.clone();
            if self.selected == Some(i) {
                style = self.style_selected;
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
