use tui::layout::Rect;
pub mod files;
pub mod list;
pub mod project;
pub mod prompt;

pub fn center_rect(width: u16, height: u16, chunk: Rect) -> Rect {
    Rect::new(
        chunk
            .x
            .saturating_add(1)
            .max(chunk.x + chunk.width.saturating_sub(width) / 2),
        chunk
            .y
            .saturating_add(1)
            .max(chunk.y + chunk.height.saturating_sub(height) / 2),
        width.min(chunk.width.saturating_sub(2)),
        height.min(chunk.height.saturating_sub(2)),
    )
}
