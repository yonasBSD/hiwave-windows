# RustKit Phases 21-30: Advanced Features Plan

**Created:** January 2, 2026
**Status:** Planning
**Prerequisites:** Phases 14-20 must be complete

---

## Overview

Phases 21-30 transform RustKit from a basic HTML/CSS renderer into a modern web platform capable of running complex web applications. These phases add:

- Advanced CSS layout (Grid)
- Visual effects (Animations, SVG)
- Graphics APIs (Canvas 2D, WebGL)
- Media playback (Audio/Video)
- Offline capabilities (Service Workers, IndexedDB)
- Real-time communication (WebRTC)
- Inclusive design (Accessibility)

**Total Estimated Effort:** 12-18 months with a small team

---

## Phase Dependencies Graph

```
Phase 14 (Events) ──┬──→ Phase 21 (Grid)
                    ├──→ Phase 22 (Animations) ──→ Phase 23 (SVG)
                    └──→ Phase 24 (Canvas 2D) ──→ Phase 26 (WebGL)

Phase 15 (Forms) ───→ Phase 30 (Accessibility)

Phase 16 (Images) ──→ Phase 25 (Audio/Video)

Phase 19 (Navigation) ──→ Phase 27 (Service Workers)

Phase 20 (Security) ──┬──→ Phase 27 (Service Workers)
                      └──→ Phase 29 (WebRTC)

New: Phase 28 (IndexedDB) ←── Phase 27 (Service Workers)
```

---

## Phase 21: CSS Grid Layout

### Overview
Implement CSS Grid, the most powerful layout system in CSS. Grid enables two-dimensional layouts with precise control over rows, columns, and item placement.

### Priority: High
### Estimated Duration: 4-6 weeks
### Dependencies: Phase 12 (Box Model), Phase 17 (Flexbox)

### Sub-Tasks

#### 21.1 Grid Container Properties
- [ ] `display: grid` and `display: inline-grid`
- [ ] `grid-template-columns` and `grid-template-rows`
  - Track sizing: `fr`, `auto`, `min-content`, `max-content`
  - `repeat()` function with `auto-fill` and `auto-fit`
  - `minmax()` function
- [ ] `grid-template-areas` with named areas
- [ ] `grid-auto-columns` and `grid-auto-rows`
- [ ] `grid-auto-flow` (row, column, dense)
- [ ] `gap`, `row-gap`, `column-gap`

#### 21.2 Grid Item Properties
- [ ] `grid-column-start`, `grid-column-end`, `grid-column`
- [ ] `grid-row-start`, `grid-row-end`, `grid-row`
- [ ] `grid-area` (shorthand and named area placement)
- [ ] Span syntax (`span 2`, `span name`)
- [ ] Negative line numbers
- [ ] `order` property integration

#### 21.3 Alignment
- [ ] `justify-items`, `align-items` (container)
- [ ] `justify-content`, `align-content` (container)
- [ ] `justify-self`, `align-self` (items)
- [ ] `place-items`, `place-content`, `place-self` (shorthands)

#### 21.4 Grid Layout Algorithm
- [ ] Track sizing algorithm (complex!)
  - Intrinsic sizing
  - Flexible track sizing with `fr` units
  - Growth limits and base sizes
- [ ] Item placement algorithm
  - Explicit placement
  - Auto-placement with packing modes
- [ ] Subgrid support (optional, CSS Grid Level 2)

#### 21.5 Integration
- [ ] Grid formatting context
- [ ] Interaction with floats (grid items can't float)
- [ ] Interaction with positioned elements
- [ ] Percentage resolution in grid tracks

### Third-Party Libraries
- None required - must be custom implementation
- Reference: [CSS Grid Specification](https://www.w3.org/TR/css-grid-1/)

### Custom Implementation Required
```rust
// New types needed in rustkit-layout
pub struct GridContainer {
    template_columns: Vec<TrackSize>,
    template_rows: Vec<TrackSize>,
    template_areas: Option<GridTemplateAreas>,
    auto_columns: TrackSize,
    auto_rows: TrackSize,
    auto_flow: GridAutoFlow,
    gap: (Length, Length),
}

pub enum TrackSize {
    Length(Length),
    Fr(f32),
    MinMax(Box<TrackSize>, Box<TrackSize>),
    MinContent,
    MaxContent,
    Auto,
    FitContent(Length),
}

pub struct GridItem {
    column: GridLine,
    row: GridLine,
    justify_self: AlignSelf,
    align_self: AlignSelf,
}
```

### Acceptance Criteria
- [ ] Pass 60% of WPT `css/css-grid/` tests
- [ ] 12-column responsive grid layouts work
- [ ] Named grid areas place correctly
- [ ] `fr` units distribute space correctly
- [ ] Auto-placement fills grid correctly
- [ ] Nested grids work

### Risk Assessment
- **High complexity**: Grid layout algorithm is one of the most complex in CSS
- **Performance**: Track sizing can be expensive for large grids
- **Testing burden**: Thousands of edge cases

---

## Phase 22: CSS Animations & Transitions

### Overview
Add motion to the web with CSS transitions (simple property interpolation) and CSS animations (keyframe-based sequences).

### Priority: High
### Estimated Duration: 3-4 weeks
### Dependencies: Phase 14 (Events), Phase 12 (Box Model)

### Sub-Tasks

#### 22.1 CSS Transitions
- [ ] `transition-property` (specific properties or `all`)
- [ ] `transition-duration`
- [ ] `transition-timing-function`
  - Keywords: `ease`, `linear`, `ease-in`, `ease-out`, `ease-in-out`
  - `cubic-bezier()` function
  - `steps()` function
- [ ] `transition-delay`
- [ ] `transition` shorthand
- [ ] Transition events: `transitionstart`, `transitionend`, `transitioncancel`

#### 22.2 Animatable Properties
- [ ] Define which properties can animate
- [ ] Interpolation for each type:
  - Numbers and lengths
  - Colors (RGB interpolation)
  - Transforms (matrix decomposition)
  - Visibility (discrete)
  - Shadows
- [ ] Property-specific behaviors

#### 22.3 CSS Animations
- [ ] `@keyframes` rule parsing
- [ ] `animation-name`
- [ ] `animation-duration`
- [ ] `animation-timing-function` (per-keyframe and overall)
- [ ] `animation-delay`
- [ ] `animation-iteration-count` (number or `infinite`)
- [ ] `animation-direction` (`normal`, `reverse`, `alternate`, `alternate-reverse`)
- [ ] `animation-fill-mode` (`none`, `forwards`, `backwards`, `both`)
- [ ] `animation-play-state` (`running`, `paused`)
- [ ] `animation` shorthand
- [ ] Animation events: `animationstart`, `animationend`, `animationiteration`

#### 22.4 Animation Engine
- [ ] Animation timeline management
- [ ] Frame scheduling with `requestAnimationFrame`
- [ ] Compositor-driven animations (transform, opacity)
- [ ] Main-thread animations (layout-triggering properties)
- [ ] Animation compositor thread (optional optimization)

#### 22.5 Web Animations API
- [ ] `Element.animate()` method
- [ ] `Animation` object
  - `play()`, `pause()`, `cancel()`, `finish()`
  - `currentTime`, `playbackRate`
  - `playState`, `pending`
- [ ] `KeyframeEffect`
- [ ] `AnimationTimeline` and `DocumentTimeline`

### Third-Party Libraries
- Consider: `interpolation` crate for easing functions
- Consider: `euclid` for matrix math (transforms)

### Custom Implementation Required
```rust
// New module: rustkit-animation
pub struct Animation {
    id: AnimationId,
    target: NodeId,
    keyframes: Vec<Keyframe>,
    timing: AnimationTiming,
    state: AnimationPlayState,
    current_time: f64,
    start_time: Option<f64>,
}

pub struct Keyframe {
    offset: f32,  // 0.0 to 1.0
    properties: HashMap<PropertyId, PropertyValue>,
    easing: TimingFunction,
}

pub struct AnimationTiming {
    duration: f64,
    delay: f64,
    iterations: f64,
    direction: AnimationDirection,
    fill_mode: FillMode,
    easing: TimingFunction,
}

pub enum TimingFunction {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f64, f64, f64, f64),
    Steps(u32, StepPosition),
}
```

### Acceptance Criteria
- [ ] Hover transitions work smoothly
- [ ] CSS `@keyframes` animations play
- [ ] Animation events fire correctly
- [ ] `requestAnimationFrame` runs at 60fps
- [ ] Pausing/resuming animations works
- [ ] Pass 50% of WPT `css/css-animations/` tests
- [ ] Pass 50% of WPT `css/css-transitions/` tests

### Risk Assessment
- **Frame timing**: Must integrate with compositor for smooth 60fps
- **Property interpolation**: Each property type needs custom interpolation logic
- **Memory**: Long-running animations must not leak

---

## Phase 23: SVG Support

### Overview
Implement Scalable Vector Graphics rendering for icons, logos, charts, and illustrations. SVG is essential for modern web design.

### Priority: Medium-High
### Estimated Duration: 5-7 weeks
### Dependencies: Phase 22 (Animations), Phase 16 (Images)

### Sub-Tasks

#### 23.1 SVG DOM
- [ ] `<svg>` root element with viewBox, preserveAspectRatio
- [ ] Coordinate system and transformations
- [ ] SVG DOM interface (SVGElement, SVGGraphicsElement, etc.)

#### 23.2 Basic Shapes
- [ ] `<rect>` - rectangles with optional rounded corners
- [ ] `<circle>` - circles
- [ ] `<ellipse>` - ellipses
- [ ] `<line>` - lines
- [ ] `<polyline>` - connected line segments
- [ ] `<polygon>` - closed shapes

#### 23.3 Paths
- [ ] `<path>` element with `d` attribute
- [ ] Path commands:
  - M/m (moveto)
  - L/l (lineto)
  - H/h, V/v (horizontal/vertical lineto)
  - C/c (cubic Bézier)
  - S/s (smooth cubic Bézier)
  - Q/q (quadratic Bézier)
  - T/t (smooth quadratic Bézier)
  - A/a (elliptical arc)
  - Z/z (closepath)
- [ ] Path parsing and normalization

#### 23.4 Text
- [ ] `<text>` element
- [ ] `<tspan>` for styling portions
- [ ] `textLength` and `lengthAdjust`
- [ ] `<textPath>` for text on a path (optional)

#### 23.5 Grouping and Reuse
- [ ] `<g>` grouping element
- [ ] `<defs>` for reusable definitions
- [ ] `<use>` for referencing elements
- [ ] `<symbol>` for reusable graphics

#### 23.6 Painting
- [ ] Fill properties (`fill`, `fill-opacity`, `fill-rule`)
- [ ] Stroke properties (`stroke`, `stroke-width`, `stroke-opacity`, etc.)
- [ ] `<linearGradient>` and `<radialGradient>`
- [ ] `<pattern>` fills
- [ ] Markers (`<marker>`, `marker-start`, `marker-mid`, `marker-end`)

#### 23.7 Clipping and Masking
- [ ] `<clipPath>` element
- [ ] `clip-path` property
- [ ] `<mask>` element
- [ ] `mask` property

#### 23.8 Filters (Basic)
- [ ] `<filter>` element
- [ ] `feGaussianBlur` - blur effect
- [ ] `feColorMatrix` - color manipulation
- [ ] `feOffset` - drop shadow component
- [ ] `feComposite` - combining images
- [ ] `feMerge` - layering filter results
- [ ] Filter region and primitives coordinate system

#### 23.9 SVG Animation (SMIL - Optional)
- [ ] `<animate>` for property animation
- [ ] `<animateTransform>` for transform animation
- [ ] `<animateMotion>` for path-based motion
- [ ] Integration with CSS animations (preferred method)

### Third-Party Libraries
- **Recommended:** `resvg` - High-quality SVG rendering library in Rust
  - Handles path parsing, rendering, text layout
  - Can output to `tiny-skia` (software) or adapt to wgpu
- Alternative: `svg` crate for parsing only
- `lyon` - Tessellation for GPU rendering of paths

### Custom Implementation Required
```rust
// Integration layer with resvg or custom implementation
pub struct SvgDocument {
    tree: usvg::Tree,  // From resvg's usvg
    viewport: Rect,
}

pub struct SvgRenderer {
    // Either use resvg's renderer or custom wgpu pipeline
}

// If custom:
pub enum SvgNode {
    Group(SvgGroup),
    Path(SvgPath),
    Rect(SvgRect),
    Circle(SvgCircle),
    Text(SvgText),
    Use(SvgUse),
    // ...
}
```

### Acceptance Criteria
- [ ] Render inline `<svg>` in HTML
- [ ] Render SVG images (`<img src="icon.svg">`)
- [ ] Basic shapes render correctly
- [ ] Paths with curves render correctly
- [ ] Gradients work
- [ ] `<use>` references resolve
- [ ] CSS styling of SVG elements works
- [ ] SVG animations play (CSS method)
- [ ] Pass 40% of WPT `svg/` tests

### Risk Assessment
- **Scope creep**: SVG spec is enormous; must limit to common features
- **Performance**: Complex SVGs with many paths can be slow
- **Text**: SVG text layout is complex (may defer advanced features)

---

## Phase 24: Canvas 2D

### Overview
Implement the HTML Canvas 2D API for immediate-mode graphics rendering. Essential for games, charts, image editing, and custom visualizations.

### Priority: High
### Estimated Duration: 4-5 weeks
### Dependencies: Phase 14 (Events), Phase 16 (Images)

### Sub-Tasks

#### 24.1 Canvas Element
- [ ] `<canvas>` element with width/height attributes
- [ ] `canvas.getContext('2d')` method
- [ ] `canvas.width`, `canvas.height` properties
- [ ] `canvas.toDataURL()` and `canvas.toBlob()`
- [ ] CSS sizing vs. canvas resolution

#### 24.2 Drawing State
- [ ] `save()` and `restore()` state stack
- [ ] Transformation matrix state
- [ ] Clipping region state
- [ ] Style state (fill, stroke, etc.)

#### 24.3 Transformations
- [ ] `translate(x, y)`
- [ ] `rotate(angle)`
- [ ] `scale(x, y)`
- [ ] `transform(a, b, c, d, e, f)` - apply matrix
- [ ] `setTransform(a, b, c, d, e, f)` - replace matrix
- [ ] `resetTransform()`
- [ ] `getTransform()` - returns DOMMatrix

#### 24.4 Compositing
- [ ] `globalAlpha`
- [ ] `globalCompositeOperation` (source-over, multiply, screen, etc.)

#### 24.5 Fill and Stroke Styles
- [ ] Solid colors (strings, CSS colors)
- [ ] `CanvasGradient` - `createLinearGradient()`, `createRadialGradient()`
- [ ] `CanvasPattern` - `createPattern()`
- [ ] `fillStyle` and `strokeStyle` properties

#### 24.6 Line Styles
- [ ] `lineWidth`
- [ ] `lineCap` (butt, round, square)
- [ ] `lineJoin` (round, bevel, miter)
- [ ] `miterLimit`
- [ ] `setLineDash()` and `getLineDash()`
- [ ] `lineDashOffset`

#### 24.7 Shadows
- [ ] `shadowColor`
- [ ] `shadowBlur`
- [ ] `shadowOffsetX`, `shadowOffsetY`

#### 24.8 Rectangles
- [ ] `fillRect(x, y, w, h)`
- [ ] `strokeRect(x, y, w, h)`
- [ ] `clearRect(x, y, w, h)`

#### 24.9 Paths
- [ ] `beginPath()`
- [ ] `closePath()`
- [ ] `moveTo(x, y)`
- [ ] `lineTo(x, y)`
- [ ] `bezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y)`
- [ ] `quadraticCurveTo(cpx, cpy, x, y)`
- [ ] `arc(x, y, r, startAngle, endAngle, counterclockwise)`
- [ ] `arcTo(x1, y1, x2, y2, radius)`
- [ ] `ellipse(x, y, rx, ry, rotation, start, end, ccw)`
- [ ] `rect(x, y, w, h)`
- [ ] `fill()` and `stroke()`
- [ ] `clip()`
- [ ] `isPointInPath()` and `isPointInStroke()`
- [ ] `Path2D` object

#### 24.10 Text
- [ ] `font` property
- [ ] `textAlign` (start, end, left, right, center)
- [ ] `textBaseline` (top, hanging, middle, alphabetic, ideographic, bottom)
- [ ] `direction` (ltr, rtl, inherit)
- [ ] `fillText(text, x, y, maxWidth)`
- [ ] `strokeText(text, x, y, maxWidth)`
- [ ] `measureText(text)` → TextMetrics

#### 24.11 Images
- [ ] `drawImage(image, dx, dy)`
- [ ] `drawImage(image, dx, dy, dw, dh)`
- [ ] `drawImage(image, sx, sy, sw, sh, dx, dy, dw, dh)`
- [ ] Support for: HTMLImageElement, HTMLCanvasElement, ImageBitmap

#### 24.12 Pixel Manipulation
- [ ] `createImageData(w, h)` and `createImageData(imagedata)`
- [ ] `getImageData(sx, sy, sw, sh)`
- [ ] `putImageData(imagedata, dx, dy)`
- [ ] `ImageData` object

#### 24.13 Hit Regions (Optional)
- [ ] `addHitRegion()`, `removeHitRegion()`, `clearHitRegions()`

### Third-Party Libraries
- **Recommended:** `tiny-skia` - Pure Rust 2D graphics library
  - CPU-based rendering, very compatible
  - Supports paths, gradients, patterns, text (basic)
- Alternative: Custom wgpu-based renderer for GPU acceleration
- `fontdue` or DirectWrite for text rendering

### Custom Implementation Required
```rust
// New crate: rustkit-canvas or extend rustkit-compositor
pub struct CanvasRenderingContext2D {
    canvas: CanvasId,
    state_stack: Vec<CanvasState>,
    current_state: CanvasState,
    pixmap: tiny_skia::Pixmap,  // Or wgpu texture
    current_path: Path,
}

pub struct CanvasState {
    transform: Transform,
    clip_path: Option<Path>,
    fill_style: FillStyle,
    stroke_style: StrokeStyle,
    line_width: f32,
    line_cap: LineCap,
    line_join: LineJoin,
    miter_limit: f32,
    line_dash: Vec<f32>,
    line_dash_offset: f32,
    shadow_color: Color,
    shadow_blur: f32,
    shadow_offset: (f32, f32),
    global_alpha: f32,
    global_composite_op: CompositeOp,
    font: CanvasFont,
    text_align: TextAlign,
    text_baseline: TextBaseline,
}

pub enum FillStyle {
    Color(Color),
    Gradient(CanvasGradient),
    Pattern(CanvasPattern),
}
```

### Acceptance Criteria
- [ ] Draw shapes (rects, circles, paths)
- [ ] Draw images
- [ ] Draw text
- [ ] Transformations work
- [ ] Gradients and patterns work
- [ ] `getImageData`/`putImageData` work
- [ ] Save/restore state works
- [ ] Pass 50% of WPT `html/canvas/element/` tests
- [ ] Simple canvas games run (e.g., pong, snake)

### Risk Assessment
- **Performance**: CPU rendering may be slow for complex scenes
- **Text**: Canvas text rendering quality depends on font library
- **Pixel operations**: Must handle color spaces correctly

---

## Phase 25: Audio/Video

### Overview
Implement `<audio>` and `<video>` elements with HTML5 media APIs. Essential for media-rich websites.

### Priority: Medium
### Estimated Duration: 6-8 weeks
### Dependencies: Phase 14 (Events), Phase 16 (Images)

### Sub-Tasks

#### 25.1 Media Elements
- [ ] `<audio>` element
- [ ] `<video>` element with poster attribute
- [ ] `<source>` element for multiple formats
- [ ] `<track>` element for subtitles/captions

#### 25.2 Media Attributes
- [ ] `src` - media URL
- [ ] `controls` - show native controls
- [ ] `autoplay` (with autoplay policy)
- [ ] `loop`
- [ ] `muted`
- [ ] `preload` (none, metadata, auto)
- [ ] `crossorigin`

#### 25.3 HTMLMediaElement API
- [ ] Properties:
  - `currentTime`, `duration`
  - `paused`, `ended`
  - `volume`, `muted`
  - `playbackRate`, `defaultPlaybackRate`
  - `readyState`, `networkState`
  - `buffered`, `seekable`, `played` (TimeRanges)
  - `currentSrc`
  - `videoWidth`, `videoHeight` (video only)
- [ ] Methods:
  - `play()` → Promise
  - `pause()`
  - `load()`
  - `canPlayType(type)` → string

#### 25.4 Media Events
- [ ] `loadstart`, `progress`, `suspend`, `abort`, `error`
- [ ] `emptied`, `stalled`
- [ ] `loadedmetadata`, `loadeddata`
- [ ] `canplay`, `canplaythrough`
- [ ] `playing`, `pause`, `ended`
- [ ] `waiting`, `seeking`, `seeked`
- [ ] `timeupdate`, `durationchange`
- [ ] `ratechange`, `volumechange`

#### 25.5 Native Controls
- [ ] Play/pause button
- [ ] Progress bar with seeking
- [ ] Volume slider
- [ ] Mute button
- [ ] Fullscreen button (video)
- [ ] Time display

#### 25.6 Text Tracks (Subtitles)
- [ ] WebVTT parsing
- [ ] `TextTrack` API
- [ ] Cue rendering and positioning
- [ ] Track selection UI

#### 25.7 Codec Support
- [ ] Audio: MP3, AAC, Opus, Vorbis, WAV
- [ ] Video: H.264, VP8, VP9, AV1 (optional)
- [ ] Container: MP4, WebM, Ogg

#### 25.8 Media Source Extensions (MSE) - Optional
- [ ] `MediaSource` object
- [ ] `SourceBuffer` for appending media segments
- [ ] Adaptive bitrate streaming support

### Third-Party Libraries
- **Windows:** Media Foundation (via `windows` crate)
  - Native codec support
  - Hardware acceleration
- **Cross-platform alternatives:**
  - `gstreamer-rs` - GStreamer bindings
  - `ffmpeg-next` - FFmpeg bindings
  - `symphonia` - Pure Rust audio decoding
  - `rav1e` + `dav1d` for AV1

### Custom Implementation Required
```rust
// New crate: rustkit-media
pub struct MediaPlayer {
    state: MediaState,
    source: Option<MediaSource>,
    decoder: Box<dyn MediaDecoder>,
    audio_output: Box<dyn AudioOutput>,
    video_output: Option<VideoSurface>,
}

pub trait MediaDecoder {
    fn open(&mut self, url: &Url) -> Result<MediaInfo, MediaError>;
    fn decode_frame(&mut self) -> Result<MediaFrame, MediaError>;
    fn seek(&mut self, time: f64) -> Result<(), MediaError>;
}

pub enum MediaFrame {
    Audio(AudioFrame),
    Video(VideoFrame),
}

// Platform-specific implementations
#[cfg(windows)]
pub struct MediaFoundationDecoder { ... }
```

### Acceptance Criteria
- [ ] Play MP3 audio files
- [ ] Play MP4/H.264 video files
- [ ] Native controls work
- [ ] Seeking works
- [ ] Volume control works
- [ ] Autoplay respects policy
- [ ] Subtitles display
- [ ] Video renders to texture
- [ ] Pass 40% of WPT `html/semantics/embedded-content/media-elements/` tests

### Risk Assessment
- **Platform dependency**: Media codecs vary by platform
- **DRM**: EME (Encrypted Media Extensions) is complex and may require licensing
- **Performance**: Video decoding is CPU/GPU intensive
- **Legal**: Codec licensing (H.264, AAC)

---

## Phase 26: WebGL

### Overview
Implement WebGL 1.0 (OpenGL ES 2.0 based) for 3D graphics and GPU-accelerated 2D rendering. Essential for games, data visualization, and creative applications.

### Priority: Medium
### Estimated Duration: 6-8 weeks
### Dependencies: Phase 24 (Canvas 2D for context pattern)

### Sub-Tasks

#### 26.1 WebGL Context
- [ ] `canvas.getContext('webgl')` and `getContext('webgl2')`
- [ ] Context attributes (alpha, depth, stencil, antialias, etc.)
- [ ] Context loss and restoration
- [ ] `WebGLRenderingContext` interface

#### 26.2 Shaders
- [ ] `createShader()`, `shaderSource()`, `compileShader()`
- [ ] `createProgram()`, `attachShader()`, `linkProgram()`
- [ ] `useProgram()`
- [ ] Shader compilation errors
- [ ] GLSL ES 1.0 validation

#### 26.3 Buffers
- [ ] `createBuffer()`, `bindBuffer()`, `bufferData()`
- [ ] `ARRAY_BUFFER`, `ELEMENT_ARRAY_BUFFER`
- [ ] Typed array support (Float32Array, Uint16Array, etc.)

#### 26.4 Attributes and Uniforms
- [ ] `getAttribLocation()`, `vertexAttribPointer()`, `enableVertexAttribArray()`
- [ ] `getUniformLocation()`, `uniform[1234][fi][v]()`
- [ ] `uniformMatrix[234]fv()`

#### 26.5 Textures
- [ ] `createTexture()`, `bindTexture()`, `texImage2D()`
- [ ] `texParameteri()` for filtering and wrapping
- [ ] Texture formats (RGBA, RGB, LUMINANCE, etc.)
- [ ] Mipmaps with `generateMipmap()`
- [ ] Texture units with `activeTexture()`

#### 26.6 Framebuffers
- [ ] `createFramebuffer()`, `bindFramebuffer()`
- [ ] `framebufferTexture2D()`, `framebufferRenderbuffer()`
- [ ] `checkFramebufferStatus()`
- [ ] Render-to-texture

#### 26.7 Renderbuffers
- [ ] `createRenderbuffer()`, `bindRenderbuffer()`
- [ ] `renderbufferStorage()`

#### 26.8 Drawing
- [ ] `drawArrays(mode, first, count)`
- [ ] `drawElements(mode, count, type, offset)`
- [ ] Draw modes: POINTS, LINES, LINE_STRIP, TRIANGLES, etc.

#### 26.9 State
- [ ] `enable()`, `disable()` - depth test, blending, culling, etc.
- [ ] `blendFunc()`, `blendEquation()`
- [ ] `depthFunc()`, `depthMask()`, `depthRange()`
- [ ] `cullFace()`, `frontFace()`
- [ ] `viewport()`, `scissor()`
- [ ] `clear()`, `clearColor()`, `clearDepth()`, `clearStencil()`

#### 26.10 Reading
- [ ] `readPixels()`
- [ ] `getParameter()`
- [ ] `getError()`

#### 26.11 Extensions
- [ ] Extension query mechanism
- [ ] Common extensions:
  - `OES_texture_float`
  - `OES_element_index_uint`
  - `WEBGL_lose_context`
  - `ANGLE_instanced_arrays`

#### 26.12 WebGL 2.0 (Optional, Future)
- [ ] Uniform buffer objects
- [ ] Transform feedback
- [ ] Multiple render targets
- [ ] 3D textures
- [ ] Sampler objects

### Third-Party Libraries
- **wgpu** - Already used; can translate WebGL to wgpu
- `naga` - Shader translation (GLSL → WGSL/SPIR-V)
- Consider: `wgpu-webgl` approach for compatibility

### Custom Implementation Required
```rust
// New module in rustkit-compositor or separate crate
pub struct WebGLRenderingContext {
    canvas_id: CanvasId,
    wgpu_device: Arc<wgpu::Device>,
    wgpu_queue: Arc<wgpu::Queue>,

    // WebGL state machine
    current_program: Option<WebGLProgram>,
    bound_array_buffer: Option<WebGLBuffer>,
    bound_element_buffer: Option<WebGLBuffer>,
    bound_framebuffer: Option<WebGLFramebuffer>,
    bound_textures: [Option<WebGLTexture>; 32],
    active_texture_unit: u32,

    // State
    viewport: (i32, i32, u32, u32),
    clear_color: [f32; 4],
    blend_enabled: bool,
    depth_test_enabled: bool,
    // ... lots more state
}

// Translation layer
fn compile_glsl_to_wgsl(source: &str, stage: ShaderStage) -> Result<String, ShaderError> {
    // Use naga for translation
}
```

### Acceptance Criteria
- [ ] Create WebGL context
- [ ] Compile and link shaders
- [ ] Draw triangles
- [ ] Texture mapping works
- [ ] Depth testing works
- [ ] Blending works
- [ ] Simple three.js scene renders
- [ ] Pass 40% of WebGL conformance tests

### Risk Assessment
- **Complexity**: WebGL API surface is large
- **Shader translation**: GLSL to WGSL is non-trivial
- **Conformance**: WebGL conformance tests are extensive
- **Security**: GPU access requires careful validation

---

## Phase 27: Service Workers

### Overview
Implement Service Workers for offline support, push notifications, and request interception. Foundation for Progressive Web Apps (PWAs).

### Priority: Medium
### Estimated Duration: 5-6 weeks
### Dependencies: Phase 19 (Navigation), Phase 20 (Security)

### Sub-Tasks

#### 27.1 Service Worker Registration
- [ ] `navigator.serviceWorker.register(scriptURL, options)`
- [ ] Scope resolution
- [ ] Registration promise
- [ ] `ServiceWorkerRegistration` object

#### 27.2 Service Worker Lifecycle
- [ ] `install` event
- [ ] `activate` event
- [ ] `waiting`, `active`, `installing` states
- [ ] Update flow
- [ ] `skipWaiting()` and `clients.claim()`

#### 27.3 Fetch Interception
- [ ] `fetch` event in service worker
- [ ] `FetchEvent.respondWith()`
- [ ] `FetchEvent.request`
- [ ] Request/Response cloning

#### 27.4 Cache API
- [ ] `caches.open(cacheName)`
- [ ] `cache.add()`, `cache.addAll()`
- [ ] `cache.put()`
- [ ] `cache.match()`, `cache.matchAll()`
- [ ] `cache.delete()`
- [ ] `cache.keys()`
- [ ] `caches.match()`, `caches.keys()`, `caches.delete()`

#### 27.5 Clients API
- [ ] `clients.get(id)`
- [ ] `clients.matchAll(options)`
- [ ] `clients.openWindow(url)`
- [ ] `clients.claim()`
- [ ] `Client` object (id, url, type)

#### 27.6 Service Worker Communication
- [ ] `postMessage` to/from service worker
- [ ] `MessageChannel` support
- [ ] `BroadcastChannel` (optional)

#### 27.7 Push Notifications (Optional)
- [ ] `push` event
- [ ] `PushManager.subscribe()`
- [ ] `PushSubscription` object
- [ ] Notification display

#### 27.8 Background Sync (Optional)
- [ ] `sync` event
- [ ] `SyncManager.register()`

### Third-Party Libraries
- None specifically required
- Reuse rustkit-js (boa) for worker execution
- Reuse rustkit-net for fetch

### Custom Implementation Required
```rust
// New crate: rustkit-sw (Service Workers)
pub struct ServiceWorkerContainer {
    registrations: HashMap<Scope, ServiceWorkerRegistration>,
    controller: Option<ServiceWorker>,
}

pub struct ServiceWorker {
    id: ServiceWorkerId,
    script_url: Url,
    state: ServiceWorkerState,
    js_context: JsRuntime,  // Isolated context
}

pub struct ServiceWorkerRegistration {
    scope: Url,
    installing: Option<ServiceWorker>,
    waiting: Option<ServiceWorker>,
    active: Option<ServiceWorker>,
}

// Cache storage (persistent)
pub struct CacheStorage {
    storage_path: PathBuf,
    caches: HashMap<String, Cache>,
}

pub struct Cache {
    name: String,
    entries: HashMap<RequestKey, CachedResponse>,
}
```

### Acceptance Criteria
- [ ] Register a service worker
- [ ] Service worker intercepts fetch requests
- [ ] Cache API stores/retrieves responses
- [ ] Offline page works
- [ ] Service worker updates correctly
- [ ] `postMessage` works
- [ ] Pass 40% of WPT `service-workers/` tests

### Risk Assessment
- **Isolation**: Service workers need isolated JS contexts
- **Persistence**: Cache storage must survive browser restart
- **Updates**: Update algorithm is complex
- **Security**: Must enforce origin restrictions

---

## Phase 28: IndexedDB

### Overview
Implement IndexedDB, a low-level API for client-side storage of significant amounts of structured data, including files and blobs.

### Priority: Medium
### Estimated Duration: 4-5 weeks
### Dependencies: Phase 20 (Security for origin isolation)

### Sub-Tasks

#### 28.1 Opening Databases
- [ ] `indexedDB.open(name, version)`
- [ ] Version change handling
- [ ] `onupgradeneeded` event
- [ ] `onsuccess`, `onerror` events
- [ ] `indexedDB.deleteDatabase(name)`

#### 28.2 Object Stores
- [ ] `db.createObjectStore(name, options)`
- [ ] `db.deleteObjectStore(name)`
- [ ] Key paths and key generators
- [ ] `objectStore.put()`, `objectStore.add()`
- [ ] `objectStore.get()`, `objectStore.getAll()`
- [ ] `objectStore.delete()`, `objectStore.clear()`
- [ ] `objectStore.count()`

#### 28.3 Indexes
- [ ] `objectStore.createIndex(name, keyPath, options)`
- [ ] `objectStore.deleteIndex(name)`
- [ ] `index.get()`, `index.getAll()`
- [ ] `index.getKey()`, `index.getAllKeys()`
- [ ] Unique indexes

#### 28.4 Cursors
- [ ] `objectStore.openCursor(query, direction)`
- [ ] `index.openCursor(query, direction)`
- [ ] `cursor.continue()`, `cursor.advance(count)`
- [ ] `cursor.update()`, `cursor.delete()`
- [ ] Key cursors with `openKeyCursor()`

#### 28.5 Transactions
- [ ] `db.transaction(storeNames, mode)`
- [ ] Transaction modes: `readonly`, `readwrite`
- [ ] `transaction.objectStore(name)`
- [ ] Auto-commit behavior
- [ ] `transaction.abort()`
- [ ] `oncomplete`, `onerror`, `onabort` events

#### 28.6 Key Ranges
- [ ] `IDBKeyRange.only(value)`
- [ ] `IDBKeyRange.lowerBound(lower, open)`
- [ ] `IDBKeyRange.upperBound(upper, open)`
- [ ] `IDBKeyRange.bound(lower, upper, lowerOpen, upperOpen)`

#### 28.7 Structured Clone
- [ ] Clone algorithm for stored values
- [ ] Support for: primitives, objects, arrays, Date, RegExp, Blob, File, ArrayBuffer, typed arrays, Map, Set
- [ ] Circular reference handling

### Third-Party Libraries
- **Recommended:** `sled` or `rocksdb` for underlying storage
- Alternative: SQLite via `rusqlite`
- `bincode` or `serde_json` for serialization

### Custom Implementation Required
```rust
// New crate: rustkit-storage
pub struct IndexedDBFactory {
    databases: HashMap<(Origin, String), IDBDatabase>,
    storage_path: PathBuf,
}

pub struct IDBDatabase {
    name: String,
    version: u64,
    object_stores: HashMap<String, IDBObjectStore>,
    backend: Box<dyn StorageBackend>,  // sled, sqlite, etc.
}

pub struct IDBObjectStore {
    name: String,
    key_path: Option<KeyPath>,
    auto_increment: bool,
    indexes: HashMap<String, IDBIndex>,
}

pub struct IDBTransaction {
    mode: TransactionMode,
    stores: Vec<String>,
    operations: Vec<TransactionOp>,
    state: TransactionState,
}

// Storage backend trait
pub trait StorageBackend: Send + Sync {
    fn get(&self, store: &str, key: &IDBKey) -> Result<Option<Value>, StorageError>;
    fn put(&mut self, store: &str, key: &IDBKey, value: Value) -> Result<(), StorageError>;
    fn delete(&mut self, store: &str, key: &IDBKey) -> Result<bool, StorageError>;
    fn cursor(&self, store: &str, range: Option<&IDBKeyRange>, direction: Direction)
        -> Box<dyn Cursor>;
}
```

### Acceptance Criteria
- [ ] Open/create databases
- [ ] Create object stores with indexes
- [ ] CRUD operations work
- [ ] Transactions commit and rollback
- [ ] Cursors iterate correctly
- [ ] Key ranges filter correctly
- [ ] Data persists across sessions
- [ ] Pass 50% of WPT `IndexedDB/` tests

### Risk Assessment
- **Complexity**: IndexedDB API is large and asynchronous
- **Structured clone**: Must implement full clone algorithm
- **Transactions**: ACID compliance is challenging
- **Storage quotas**: Must implement quota management

---

## Phase 29: WebRTC

### Overview
Implement WebRTC for real-time peer-to-peer communication including video chat, audio calls, and data channels.

### Priority: Low-Medium
### Estimated Duration: 8-10 weeks
### Dependencies: Phase 25 (Audio/Video), Phase 20 (Security)

### Sub-Tasks

#### 29.1 RTCPeerConnection
- [ ] Constructor with configuration (ICE servers)
- [ ] `createOffer()` and `createAnswer()`
- [ ] `setLocalDescription()` and `setRemoteDescription()`
- [ ] `addIceCandidate()`
- [ ] Connection state management
- [ ] ICE gathering state

#### 29.2 ICE (Interactive Connectivity Establishment)
- [ ] ICE candidate gathering
- [ ] STUN server support
- [ ] TURN server support (relay)
- [ ] ICE candidate trickling
- [ ] ICE restart

#### 29.3 SDP (Session Description Protocol)
- [ ] SDP parsing and generation
- [ ] Media descriptions (audio, video, data)
- [ ] Codec negotiation
- [ ] `RTCSessionDescription` object

#### 29.4 Media Tracks
- [ ] `addTrack(track, ...streams)`
- [ ] `removeTrack(sender)`
- [ ] `ontrack` event
- [ ] `RTCRtpSender` and `RTCRtpReceiver`
- [ ] `getTransceivers()`

#### 29.5 MediaStream
- [ ] `navigator.mediaDevices.getUserMedia(constraints)`
- [ ] `MediaStream` object
- [ ] `MediaStreamTrack` (audio/video)
- [ ] Track constraints (width, height, frameRate, etc.)
- [ ] `enumerateDevices()`
- [ ] Screen capture with `getDisplayMedia()` (optional)

#### 29.6 Data Channels
- [ ] `createDataChannel(label, options)`
- [ ] `ondatachannel` event
- [ ] `RTCDataChannel` object
  - `send()` for strings and binary
  - `onmessage`, `onopen`, `onclose`, `onerror`
  - Ordered and unordered delivery
  - Reliability options

#### 29.7 Statistics
- [ ] `getStats()` returning `RTCStatsReport`
- [ ] Various stats types (inbound-rtp, outbound-rtp, etc.)

#### 29.8 DTLS and SRTP
- [ ] DTLS handshake for key exchange
- [ ] SRTP encryption for media
- [ ] Certificate fingerprint validation

### Third-Party Libraries
- **Highly Recommended:** `webrtc-rs` (Pure Rust WebRTC implementation)
  - Includes ICE, DTLS, SRTP, SCTP
  - Actively maintained
- Alternative: Native WebRTC library via FFI
- `opus` for audio codec
- Video codec: platform-specific or `vpx-rs`

### Custom Implementation Required
```rust
// Integration with webrtc-rs
pub struct RTCPeerConnection {
    inner: webrtc::peer_connection::RTCPeerConnection,
    event_handlers: PeerConnectionEventHandlers,
}

impl RTCPeerConnection {
    pub async fn create_offer(&self) -> Result<RTCSessionDescription, JsError> {
        let offer = self.inner.create_offer(None).await?;
        Ok(RTCSessionDescription::from(offer))
    }

    // ... wrap webrtc-rs API for JavaScript
}

// Media device access
pub struct MediaDevices {
    audio_devices: Vec<MediaDeviceInfo>,
    video_devices: Vec<MediaDeviceInfo>,
}

impl MediaDevices {
    pub async fn get_user_media(&self, constraints: MediaStreamConstraints)
        -> Result<MediaStream, MediaError> {
        // Access camera/microphone
        // Platform-specific implementation
    }
}
```

### Acceptance Criteria
- [ ] Establish peer connection with signaling
- [ ] ICE candidates exchange
- [ ] Audio call between two tabs
- [ ] Video call between two tabs
- [ ] Data channel messaging works
- [ ] Connection survives ICE restart
- [ ] Works with common STUN/TURN servers

### Risk Assessment
- **Complexity**: WebRTC is one of the most complex web APIs
- **NAT traversal**: Real-world network conditions are challenging
- **Media pipelines**: Audio/video sync and processing
- **Platform specifics**: Camera/microphone access varies by platform

---

## Phase 30: Accessibility (a11y)

### Overview
Implement accessibility features to make web content usable by people with disabilities. This includes screen reader support, keyboard navigation, and ARIA.

### Priority: High (should be ongoing, not just one phase)
### Estimated Duration: 6-8 weeks (core), ongoing
### Dependencies: Phase 14 (Events), Phase 15 (Forms)

### Sub-Tasks

#### 30.1 Accessibility Tree
- [ ] Build accessibility tree parallel to DOM
- [ ] Map HTML elements to accessibility roles
- [ ] Compute accessible names
- [ ] Compute accessible descriptions
- [ ] State and property computation

#### 30.2 ARIA Support
- [ ] `role` attribute
- [ ] ARIA states: `aria-checked`, `aria-selected`, `aria-expanded`, etc.
- [ ] ARIA properties: `aria-label`, `aria-labelledby`, `aria-describedby`
- [ ] ARIA live regions: `aria-live`, `aria-atomic`, `aria-relevant`
- [ ] ARIA relationships: `aria-controls`, `aria-owns`, `aria-flowto`

#### 30.3 Semantic HTML
- [ ] Proper roles for HTML elements
  - `<button>` → button role
  - `<a href>` → link role
  - `<input>` → various roles based on type
  - `<nav>`, `<main>`, `<header>`, `<footer>` → landmark roles
  - `<h1>`-`<h6>` → heading roles with levels
  - `<table>` → table/grid roles
  - `<ul>`, `<ol>` → list roles

#### 30.4 Focus Management
- [ ] Focus visible indicator (`:focus-visible`)
- [ ] `tabindex` support
- [ ] Focus trapping for modals
- [ ] Skip links
- [ ] Focus order matching DOM order

#### 30.5 Keyboard Navigation
- [ ] Tab/Shift+Tab for focus
- [ ] Arrow keys for widgets (menus, trees, grids)
- [ ] Enter/Space for activation
- [ ] Escape for dismissal
- [ ] Keyboard shortcuts (accesskey)

#### 30.6 Screen Reader Integration
- [ ] **Windows:** UI Automation provider
  - Implement `IRawElementProviderSimple`
  - Expose accessibility tree to NVDA, JAWS, Narrator
- [ ] Alternative: IAccessible2 (older API)

#### 30.7 Text Alternatives
- [ ] `alt` text for images
- [ ] `<figcaption>` for figures
- [ ] Table headers association
- [ ] Form labels association

#### 30.8 Color and Contrast
- [ ] Respect `prefers-contrast` media query
- [ ] Respect `prefers-reduced-motion`
- [ ] Respect `forced-colors` mode
- [ ] High contrast mode support

#### 30.9 Accessibility APIs
- [ ] `Element.ariaLabel`, `Element.ariaDescribedBy`, etc.
- [ ] `Element.role`
- [ ] `ElementInternals` for custom elements

### Third-Party Libraries
- **Windows:** `windows` crate for UI Automation COM interfaces
- Reference: `accesskit` - Cross-platform accessibility toolkit in Rust (could be very useful)

### Custom Implementation Required
```rust
// New crate: rustkit-a11y
pub struct AccessibilityTree {
    root: AccessibilityNode,
    nodes: HashMap<NodeId, AccessibilityNode>,
}

pub struct AccessibilityNode {
    id: AccessibilityId,
    role: Role,
    name: Option<String>,
    description: Option<String>,
    value: Option<String>,
    states: HashSet<State>,
    properties: HashMap<Property, PropertyValue>,
    parent: Option<AccessibilityId>,
    children: Vec<AccessibilityId>,
    dom_node: NodeId,
    bounds: Rect,
}

pub enum Role {
    Button,
    Link,
    Textbox,
    Checkbox,
    Radio,
    Slider,
    Heading(u8),
    List,
    ListItem,
    Table,
    Row,
    Cell,
    Menu,
    MenuItem,
    Dialog,
    Alert,
    // ... many more
}

// Windows UI Automation provider
#[cfg(windows)]
pub struct UIAutomationProvider {
    a11y_tree: Arc<RwLock<AccessibilityTree>>,
}

#[cfg(windows)]
impl IRawElementProviderSimple_Impl for UIAutomationProvider {
    // Implement UI Automation interface
}
```

### Acceptance Criteria
- [ ] Screen reader announces page content
- [ ] All interactive elements are keyboard accessible
- [ ] Focus is visible
- [ ] Images have alt text (when provided)
- [ ] Form inputs have labels
- [ ] ARIA roles are exposed
- [ ] Live regions announce changes
- [ ] Pass 50% of WPT `wai-aria/` tests
- [ ] Pass manual screen reader testing with NVDA

### Risk Assessment
- **Platform APIs**: UI Automation is Windows-specific
- **Testing**: Accessibility testing requires manual verification
- **Ongoing**: A11y should be built into every feature, not bolted on
- **Scope**: Full WCAG compliance is extensive

---

## Summary Table

| Phase | Name | Duration | Complexity | Dependencies |
|-------|------|----------|------------|--------------|
| 21 | CSS Grid | 4-6 weeks | Very High | 12, 17 |
| 22 | Animations | 3-4 weeks | High | 14, 12 |
| 23 | SVG | 5-7 weeks | High | 22, 16 |
| 24 | Canvas 2D | 4-5 weeks | Medium-High | 14, 16 |
| 25 | Audio/Video | 6-8 weeks | High | 14, 16 |
| 26 | WebGL | 6-8 weeks | Very High | 24 |
| 27 | Service Workers | 5-6 weeks | High | 19, 20 |
| 28 | IndexedDB | 4-5 weeks | Medium-High | 20 |
| 29 | WebRTC | 8-10 weeks | Very High | 25, 20 |
| 30 | Accessibility | 6-8 weeks | High | 14, 15 |

**Total Estimated Duration:** 52-67 weeks (~12-16 months)

---

## Recommended Order

Given dependencies and importance for real-world usage:

1. **Phase 21: CSS Grid** - Essential for modern layouts
2. **Phase 22: Animations** - Expected by users, enables Phase 23
3. **Phase 24: Canvas 2D** - High value for games/charts
4. **Phase 30: Accessibility** - Should start early, build incrementally
5. **Phase 23: SVG** - Common for icons and graphics
6. **Phase 28: IndexedDB** - Needed by many web apps
7. **Phase 27: Service Workers** - Enables offline/PWA
8. **Phase 25: Audio/Video** - Media playback
9. **Phase 26: WebGL** - 3D graphics
10. **Phase 29: WebRTC** - Real-time communication

---

## Key Third-Party Libraries to Evaluate

| Library | Purpose | Phase |
|---------|---------|-------|
| `resvg` | SVG rendering | 23 |
| `tiny-skia` | 2D graphics | 23, 24 |
| `lyon` | Path tessellation | 23, 24 |
| `symphonia` | Audio decoding | 25 |
| `webrtc-rs` | WebRTC implementation | 29 |
| `accesskit` | Accessibility toolkit | 30 |
| `sled` / `rocksdb` | Key-value storage | 28 |
| `naga` | Shader translation | 26 |

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Scope creep | High | Strict MVP per phase, defer "nice-to-haves" |
| Performance | High | Benchmark early, profile often |
| Conformance testing | Medium | Use WPT subsets, not 100% compliance |
| Platform lock-in | Medium | Abstract platform-specific code |
| Maintenance burden | High | Good documentation, modular design |
| Third-party abandonment | Medium | Evaluate library health before adoption |
