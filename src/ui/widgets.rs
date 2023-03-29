use tui::layout::Rect;
pub mod files;
pub mod list;
pub mod prompt;

pub fn center_rect(width: u16, height: u16, chunk: Rect, margin: u16) -> Rect {
    Rect::new(
        chunk
            .x
            .saturating_add(margin)
            .max(chunk.x + chunk.width.saturating_sub(width) / 2),
        chunk
            .y
            .saturating_add(margin)
            .max(chunk.y + chunk.height.saturating_sub(height) / 2),
        width.min(chunk.width.saturating_sub(margin * 2)),
        height.min(chunk.height.saturating_sub(margin * 2)),
    )
}
