//! # RustKit Canvas
//!
//! HTML5 Canvas 2D API implementation for the RustKit browser engine.
//!
//! ## Features
//!
//! - **CanvasRenderingContext2D**: Full 2D drawing context
//! - **Path operations**: moveTo, lineTo, arc, bezierCurveTo, quadraticCurveTo
//! - **Drawing**: fillRect, strokeRect, fillText, drawImage
//! - **State management**: save, restore, transform stack
//! - **Styles**: fillStyle, strokeStyle, lineWidth, lineCap, lineJoin
//! - **Image data**: getImageData, putImageData, createImageData
//!
//! ## Architecture
//!
//! ```text
//! Canvas2D
//!    ├── Context State Stack
//!    │      ├── Transform Matrix
//!    │      ├── Fill/Stroke Style
//!    │      └── Clipping Region
//!    ├── Current Path
//!    └── Pixel Buffer (ImageData)
//! ```

use rustkit_css::Color;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

// ==================== Errors ====================

/// Errors that can occur in canvas operations.
#[derive(Error, Debug)]
pub enum CanvasError {
    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(String),

    #[error("Invalid image data")]
    InvalidImageData,
}

// ==================== Identifiers ====================

/// Unique canvas identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanvasId(u64);

impl CanvasId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for CanvasId {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Transform Matrix ====================

/// 2D affine transformation matrix.
/// Represents: [a c e]
///             [b d f]
///             [0 0 1]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    /// Create identity transform.
    pub fn identity() -> Self {
        Self {
            a: 1.0, b: 0.0,
            c: 0.0, d: 1.0,
            e: 0.0, f: 0.0,
        }
    }

    /// Create translation transform.
    pub fn translate(tx: f32, ty: f32) -> Self {
        Self {
            a: 1.0, b: 0.0,
            c: 0.0, d: 1.0,
            e: tx, f: ty,
        }
    }

    /// Create scale transform.
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx, b: 0.0,
            c: 0.0, d: sy,
            e: 0.0, f: 0.0,
        }
    }

    /// Create rotation transform (radians).
    pub fn rotate(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            a: cos, b: sin,
            c: -sin, d: cos,
            e: 0.0, f: 0.0,
        }
    }

    /// Multiply two transforms.
    pub fn multiply(&self, other: &Transform) -> Self {
        Transform {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    /// Transform a point.
    pub fn apply(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }

    /// Get inverse transform.
    pub fn inverse(&self) -> Option<Self> {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() < 1e-10 {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Transform {
            a: self.d * inv_det,
            b: -self.b * inv_det,
            c: -self.c * inv_det,
            d: self.a * inv_det,
            e: (self.c * self.f - self.d * self.e) * inv_det,
            f: (self.b * self.e - self.a * self.f) * inv_det,
        })
    }
}

// ==================== Paint Style ====================

/// Fill or stroke style.
#[derive(Debug, Clone)]
pub enum PaintStyle {
    /// Solid color.
    Color(Color),
    /// Linear gradient.
    LinearGradient(LinearGradient),
    /// Radial gradient.
    RadialGradient(RadialGradient),
    /// Pattern.
    Pattern(Pattern),
}

impl Default for PaintStyle {
    fn default() -> Self {
        PaintStyle::Color(Color::BLACK)
    }
}

impl PaintStyle {
    /// Get solid color if applicable.
    pub fn as_color(&self) -> Option<Color> {
        match self {
            PaintStyle::Color(c) => Some(*c),
            _ => None,
        }
    }

    /// Create from CSS color string.
    pub fn from_color_string(s: &str) -> Self {
        if let Some(color) = parse_color(s) {
            PaintStyle::Color(color)
        } else {
            PaintStyle::Color(Color::BLACK)
        }
    }
}

/// Linear gradient.
#[derive(Debug, Clone)]
pub struct LinearGradient {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub stops: Vec<GradientStop>,
}

impl LinearGradient {
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self {
            x0, y0, x1, y1,
            stops: Vec::new(),
        }
    }

    pub fn add_color_stop(&mut self, offset: f32, color: Color) {
        self.stops.push(GradientStop {
            offset: offset.clamp(0.0, 1.0),
            color,
        });
        self.stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
    }

    /// Get color at position (0.0 to 1.0).
    pub fn color_at(&self, t: f32) -> Color {
        if self.stops.is_empty() {
            return Color::TRANSPARENT;
        }
        if self.stops.len() == 1 || t <= 0.0 {
            return self.stops[0].color;
        }
        if t >= 1.0 {
            return self.stops.last().unwrap().color;
        }

        // Find bracketing stops
        for i in 0..self.stops.len() - 1 {
            if t >= self.stops[i].offset && t <= self.stops[i + 1].offset {
                let range = self.stops[i + 1].offset - self.stops[i].offset;
                let local_t = if range > 0.0 {
                    (t - self.stops[i].offset) / range
                } else {
                    0.0
                };
                return interpolate_color(&self.stops[i].color, &self.stops[i + 1].color, local_t);
            }
        }

        self.stops.last().unwrap().color
    }
}

/// Radial gradient.
#[derive(Debug, Clone)]
pub struct RadialGradient {
    pub x0: f32,
    pub y0: f32,
    pub r0: f32,
    pub x1: f32,
    pub y1: f32,
    pub r1: f32,
    pub stops: Vec<GradientStop>,
}

impl RadialGradient {
    pub fn new(x0: f32, y0: f32, r0: f32, x1: f32, y1: f32, r1: f32) -> Self {
        Self {
            x0, y0, r0, x1, y1, r1,
            stops: Vec::new(),
        }
    }

    pub fn add_color_stop(&mut self, offset: f32, color: Color) {
        self.stops.push(GradientStop {
            offset: offset.clamp(0.0, 1.0),
            color,
        });
        self.stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
    }
}

/// Gradient color stop.
#[derive(Debug, Clone)]
pub struct GradientStop {
    pub offset: f32,
    pub color: Color,
}

/// Pattern fill.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub image_id: u64,
    pub repetition: PatternRepetition,
}

/// Pattern repetition mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PatternRepetition {
    #[default]
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
}

// ==================== Line Style ====================

/// Line cap style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// Line join style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

// ==================== Text Style ====================

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Left,
    Right,
    Center,
}

/// Text baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextBaseline {
    Top,
    Hanging,
    Middle,
    #[default]
    Alphabetic,
    Ideographic,
    Bottom,
}

/// Text direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextDirection {
    #[default]
    Ltr,
    Rtl,
    Inherit,
}

// ==================== Compositing ====================

/// Global composite operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompositeOperation {
    #[default]
    SourceOver,
    SourceIn,
    SourceOut,
    SourceAtop,
    DestinationOver,
    DestinationIn,
    DestinationOut,
    DestinationAtop,
    Lighter,
    Copy,
    Xor,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
}

// ==================== Path ====================

/// Path command.
#[derive(Debug, Clone)]
pub enum PathCommand {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadraticCurveTo(f32, f32, f32, f32),
    BezierCurveTo(f32, f32, f32, f32, f32, f32),
    ArcTo(f32, f32, f32, f32, f32),
    Arc(f32, f32, f32, f32, f32, bool),
    Ellipse(f32, f32, f32, f32, f32, f32, f32, bool),
    Rect(f32, f32, f32, f32),
    ClosePath,
}

/// A 2D path.
#[derive(Debug, Clone, Default)]
pub struct Path2D {
    commands: Vec<PathCommand>,
    start_x: f32,
    start_y: f32,
    current_x: f32,
    current_y: f32,
}

impl Path2D {
    /// Create a new empty path.
    pub fn new() -> Self {
        Self::default()
    }

    /// Move to a point.
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::MoveTo(x, y));
        self.start_x = x;
        self.start_y = y;
        self.current_x = x;
        self.current_y = y;
    }

    /// Draw a line to a point.
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::LineTo(x, y));
        self.current_x = x;
        self.current_y = y;
    }

    /// Draw a quadratic bezier curve.
    pub fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        self.commands.push(PathCommand::QuadraticCurveTo(cpx, cpy, x, y));
        self.current_x = x;
        self.current_y = y;
    }

    /// Draw a cubic bezier curve.
    pub fn bezier_curve_to(&mut self, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32) {
        self.commands.push(PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y));
        self.current_x = x;
        self.current_y = y;
    }

    /// Draw an arc to a point.
    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
        self.commands.push(PathCommand::ArcTo(x1, y1, x2, y2, radius));
        // Update current position (simplified)
        self.current_x = x2;
        self.current_y = y2;
    }

    /// Draw an arc.
    pub fn arc(&mut self, x: f32, y: f32, radius: f32, start_angle: f32, end_angle: f32, counterclockwise: bool) {
        self.commands.push(PathCommand::Arc(x, y, radius, start_angle, end_angle, counterclockwise));
        // Update current position
        self.current_x = x + radius * end_angle.cos();
        self.current_y = y + radius * end_angle.sin();
    }

    /// Draw an ellipse.
    pub fn ellipse(&mut self, x: f32, y: f32, radius_x: f32, radius_y: f32, rotation: f32, start_angle: f32, end_angle: f32, counterclockwise: bool) {
        self.commands.push(PathCommand::Ellipse(x, y, radius_x, radius_y, rotation, start_angle, end_angle, counterclockwise));
    }

    /// Draw a rectangle.
    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.commands.push(PathCommand::Rect(x, y, width, height));
    }

    /// Close the current subpath.
    pub fn close_path(&mut self) {
        self.commands.push(PathCommand::ClosePath);
        self.current_x = self.start_x;
        self.current_y = self.start_y;
    }

    /// Get the commands.
    pub fn commands(&self) -> &[PathCommand] {
        &self.commands
    }

    /// Check if path is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Convert to line segments for rendering.
    pub fn to_segments(&self) -> Vec<Vec<(f32, f32)>> {
        let mut segments = Vec::new();
        let mut current_segment = Vec::new();
        let mut current_x = 0.0_f32;
        let mut current_y = 0.0_f32;
        let mut start_x = 0.0_f32;
        let mut start_y = 0.0_f32;

        for cmd in &self.commands {
            match cmd {
                PathCommand::MoveTo(x, y) => {
                    if !current_segment.is_empty() {
                        segments.push(std::mem::take(&mut current_segment));
                    }
                    current_x = *x;
                    current_y = *y;
                    start_x = *x;
                    start_y = *y;
                    current_segment.push((current_x, current_y));
                }
                PathCommand::LineTo(x, y) => {
                    current_x = *x;
                    current_y = *y;
                    current_segment.push((current_x, current_y));
                }
                PathCommand::QuadraticCurveTo(cpx, cpy, x, y) => {
                    let points = quadratic_bezier_points(
                        (current_x, current_y),
                        (*cpx, *cpy),
                        (*x, *y),
                        20,
                    );
                    current_segment.extend(points);
                    current_x = *x;
                    current_y = *y;
                }
                PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y) => {
                    let points = cubic_bezier_points(
                        (current_x, current_y),
                        (*cp1x, *cp1y),
                        (*cp2x, *cp2y),
                        (*x, *y),
                        20,
                    );
                    current_segment.extend(points);
                    current_x = *x;
                    current_y = *y;
                }
                PathCommand::Arc(cx, cy, r, start, end, ccw) => {
                    let points = arc_points(*cx, *cy, *r, *start, *end, *ccw, 32);
                    if !points.is_empty() {
                        if current_segment.is_empty() {
                            current_segment.push(points[0]);
                        }
                        current_segment.extend(&points[1..]);
                        if let Some(&(x, y)) = points.last() {
                            current_x = x;
                            current_y = y;
                        }
                    }
                }
                PathCommand::Rect(x, y, w, h) => {
                    if !current_segment.is_empty() {
                        segments.push(std::mem::take(&mut current_segment));
                    }
                    current_segment = vec![
                        (*x, *y),
                        (*x + *w, *y),
                        (*x + *w, *y + *h),
                        (*x, *y + *h),
                        (*x, *y),
                    ];
                    segments.push(std::mem::take(&mut current_segment));
                    current_x = *x;
                    current_y = *y;
                    start_x = *x;
                    start_y = *y;
                }
                PathCommand::ClosePath => {
                    if !current_segment.is_empty() {
                        current_segment.push((start_x, start_y));
                        segments.push(std::mem::take(&mut current_segment));
                    }
                    current_x = start_x;
                    current_y = start_y;
                }
                _ => {}
            }
        }

        if !current_segment.is_empty() {
            segments.push(current_segment);
        }

        segments
    }
}

// ==================== Image Data ====================

/// Raw pixel data.
#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA format
}

impl ImageData {
    /// Create new image data with all pixels transparent black.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            data: vec![0; size],
        }
    }

    /// Create from existing data.
    pub fn from_data(width: u32, height: u32, data: Vec<u8>) -> Result<Self, CanvasError> {
        if data.len() != (width * height * 4) as usize {
            return Err(CanvasError::InvalidImageData);
        }
        Ok(Self { width, height, data })
    }

    /// Get pixel at (x, y).
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        Some(Color {
            r: self.data[idx],
            g: self.data[idx + 1],
            b: self.data[idx + 2],
            a: self.data[idx + 3] as f32 / 255.0,
        })
    }

    /// Set pixel at (x, y).
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        self.data[idx] = color.r;
        self.data[idx + 1] = color.g;
        self.data[idx + 2] = color.b;
        self.data[idx + 3] = (color.a * 255.0) as u8;
    }

    /// Fill with a color.
    pub fn fill(&mut self, color: Color) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x, y, color);
            }
        }
    }

    /// Clear to transparent black.
    pub fn clear(&mut self) {
        self.data.fill(0);
    }
}

// ==================== Context State ====================

/// Canvas context state (for save/restore).
#[derive(Debug, Clone)]
pub struct ContextState {
    pub transform: Transform,
    pub fill_style: PaintStyle,
    pub stroke_style: PaintStyle,
    pub line_width: f32,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f32,
    pub line_dash: Vec<f32>,
    pub line_dash_offset: f32,
    pub font: String,
    pub text_align: TextAlign,
    pub text_baseline: TextBaseline,
    pub direction: TextDirection,
    pub global_alpha: f32,
    pub global_composite_operation: CompositeOperation,
    pub shadow_blur: f32,
    pub shadow_color: Color,
    pub shadow_offset_x: f32,
    pub shadow_offset_y: f32,
    pub image_smoothing_enabled: bool,
    pub clipping_region: Option<Path2D>,
}

impl Default for ContextState {
    fn default() -> Self {
        Self {
            transform: Transform::identity(),
            fill_style: PaintStyle::Color(Color::BLACK),
            stroke_style: PaintStyle::Color(Color::BLACK),
            line_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 10.0,
            line_dash: Vec::new(),
            line_dash_offset: 0.0,
            font: "10px sans-serif".to_string(),
            text_align: TextAlign::Start,
            text_baseline: TextBaseline::Alphabetic,
            direction: TextDirection::Ltr,
            global_alpha: 1.0,
            global_composite_operation: CompositeOperation::SourceOver,
            shadow_blur: 0.0,
            shadow_color: Color::TRANSPARENT,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            image_smoothing_enabled: true,
            clipping_region: None,
        }
    }
}

// ==================== Draw Command ====================

/// A canvas drawing command.
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Fill a rectangle.
    FillRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        transform: Transform,
    },
    /// Stroke a rectangle.
    StrokeRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        line_width: f32,
        transform: Transform,
    },
    /// Clear a rectangle.
    ClearRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        transform: Transform,
    },
    /// Fill a path.
    FillPath {
        segments: Vec<Vec<(f32, f32)>>,
        color: Color,
        transform: Transform,
    },
    /// Stroke a path.
    StrokePath {
        segments: Vec<Vec<(f32, f32)>>,
        color: Color,
        line_width: f32,
        line_cap: LineCap,
        line_join: LineJoin,
        transform: Transform,
    },
    /// Fill text.
    FillText {
        text: String,
        x: f32,
        y: f32,
        color: Color,
        font: String,
        transform: Transform,
    },
    /// Stroke text.
    StrokeText {
        text: String,
        x: f32,
        y: f32,
        color: Color,
        line_width: f32,
        font: String,
        transform: Transform,
    },
    /// Draw an image.
    DrawImage {
        image_id: u64,
        sx: f32,
        sy: f32,
        sw: f32,
        sh: f32,
        dx: f32,
        dy: f32,
        dw: f32,
        dh: f32,
        transform: Transform,
    },
    /// Put image data directly.
    PutImageData {
        data: ImageData,
        x: i32,
        y: i32,
    },
}

// ==================== Canvas Context ====================

/// The 2D rendering context.
#[derive(Debug)]
pub struct CanvasRenderingContext2D {
    /// Canvas ID.
    pub id: CanvasId,
    /// Canvas width.
    pub width: u32,
    /// Canvas height.
    pub height: u32,
    /// Current state.
    state: ContextState,
    /// State stack for save/restore.
    state_stack: Vec<ContextState>,
    /// Current path.
    path: Path2D,
    /// Accumulated draw commands.
    commands: Vec<DrawCommand>,
    /// Backing pixel buffer (optional, for getImageData).
    image_data: Option<ImageData>,
}

impl CanvasRenderingContext2D {
    /// Create a new 2D context.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            id: CanvasId::new(),
            width,
            height,
            state: ContextState::default(),
            state_stack: Vec::new(),
            path: Path2D::new(),
            commands: Vec::new(),
            image_data: None,
        }
    }

    /// Resize the canvas.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.image_data = None;
        self.commands.clear();
    }

    // ==================== State ====================

    /// Save the current state.
    pub fn save(&mut self) {
        self.state_stack.push(self.state.clone());
    }

    /// Restore the last saved state.
    pub fn restore(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.state = state;
        }
    }

    // ==================== Transforms ====================

    /// Get the current transform.
    pub fn get_transform(&self) -> Transform {
        self.state.transform
    }

    /// Set the transform.
    pub fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        self.state.transform = Transform { a, b, c, d, e, f };
    }

    /// Reset transform to identity.
    pub fn reset_transform(&mut self) {
        self.state.transform = Transform::identity();
    }

    /// Apply a transform.
    pub fn transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        let m = Transform { a, b, c, d, e, f };
        self.state.transform = self.state.transform.multiply(&m);
    }

    /// Translate.
    pub fn translate(&mut self, x: f32, y: f32) {
        self.state.transform = self.state.transform.multiply(&Transform::translate(x, y));
    }

    /// Rotate.
    pub fn rotate(&mut self, angle: f32) {
        self.state.transform = self.state.transform.multiply(&Transform::rotate(angle));
    }

    /// Scale.
    pub fn scale(&mut self, x: f32, y: f32) {
        self.state.transform = self.state.transform.multiply(&Transform::scale(x, y));
    }

    // ==================== Style Getters/Setters ====================

    /// Set fill style from color.
    pub fn set_fill_style_color(&mut self, color: Color) {
        self.state.fill_style = PaintStyle::Color(color);
    }

    /// Set fill style from string.
    pub fn set_fill_style(&mut self, style: &str) {
        self.state.fill_style = PaintStyle::from_color_string(style);
    }

    /// Set stroke style from color.
    pub fn set_stroke_style_color(&mut self, color: Color) {
        self.state.stroke_style = PaintStyle::Color(color);
    }

    /// Set stroke style from string.
    pub fn set_stroke_style(&mut self, style: &str) {
        self.state.stroke_style = PaintStyle::from_color_string(style);
    }

    /// Set line width.
    pub fn set_line_width(&mut self, width: f32) {
        self.state.line_width = width.max(0.0);
    }

    /// Get line width.
    pub fn line_width(&self) -> f32 {
        self.state.line_width
    }

    /// Set line cap.
    pub fn set_line_cap(&mut self, cap: LineCap) {
        self.state.line_cap = cap;
    }

    /// Set line join.
    pub fn set_line_join(&mut self, join: LineJoin) {
        self.state.line_join = join;
    }

    /// Set miter limit.
    pub fn set_miter_limit(&mut self, limit: f32) {
        self.state.miter_limit = limit.max(0.0);
    }

    /// Set line dash.
    pub fn set_line_dash(&mut self, dash: Vec<f32>) {
        self.state.line_dash = dash;
    }

    /// Get line dash.
    pub fn get_line_dash(&self) -> &[f32] {
        &self.state.line_dash
    }

    /// Set line dash offset.
    pub fn set_line_dash_offset(&mut self, offset: f32) {
        self.state.line_dash_offset = offset;
    }

    /// Set global alpha.
    pub fn set_global_alpha(&mut self, alpha: f32) {
        self.state.global_alpha = alpha.clamp(0.0, 1.0);
    }

    /// Get global alpha.
    pub fn global_alpha(&self) -> f32 {
        self.state.global_alpha
    }

    /// Set font.
    pub fn set_font(&mut self, font: &str) {
        self.state.font = font.to_string();
    }

    /// Set text align.
    pub fn set_text_align(&mut self, align: TextAlign) {
        self.state.text_align = align;
    }

    /// Set text baseline.
    pub fn set_text_baseline(&mut self, baseline: TextBaseline) {
        self.state.text_baseline = baseline;
    }

    /// Set shadow blur.
    pub fn set_shadow_blur(&mut self, blur: f32) {
        self.state.shadow_blur = blur.max(0.0);
    }

    /// Set shadow color.
    pub fn set_shadow_color(&mut self, color: Color) {
        self.state.shadow_color = color;
    }

    /// Set shadow offset.
    pub fn set_shadow_offset(&mut self, x: f32, y: f32) {
        self.state.shadow_offset_x = x;
        self.state.shadow_offset_y = y;
    }

    // ==================== Path Methods ====================

    /// Begin a new path.
    pub fn begin_path(&mut self) {
        self.path = Path2D::new();
    }

    /// Move to a point.
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(x, y);
    }

    /// Line to a point.
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(x, y);
    }

    /// Quadratic curve.
    pub fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        self.path.quadratic_curve_to(cpx, cpy, x, y);
    }

    /// Bezier curve.
    pub fn bezier_curve_to(&mut self, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32) {
        self.path.bezier_curve_to(cp1x, cp1y, cp2x, cp2y, x, y);
    }

    /// Arc to.
    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
        self.path.arc_to(x1, y1, x2, y2, radius);
    }

    /// Arc.
    pub fn arc(&mut self, x: f32, y: f32, radius: f32, start_angle: f32, end_angle: f32, counterclockwise: bool) {
        self.path.arc(x, y, radius, start_angle, end_angle, counterclockwise);
    }

    /// Ellipse.
    pub fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32, rotation: f32, start_angle: f32, end_angle: f32, counterclockwise: bool) {
        self.path.ellipse(x, y, rx, ry, rotation, start_angle, end_angle, counterclockwise);
    }

    /// Add rectangle to path.
    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.path.rect(x, y, width, height);
    }

    /// Close the current subpath.
    pub fn close_path(&mut self) {
        self.path.close_path();
    }

    // ==================== Drawing Methods ====================

    /// Fill the current path.
    pub fn fill(&mut self) {
        if self.path.is_empty() {
            return;
        }

        let color = self.state.fill_style.as_color().unwrap_or(Color::BLACK);
        let color = Color {
            a: color.a * self.state.global_alpha,
            ..color
        };

        self.commands.push(DrawCommand::FillPath {
            segments: self.path.to_segments(),
            color,
            transform: self.state.transform,
        });
    }

    /// Stroke the current path.
    pub fn stroke(&mut self) {
        if self.path.is_empty() {
            return;
        }

        let color = self.state.stroke_style.as_color().unwrap_or(Color::BLACK);
        let color = Color {
            a: color.a * self.state.global_alpha,
            ..color
        };

        self.commands.push(DrawCommand::StrokePath {
            segments: self.path.to_segments(),
            color,
            line_width: self.state.line_width,
            line_cap: self.state.line_cap,
            line_join: self.state.line_join,
            transform: self.state.transform,
        });
    }

    /// Fill a rectangle.
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let color = self.state.fill_style.as_color().unwrap_or(Color::BLACK);
        let color = Color {
            a: color.a * self.state.global_alpha,
            ..color
        };

        self.commands.push(DrawCommand::FillRect {
            x,
            y,
            width,
            height,
            color,
            transform: self.state.transform,
        });
    }

    /// Stroke a rectangle.
    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let color = self.state.stroke_style.as_color().unwrap_or(Color::BLACK);
        let color = Color {
            a: color.a * self.state.global_alpha,
            ..color
        };

        self.commands.push(DrawCommand::StrokeRect {
            x,
            y,
            width,
            height,
            color,
            line_width: self.state.line_width,
            transform: self.state.transform,
        });
    }

    /// Clear a rectangle.
    pub fn clear_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.commands.push(DrawCommand::ClearRect {
            x,
            y,
            width,
            height,
            transform: self.state.transform,
        });
    }

    /// Fill text.
    pub fn fill_text(&mut self, text: &str, x: f32, y: f32) {
        let color = self.state.fill_style.as_color().unwrap_or(Color::BLACK);
        let color = Color {
            a: color.a * self.state.global_alpha,
            ..color
        };

        self.commands.push(DrawCommand::FillText {
            text: text.to_string(),
            x,
            y,
            color,
            font: self.state.font.clone(),
            transform: self.state.transform,
        });
    }

    /// Stroke text.
    pub fn stroke_text(&mut self, text: &str, x: f32, y: f32) {
        let color = self.state.stroke_style.as_color().unwrap_or(Color::BLACK);
        let color = Color {
            a: color.a * self.state.global_alpha,
            ..color
        };

        self.commands.push(DrawCommand::StrokeText {
            text: text.to_string(),
            x,
            y,
            color,
            line_width: self.state.line_width,
            font: self.state.font.clone(),
            transform: self.state.transform,
        });
    }

    /// Draw an image.
    pub fn draw_image(&mut self, image_id: u64, dx: f32, dy: f32) {
        self.commands.push(DrawCommand::DrawImage {
            image_id,
            sx: 0.0,
            sy: 0.0,
            sw: 0.0, // 0 means use full image
            sh: 0.0,
            dx,
            dy,
            dw: 0.0,
            dh: 0.0,
            transform: self.state.transform,
        });
    }

    /// Draw an image with size.
    pub fn draw_image_scaled(&mut self, image_id: u64, dx: f32, dy: f32, dw: f32, dh: f32) {
        self.commands.push(DrawCommand::DrawImage {
            image_id,
            sx: 0.0,
            sy: 0.0,
            sw: 0.0,
            sh: 0.0,
            dx,
            dy,
            dw,
            dh,
            transform: self.state.transform,
        });
    }

    /// Draw an image with source and destination rects.
    pub fn draw_image_full(
        &mut self,
        image_id: u64,
        sx: f32,
        sy: f32,
        sw: f32,
        sh: f32,
        dx: f32,
        dy: f32,
        dw: f32,
        dh: f32,
    ) {
        self.commands.push(DrawCommand::DrawImage {
            image_id,
            sx,
            sy,
            sw,
            sh,
            dx,
            dy,
            dw,
            dh,
            transform: self.state.transform,
        });
    }

    // ==================== Image Data ====================

    /// Create new image data.
    pub fn create_image_data(&self, width: u32, height: u32) -> ImageData {
        ImageData::new(width, height)
    }

    /// Get image data for a region.
    pub fn get_image_data(&self, _x: i32, _y: i32, width: u32, height: u32) -> ImageData {
        // In a real implementation, this would read from the pixel buffer
        // For now, return empty image data
        ImageData::new(width, height)
    }

    /// Put image data.
    pub fn put_image_data(&mut self, data: ImageData, x: i32, y: i32) {
        self.commands.push(DrawCommand::PutImageData { data, x, y });
    }

    // ==================== Gradients ====================

    /// Create a linear gradient.
    pub fn create_linear_gradient(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> LinearGradient {
        LinearGradient::new(x0, y0, x1, y1)
    }

    /// Create a radial gradient.
    pub fn create_radial_gradient(&self, x0: f32, y0: f32, r0: f32, x1: f32, y1: f32, r1: f32) -> RadialGradient {
        RadialGradient::new(x0, y0, r0, x1, y1, r1)
    }

    /// Set fill style to gradient.
    pub fn set_fill_style_gradient(&mut self, gradient: LinearGradient) {
        self.state.fill_style = PaintStyle::LinearGradient(gradient);
    }

    // ==================== Output ====================

    /// Get draw commands and clear them.
    pub fn take_commands(&mut self) -> Vec<DrawCommand> {
        std::mem::take(&mut self.commands)
    }

    /// Get draw commands without clearing.
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Clear all draw commands.
    pub fn clear_commands(&mut self) {
        self.commands.clear();
    }

    // ==================== Hit Testing ====================

    /// Check if a point is in the current path.
    pub fn is_point_in_path(&self, x: f32, y: f32) -> bool {
        // Transform point to path space
        if let Some(inv) = self.state.transform.inverse() {
            let (px, py) = inv.apply(x, y);
            // Use winding number algorithm
            point_in_path(&self.path, px, py)
        } else {
            false
        }
    }

    /// Check if a point is on the current stroke.
    pub fn is_point_in_stroke(&self, x: f32, y: f32) -> bool {
        if let Some(inv) = self.state.transform.inverse() {
            let (px, py) = inv.apply(x, y);
            point_on_stroke(&self.path, px, py, self.state.line_width)
        } else {
            false
        }
    }
}

// ==================== Helper Functions ====================

/// Generate points along a quadratic bezier curve.
fn quadratic_bezier_points(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), segments: usize) -> Vec<(f32, f32)> {
    let mut points = Vec::with_capacity(segments);
    
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let mt = 1.0 - t;

        let x = mt * mt * p0.0 + 2.0 * mt * t * p1.0 + t * t * p2.0;
        let y = mt * mt * p0.1 + 2.0 * mt * t * p1.1 + t * t * p2.1;

        points.push((x, y));
    }

    points
}

/// Generate points along a cubic bezier curve.
fn cubic_bezier_points(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), segments: usize) -> Vec<(f32, f32)> {
    let mut points = Vec::with_capacity(segments);
    
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        let x = mt3 * p0.0 + 3.0 * mt2 * t * p1.0 + 3.0 * mt * t2 * p2.0 + t3 * p3.0;
        let y = mt3 * p0.1 + 3.0 * mt2 * t * p1.1 + 3.0 * mt * t2 * p2.1 + t3 * p3.1;

        points.push((x, y));
    }

    points
}

/// Generate points along an arc.
fn arc_points(cx: f32, cy: f32, r: f32, start: f32, end: f32, ccw: bool, segments: u32) -> Vec<(f32, f32)> {
    let mut points = Vec::new();
    
    let mut angle_diff = end - start;
    if ccw {
        while angle_diff > 0.0 {
            angle_diff -= 2.0 * PI;
        }
    } else {
        while angle_diff < 0.0 {
            angle_diff += 2.0 * PI;
        }
    }

    let num_segments = ((angle_diff.abs() / (2.0 * PI) * segments as f32).ceil() as u32).max(1);
    let step = angle_diff / num_segments as f32;

    for i in 0..=num_segments {
        let angle = start + step * i as f32;
        points.push((cx + r * angle.cos(), cy + r * angle.sin()));
    }

    points
}

/// Parse a color string.
fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().to_lowercase();

    // Hex colors
    if s.starts_with('#') {
        let hex = &s[1..];
        return match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Color::from_rgb(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::from_rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color::new(r, g, b, a as f32 / 255.0))
            }
            _ => None,
        };
    }

    // RGB/RGBA
    if s.starts_with("rgb") {
        let inner = s.trim_start_matches("rgba(")
            .trim_start_matches("rgb(")
            .trim_end_matches(')');
        let parts: Vec<&str> = inner.split(|c| c == ',' || c == '/').collect();
        
        if parts.len() >= 3 {
            let r: u8 = parts[0].trim().parse().ok()?;
            let g: u8 = parts[1].trim().parse().ok()?;
            let b: u8 = parts[2].trim().parse().ok()?;
            let a: f32 = parts.get(3).and_then(|s| s.trim().parse().ok()).unwrap_or(1.0);
            return Some(Color::new(r, g, b, a));
        }
    }

    // Named colors
    match s.as_str() {
        "black" => Some(Color::from_rgb(0, 0, 0)),
        "white" => Some(Color::from_rgb(255, 255, 255)),
        "red" => Some(Color::from_rgb(255, 0, 0)),
        "green" => Some(Color::from_rgb(0, 128, 0)),
        "blue" => Some(Color::from_rgb(0, 0, 255)),
        "yellow" => Some(Color::from_rgb(255, 255, 0)),
        "cyan" => Some(Color::from_rgb(0, 255, 255)),
        "magenta" => Some(Color::from_rgb(255, 0, 255)),
        "gray" | "grey" => Some(Color::from_rgb(128, 128, 128)),
        "transparent" => Some(Color::TRANSPARENT),
        _ => None,
    }
}

/// Interpolate between two colors.
fn interpolate_color(a: &Color, b: &Color, t: f32) -> Color {
    let lerp = |a: u8, b: u8, t: f32| ((a as f32) + ((b as f32) - (a as f32)) * t).round() as u8;
    
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: a.a + (b.a - a.a) * t,
    }
}

/// Check if point is inside path using winding number.
fn point_in_path(path: &Path2D, x: f32, y: f32) -> bool {
    let segments = path.to_segments();
    let mut winding = 0;

    for segment in &segments {
        if segment.len() < 2 {
            continue;
        }

        for i in 0..segment.len() - 1 {
            let (x1, y1) = segment[i];
            let (x2, y2) = segment[i + 1];

            if y1 <= y {
                if y2 > y {
                    let vt = (y - y1) / (y2 - y1);
                    if x < x1 + vt * (x2 - x1) {
                        winding += 1;
                    }
                }
            } else if y2 <= y {
                let vt = (y - y1) / (y2 - y1);
                if x < x1 + vt * (x2 - x1) {
                    winding -= 1;
                }
            }
        }
    }

    winding != 0
}

/// Check if point is on stroke.
fn point_on_stroke(path: &Path2D, x: f32, y: f32, line_width: f32) -> bool {
    let segments = path.to_segments();
    let half_width = line_width / 2.0;

    for segment in &segments {
        if segment.len() < 2 {
            continue;
        }

        for i in 0..segment.len() - 1 {
            let (x1, y1) = segment[i];
            let (x2, y2) = segment[i + 1];

            // Distance from point to line segment
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len_sq = dx * dx + dy * dy;

            if len_sq == 0.0 {
                // Point segment
                let dist = ((x - x1).powi(2) + (y - y1).powi(2)).sqrt();
                if dist <= half_width {
                    return true;
                }
            } else {
                let t = ((x - x1) * dx + (y - y1) * dy) / len_sq;
                let t = t.clamp(0.0, 1.0);
                let px = x1 + t * dx;
                let py = y1 + t * dy;
                let dist = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
                if dist <= half_width {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_identity() {
        let t = Transform::identity();
        let (x, y) = t.apply(10.0, 20.0);
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
    }

    #[test]
    fn test_transform_translate() {
        let t = Transform::translate(5.0, 10.0);
        let (x, y) = t.apply(10.0, 20.0);
        assert_eq!(x, 15.0);
        assert_eq!(y, 30.0);
    }

    #[test]
    fn test_transform_scale() {
        let t = Transform::scale(2.0, 3.0);
        let (x, y) = t.apply(10.0, 20.0);
        assert_eq!(x, 20.0);
        assert_eq!(y, 60.0);
    }

    #[test]
    fn test_transform_inverse() {
        let t = Transform::translate(10.0, 20.0);
        let inv = t.inverse().unwrap();
        let composed = t.multiply(&inv);
        assert!((composed.a - 1.0).abs() < 0.001);
        assert!((composed.d - 1.0).abs() < 0.001);
        assert!((composed.e).abs() < 0.001);
        assert!((composed.f).abs() < 0.001);
    }

    #[test]
    fn test_context_creation() {
        let ctx = CanvasRenderingContext2D::new(800, 600);
        assert_eq!(ctx.width, 800);
        assert_eq!(ctx.height, 600);
    }

    #[test]
    fn test_context_save_restore() {
        let mut ctx = CanvasRenderingContext2D::new(100, 100);
        ctx.set_line_width(5.0);
        ctx.save();
        ctx.set_line_width(10.0);
        assert_eq!(ctx.line_width(), 10.0);
        ctx.restore();
        assert_eq!(ctx.line_width(), 5.0);
    }

    #[test]
    fn test_path_creation() {
        let mut path = Path2D::new();
        path.move_to(0.0, 0.0);
        path.line_to(100.0, 100.0);
        path.close_path();
        assert_eq!(path.commands().len(), 3);
    }

    #[test]
    fn test_path_to_segments() {
        let mut path = Path2D::new();
        path.move_to(0.0, 0.0);
        path.line_to(100.0, 0.0);
        path.line_to(100.0, 100.0);
        path.close_path();
        
        let segments = path.to_segments();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].len(), 4);
    }

    #[test]
    fn test_fill_rect() {
        let mut ctx = CanvasRenderingContext2D::new(100, 100);
        ctx.fill_rect(10.0, 10.0, 50.0, 50.0);
        assert_eq!(ctx.commands().len(), 1);
    }

    #[test]
    fn test_image_data() {
        let mut data = ImageData::new(10, 10);
        data.set_pixel(5, 5, Color::from_rgb(255, 0, 0));
        let pixel = data.get_pixel(5, 5).unwrap();
        assert_eq!(pixel.r, 255);
        assert_eq!(pixel.g, 0);
        assert_eq!(pixel.b, 0);
    }

    #[test]
    fn test_parse_color() {
        assert!(matches!(parse_color("#ff0000"), Some(c) if c.r == 255 && c.g == 0 && c.b == 0));
        assert!(matches!(parse_color("#f00"), Some(c) if c.r == 255 && c.g == 0 && c.b == 0));
        assert!(matches!(parse_color("red"), Some(c) if c.r == 255 && c.g == 0 && c.b == 0));
        assert!(matches!(parse_color("rgb(0, 255, 0)"), Some(c) if c.g == 255));
    }

    #[test]
    fn test_gradient() {
        let mut grad = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        grad.add_color_stop(0.0, Color::from_rgb(255, 0, 0));
        grad.add_color_stop(1.0, Color::from_rgb(0, 0, 255));
        
        let mid = grad.color_at(0.5);
        assert!(mid.r > 100 && mid.r < 150);
        assert!(mid.b > 100 && mid.b < 150);
    }

    #[test]
    fn test_point_in_path() {
        let mut path = Path2D::new();
        path.move_to(0.0, 0.0);
        path.line_to(100.0, 0.0);
        path.line_to(100.0, 100.0);
        path.line_to(0.0, 100.0);
        path.close_path();

        assert!(point_in_path(&path, 50.0, 50.0));
        assert!(!point_in_path(&path, 150.0, 50.0));
    }
}

