//! # RustKit Layout
//!
//! Layout engine for the RustKit browser engine.
//! Implements block and inline layout algorithms.
//!
//! ## Design Goals
//!
//! 1. **Block layout**: Stack boxes vertically
//! 2. **Inline layout**: Flow text and inline elements horizontally with wrapping
//! 3. **Text shaping**: Use DirectWrite for accurate text measurement
//! 4. **Display list**: Generate paint commands

use rustkit_css::{Color, ComputedStyle, Length};
use thiserror::Error;

/// Errors that can occur in layout.
#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("Layout failed: {0}")]
    LayoutFailed(String),

    #[error("Text shaping error: {0}")]
    TextShapingError(String),
}

/// A 2D rectangle.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn zero() -> Self {
        Self::default()
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }
}

/// Edge sizes (margin, padding, border).
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeSizes {
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// Box dimensions including content, padding, border, and margin.
#[derive(Debug, Clone, Default)]
pub struct Dimensions {
    /// Content area.
    pub content: Rect,
    /// Padding.
    pub padding: EdgeSizes,
    /// Border.
    pub border: EdgeSizes,
    /// Margin.
    pub margin: EdgeSizes,
}

impl Dimensions {
    /// Get the padding box (content + padding).
    pub fn padding_box(&self) -> Rect {
        Rect {
            x: self.content.x - self.padding.left,
            y: self.content.y - self.padding.top,
            width: self.content.width + self.padding.horizontal(),
            height: self.content.height + self.padding.vertical(),
        }
    }

    /// Get the border box (content + padding + border).
    pub fn border_box(&self) -> Rect {
        let pb = self.padding_box();
        Rect {
            x: pb.x - self.border.left,
            y: pb.y - self.border.top,
            width: pb.width + self.border.horizontal(),
            height: pb.height + self.border.vertical(),
        }
    }

    /// Get the margin box (content + padding + border + margin).
    pub fn margin_box(&self) -> Rect {
        let bb = self.border_box();
        Rect {
            x: bb.x - self.margin.left,
            y: bb.y - self.margin.top,
            width: bb.width + self.margin.horizontal(),
            height: bb.height + self.margin.vertical(),
        }
    }
}

/// Type of layout box.
#[derive(Debug, Clone)]
pub enum BoxType {
    /// Block-level box.
    Block,
    /// Inline-level box.
    Inline,
    /// Anonymous block (for grouping inline content).
    AnonymousBlock,
    /// Text run.
    Text(String),
}

/// A layout box in the layout tree.
#[derive(Debug)]
pub struct LayoutBox {
    /// Box type.
    pub box_type: BoxType,
    /// Computed dimensions.
    pub dimensions: Dimensions,
    /// Computed style.
    pub style: ComputedStyle,
    /// Child boxes.
    pub children: Vec<LayoutBox>,
}

impl LayoutBox {
    /// Create a new layout box.
    pub fn new(box_type: BoxType, style: ComputedStyle) -> Self {
        Self {
            box_type,
            dimensions: Dimensions::default(),
            style,
            children: Vec::new(),
        }
    }

    /// Perform layout within the given containing block.
    pub fn layout(&mut self, containing_block: &Dimensions) {
        match self.box_type {
            BoxType::Block | BoxType::AnonymousBlock => {
                self.layout_block(containing_block);
            }
            BoxType::Inline | BoxType::Text(_) => {
                // Inline layout handled by parent
            }
        }
    }

    /// Layout a block-level box.
    fn layout_block(&mut self, containing_block: &Dimensions) {
        // Calculate width first (depends on containing block)
        self.calculate_block_width(containing_block);

        // Position the box
        self.calculate_block_position(containing_block);

        // Layout children
        self.layout_block_children();

        // Height depends on children
        self.calculate_block_height();
    }

    /// Calculate block width.
    fn calculate_block_width(&mut self, containing_block: &Dimensions) {
        let style = &self.style;

        // Get values from style
        let margin_left = self.length_to_px(style.margin_left, containing_block.content.width);
        let margin_right = self.length_to_px(style.margin_right, containing_block.content.width);
        let border_left =
            self.length_to_px(style.border_left_width, containing_block.content.width);
        let border_right =
            self.length_to_px(style.border_right_width, containing_block.content.width);
        let padding_left = self.length_to_px(style.padding_left, containing_block.content.width);
        let padding_right = self.length_to_px(style.padding_right, containing_block.content.width);

        let total_margin_border_padding =
            margin_left + margin_right + border_left + border_right + padding_left + padding_right;

        // Calculate content width
        let content_width = match style.width {
            Length::Auto => {
                // Fill available space
                (containing_block.content.width - total_margin_border_padding).max(0.0)
            }
            _ => self.length_to_px(style.width, containing_block.content.width),
        };

        self.dimensions.content.width = content_width;
        self.dimensions.margin.left = margin_left;
        self.dimensions.margin.right = margin_right;
        self.dimensions.border.left = border_left;
        self.dimensions.border.right = border_right;
        self.dimensions.padding.left = padding_left;
        self.dimensions.padding.right = padding_right;
    }

    /// Calculate block position.
    fn calculate_block_position(&mut self, containing_block: &Dimensions) {
        let style = &self.style;

        self.dimensions.margin.top =
            self.length_to_px(style.margin_top, containing_block.content.width);
        self.dimensions.margin.bottom =
            self.length_to_px(style.margin_bottom, containing_block.content.width);
        self.dimensions.border.top =
            self.length_to_px(style.border_top_width, containing_block.content.width);
        self.dimensions.border.bottom =
            self.length_to_px(style.border_bottom_width, containing_block.content.width);
        self.dimensions.padding.top =
            self.length_to_px(style.padding_top, containing_block.content.width);
        self.dimensions.padding.bottom =
            self.length_to_px(style.padding_bottom, containing_block.content.width);

        // Position below the containing block's content
        self.dimensions.content.x = containing_block.content.x
            + self.dimensions.margin.left
            + self.dimensions.border.left
            + self.dimensions.padding.left;

        self.dimensions.content.y = containing_block.content.y
            + containing_block.content.height
            + self.dimensions.margin.top
            + self.dimensions.border.top
            + self.dimensions.padding.top;
    }

    /// Layout block children.
    fn layout_block_children(&mut self) {
        let mut cursor_y = 0.0;

        for child in &mut self.children {
            // Create a containing block at current cursor position
            let mut cb = self.dimensions.clone();
            cb.content.height = cursor_y;

            child.layout(&cb);

            // Advance cursor by child's margin box height
            cursor_y += child.dimensions.margin_box().height;
        }

        self.dimensions.content.height = cursor_y;
    }

    /// Calculate block height.
    fn calculate_block_height(&mut self) {
        // If height is explicitly set, use it
        if let Length::Px(h) = self.style.height {
            self.dimensions.content.height = h;
        }
        // Otherwise, content.height was set by layout_block_children
    }

    /// Convert a Length to pixels.
    fn length_to_px(&self, length: Length, container_size: f32) -> f32 {
        let font_size = match self.style.font_size {
            Length::Px(px) => px,
            _ => 16.0,
        };
        length.to_px(font_size, 16.0, container_size)
    }
}

/// A paint command for rendering.
#[derive(Debug, Clone)]
pub enum DisplayCommand {
    /// Fill a rectangle with a solid color.
    SolidColor(Color, Rect),
    /// Draw a border.
    Border {
        color: Color,
        rect: Rect,
        top: f32,
        right: f32,
        bottom: f32,
        left: f32,
    },
    /// Draw text.
    Text {
        text: String,
        x: f32,
        y: f32,
        color: Color,
        font_size: f32,
    },
}

/// A display list of paint commands.
#[derive(Debug, Default)]
pub struct DisplayList {
    pub commands: Vec<DisplayCommand>,
}

impl DisplayList {
    /// Create an empty display list.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Build display list from a layout box.
    pub fn build(root: &LayoutBox) -> Self {
        let mut list = DisplayList::new();
        list.render_box(root);
        list
    }

    /// Render a layout box and its children.
    fn render_box(&mut self, layout_box: &LayoutBox) {
        self.render_background(layout_box);
        self.render_borders(layout_box);
        self.render_text(layout_box);

        for child in &layout_box.children {
            self.render_box(child);
        }
    }

    /// Render background.
    fn render_background(&mut self, layout_box: &LayoutBox) {
        let color = layout_box.style.background_color;
        if color.a > 0.0 {
            self.commands.push(DisplayCommand::SolidColor(
                color,
                layout_box.dimensions.border_box(),
            ));
        }
    }

    /// Render borders.
    fn render_borders(&mut self, layout_box: &LayoutBox) {
        let d = &layout_box.dimensions;
        let s = &layout_box.style;

        // Only render if there's a visible border
        if d.border.top > 0.0
            || d.border.right > 0.0
            || d.border.bottom > 0.0
            || d.border.left > 0.0
        {
            self.commands.push(DisplayCommand::Border {
                color: s.border_top_color, // Simplified: use same color for all sides
                rect: d.border_box(),
                top: d.border.top,
                right: d.border.right,
                bottom: d.border.bottom,
                left: d.border.left,
            });
        }
    }

    /// Render text.
    fn render_text(&mut self, layout_box: &LayoutBox) {
        if let BoxType::Text(ref text) = layout_box.box_type {
            let font_size = match layout_box.style.font_size {
                Length::Px(px) => px,
                _ => 16.0,
            };

            self.commands.push(DisplayCommand::Text {
                text: text.clone(),
                x: layout_box.dimensions.content.x,
                y: layout_box.dimensions.content.y,
                color: layout_box.style.color,
                font_size,
            });
        }
    }
}

/// Text metrics from shaping.
#[derive(Debug, Clone)]
pub struct TextMetrics {
    pub width: f32,
    pub height: f32,
    pub ascent: f32,
    pub descent: f32,
}

/// Measure text (simplified - uses average character width approximation).
///
/// In a production engine, this would use DirectWrite or HarfBuzz for accurate shaping.
pub fn measure_text(text: &str, _font_family: &str, font_size: f32) -> TextMetrics {
    // Approximate metrics based on font size
    // Typical Latin font has ~0.5em average character width
    let avg_char_width = font_size * 0.5;
    let width = text.chars().count() as f32 * avg_char_width;

    // Standard line metrics
    let ascent = font_size * 0.8;
    let descent = font_size * 0.2;

    TextMetrics {
        width,
        height: ascent + descent,
        ascent,
        descent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.right(), 110.0);
        assert_eq!(r.bottom(), 70.0);
        assert!(r.contains(50.0, 30.0));
        assert!(!r.contains(0.0, 0.0));
    }

    #[test]
    fn test_dimensions_boxes() {
        let mut d = Dimensions::default();
        d.content = Rect::new(20.0, 20.0, 100.0, 50.0);
        d.padding = EdgeSizes {
            top: 5.0,
            right: 5.0,
            bottom: 5.0,
            left: 5.0,
        };
        d.border = EdgeSizes {
            top: 1.0,
            right: 1.0,
            bottom: 1.0,
            left: 1.0,
        };
        d.margin = EdgeSizes {
            top: 10.0,
            right: 10.0,
            bottom: 10.0,
            left: 10.0,
        };

        let pb = d.padding_box();
        assert_eq!(pb.width, 110.0);
        assert_eq!(pb.height, 60.0);

        let bb = d.border_box();
        assert_eq!(bb.width, 112.0);
        assert_eq!(bb.height, 62.0);

        let mb = d.margin_box();
        assert_eq!(mb.width, 132.0);
        assert_eq!(mb.height, 82.0);
    }

    #[test]
    fn test_layout_box_creation() {
        let style = ComputedStyle::new();
        let layout_box = LayoutBox::new(BoxType::Block, style);
        assert!(matches!(layout_box.box_type, BoxType::Block));
    }

    #[test]
    fn test_display_list_build() {
        let mut style = ComputedStyle::new();
        style.background_color = Color::from_rgb(255, 255, 255);

        let layout_box = LayoutBox::new(BoxType::Block, style);
        let display_list = DisplayList::build(&layout_box);

        assert!(!display_list.commands.is_empty());
    }
}
