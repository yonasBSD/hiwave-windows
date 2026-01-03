# RustKit Engine Roadmap

## Overview

RustKit is a Rust-native browser engine designed to replace WebKit's WinCairo port for the HiWave browser. This roadmap outlines the development phases beyond the initial MVP implementation.

**Current Status:** Phase 27 Complete (Service Workers) - Ready for Phase 28 (IndexedDB)
**Branch:** `master`

---

## Completed Phases (1-25)

| Phase | Name | Status | Description |
|-------|------|--------|-------------|
| 0 | Requirements | ✅ Complete | MVP requirements and acceptance tests |
| 1 | Fork & Harness | ✅ Complete | WebKit fork setup and embedder harness |
| 2 | Engine Skeleton | ✅ Complete | ViewHost, Compositor, Core runtime |
| 3 | HTML/DOM/CSS/Layout | ✅ Complete | DOM pipeline, CSS parsing, layout engine |
| 4 | JavaScript | ✅ Complete | Boa engine integration and DOM bindings |
| 5 | Networking | ✅ Complete | HTTP client, interception, downloads |
| 6 | Multi-view | ✅ Complete | Engine orchestration for multiple views |
| 7 | Observability | ✅ Complete | Logging, error handling, retry logic |
| 8 | HiWave Integration | ✅ Complete | IWebView adapter and feature flags |
| 9 | WPT Testing | ✅ Complete | Test harness for conformance testing |
| 10 | Benchmarks | ✅ Complete | Performance measurement infrastructure |
| 11 | WinCairo Removal | ✅ Complete | Removed legacy WebKit dependencies |
| 12 | CSS Box Model | ✅ Complete | Margin collapse, positioning, floats, z-index |
| 13 | Text Rendering | ✅ Complete | Font fallback, shaping, decorations |
| 14 | Event Handling | ✅ Complete | Mouse, keyboard, focus, touch, pointer events |
| 15 | Forms & Input | ✅ Complete | Text inputs, buttons, checkboxes, validation |
| 16 | Images & Media | ✅ Complete | PNG/JPEG/GIF/WebP decoding, background images |
| 17 | CSS Flexbox | ✅ Complete | Full flexbox layout implementation |
| 18 | Scrolling & Overflow | ✅ Complete | Scroll containers, smooth scroll, sticky |
| 19 | Navigation & History | ✅ Complete | History API, page lifecycle, hash navigation |
| 20 | Security & Isolation | ✅ Complete | CSP, CORS, same-origin, cookie security |
| 21 | CSS Grid Layout | ✅ Complete | Grid template, auto-placement, track sizing |
| 22 | Animations | ✅ Complete | CSS transitions, animations, timing functions |
| 23 | SVG Support | ✅ Complete | SVG parsing, paths, shapes, transforms |
| 24 | Canvas 2D | ✅ Complete | CanvasRenderingContext2D, paths, gradients |
| 25 | WebGL | ✅ Complete | WebGL 1.0 context, shaders, textures |
| 25.5 | GPU Renderer | ✅ Complete | Display list execution, text, images |
| 26 | Audio/Video | ✅ Complete | HTMLMediaElement, audio/video players |
| 27 | Service Workers | ✅ Complete | Registration, lifecycle, Cache API |

---

## Phase Details (Recently Completed)

### Phase 14: Event Handling ✅
**Status:** Complete

Full event system implementation:

- [x] **Mouse events** - click, dblclick, mousedown/up/move, hover
- [x] **Keyboard events** - keydown, keyup, keypress, input
- [x] **Focus management** - Tab navigation, focus/blur, :focus-visible
- [x] **Hit testing** - Accurate element hit detection
- [x] **Event bubbling** - Capture and bubble phases with stopPropagation
- [x] **Pointer events** - Modern unified input API
- [x] **Touch events** - Multi-touch support
- [x] **Drag events** - DnD API with DataTransfer
- [x] **requestAnimationFrame** - 60fps animation scheduling

---

### Phase 15: Forms & Input ✅
**Status:** Complete

HTML form controls:

- [x] **Text inputs** - `<input type="text">`, `<textarea>`
- [x] **Buttons** - `<button>`, `<input type="submit/button">`
- [x] **Checkboxes & radios** - State management
- [x] **Form submission** - GET/POST with encoding
- [x] **Validation** - HTML5 constraint validation
- [x] **Caret rendering** - Blinking cursor with selection

---

### Phase 16: Images & Media ✅
**Status:** Complete

Visual media support:

- [x] **Image decoding** - PNG, JPEG, GIF, WebP (via `image` crate)
- [x] **Image sizing** - `width`, `height`, `object-fit`, `object-position`
- [x] **Background images** - `background-image`, `background-size`, `background-repeat`
- [x] **Lazy loading** - `loading="lazy"` support
- [x] **Responsive images** - `srcset`, `sizes` parsing
- [x] **Image caching** - Memory and disk cache

---

### Phase 17: CSS Flexbox ✅
**Status:** Complete

Full Flexbox layout:

- [x] **Flex container** - `display: flex/inline-flex`, direction, wrap
- [x] **Flex items** - `flex-grow`, `flex-shrink`, `flex-basis`
- [x] **Alignment** - `justify-content`, `align-items`, `align-content`, `align-self`
- [x] **Order** - `order` property
- [x] **Gap** - `gap`, `row-gap`, `column-gap`

---

### Phase 18: Scrolling & Overflow ✅
**Status:** Complete

Scrolling implementation:

- [x] **Overflow handling** - `overflow: scroll/auto/hidden/clip`
- [x] **Scroll containers** - Proper scrolling regions
- [x] **Smooth scroll** - `scroll-behavior: smooth`
- [x] **Wheel events** - Mouse wheel scrolling
- [x] **Scroll APIs** - `scrollTo`, `scrollBy`, `scrollIntoView`
- [x] **Sticky positioning** - `position: sticky`
- [x] **Scrollbar rendering** - Custom scrollbar support

---

### Phase 19: Navigation & History ✅
**Status:** Complete

Navigation system:

- [x] **History API** - `pushState`, `replaceState`, `popstate`
- [x] **Back/Forward** - Browser navigation
- [x] **Hash navigation** - `#fragment` scrolling
- [x] **Page lifecycle** - `DOMContentLoaded`, `load`, `beforeunload`, `unload`
- [x] **Navigation timing** - Performance metrics
- [x] **Location object** - Full location API

---

### Phase 20: Security & Isolation ✅
**Status:** Complete

Security implementation:

- [x] **Content Security Policy** - CSP header parsing & enforcement
- [x] **Same-origin policy** - Cross-origin restrictions
- [x] **CORS** - Preflight and response headers
- [x] **Secure contexts** - HTTPS detection
- [x] **Cookie security** - SameSite, Secure attributes
- [x] **Referrer policy** - Full referrer policy support
- [x] **Mixed content** - Block insecure content on secure pages

---

### Phase 21: CSS Grid Layout ✅
**Status:** Complete

Grid layout implementation:

- [x] **Grid container** - `display: grid/inline-grid`
- [x] **Grid template** - `grid-template-columns`, `grid-template-rows`
- [x] **Track sizing** - `fr` units, `minmax()`, `auto`
- [x] **Grid areas** - `grid-template-areas`, named areas
- [x] **Auto-placement** - Row/column major, dense packing
- [x] **Gap** - `row-gap`, `column-gap`
- [x] **Alignment** - `justify-items`, `align-items`, `justify-self`, `align-self`

---

### Phase 22: CSS Animations & Transitions ✅
**Status:** Complete

**New crate: `rustkit-animation`**

- [x] **CSS transitions** - `transition-property`, `duration`, `timing-function`, `delay`
- [x] **CSS animations** - `@keyframes`, `animation-*` properties
- [x] **Timing functions** - `ease`, `linear`, `cubic-bezier()`, `steps()`
- [x] **Animation events** - `animationstart`, `animationend`, `transitionend`
- [x] **Property interpolation** - Colors, lengths, transforms
- [x] **Animation timeline** - Central management of all animations

---

### Phase 23: SVG Support ✅
**Status:** Complete

**New crate: `rustkit-svg`**

- [x] **SVG parsing** - Document, viewBox, elements
- [x] **Basic shapes** - rect, circle, ellipse, line, polyline, polygon
- [x] **Paths** - Full SVG path commands (M, L, C, S, Q, T, A, Z)
- [x] **Styling** - fill, stroke, opacity, transforms
- [x] **Transforms** - translate, scale, rotate, matrix
- [x] **Text** - Basic SVG text rendering

---

### Phase 24: Canvas 2D ✅
**Status:** Complete

**New crate: `rustkit-canvas`**

- [x] **CanvasRenderingContext2D** - Full 2D context
- [x] **State management** - save(), restore(), transforms
- [x] **Paths** - moveTo, lineTo, arc, bezier curves
- [x] **Drawing** - fill, stroke, fillRect, strokeRect, clearRect
- [x] **Text** - fillText, strokeText, measureText
- [x] **Images** - drawImage with all overloads
- [x] **Gradients** - LinearGradient, RadialGradient
- [x] **Pixel manipulation** - getImageData, putImageData

---

### Phase 25: WebGL ✅
**Status:** Complete

**New crate: `rustkit-webgl`**

- [x] **WebGLRenderingContext** - WebGL 1.0 context
- [x] **Shaders** - Compile and link GLSL shaders
- [x] **Buffers** - Vertex and index buffers
- [x] **Textures** - 2D textures, texture parameters
- [x] **Framebuffers** - Off-screen rendering targets
- [x] **Drawing** - drawArrays, drawElements
- [x] **Uniforms** - All uniform types
- [x] **State** - Enable/disable, blend, depth, stencil

---

## Upcoming Phases (26-30)

### Phase 26: Audio/Video ✅
**Status:** Complete

Media elements:

- [x] **`<audio>` element** - Audio playback with controls
- [x] **`<video>` element** - Video playback with controls
- [x] **Media controls** - Play, pause, seek, volume
- [x] **Media events** - play, pause, ended, timeupdate
- [x] **HTMLMediaElement API** - currentTime, duration, volume
- [x] **Subtitles** - TextTrack and TextTrackCue support
- [ ] **Fullscreen** - Fullscreen API for video (TODO)

---

### Phase 27: Service Workers ✅
**Status:** Complete

- [x] **Registration** - `navigator.serviceWorker.register()`
- [x] **Lifecycle** - install, activate, fetch events
- [x] **Fetch interception** - Cache-first offline support
- [x] **Cache API** - `caches.open()`, `cache.add()`, `cache.match()`
- [x] **Clients API** - `clients.matchAll()`, `clients.openWindow()`, `clients.claim()`
- [ ] **Push notifications** - Push API (TODO)

---

### Phase 28: IndexedDB
**Priority:** Medium | **Est. Duration:** 2-3 weeks

- [ ] **Database creation** - `indexedDB.open()`
- [ ] **Object stores** - createObjectStore, add, get, put, delete
- [ ] **Transactions** - readonly, readwrite
- [ ] **Indexes** - createIndex, getAll
- [ ] **Cursors** - openCursor, continue

---

### Phase 29: Web Workers
**Priority:** Medium | **Est. Duration:** 2-3 weeks

- [ ] **Dedicated Workers** - new Worker()
- [ ] **Message passing** - postMessage, onmessage
- [ ] **Shared Workers** - SharedWorker (basic)
- [ ] **Transferable objects** - ArrayBuffer transfer
- [ ] **Worker lifecycle** - terminate

---

### Phase 30: Accessibility
**Priority:** High | **Est. Duration:** 3-4 weeks

- [ ] **ARIA attributes** - role, aria-label, aria-describedby
- [ ] **Accessibility tree** - Generate from DOM
- [ ] **Screen reader support** - Windows UI Automation
- [ ] **Focus indicators** - Visible focus outlines
- [ ] **Keyboard navigation** - Full keyboard support

---

## Future Considerations (31+)

| Phase | Feature | Description |
|-------|---------|-------------|
| 31 | WebRTC | RTCPeerConnection, MediaStream, data channels |
| 32 | Web Crypto | SubtleCrypto, key generation, encryption |
| 33 | WebSockets | Full-duplex communication |
| 34 | WebAssembly | WASM execution (via wasmer/wasmtime) |
| 35 | CSS Filters | blur, brightness, contrast, etc. |
| 36 | CSS Transforms 3D | perspective, rotateX/Y/Z |
| 37 | Shadow DOM | Component encapsulation |
| 38 | Custom Elements | Web Components v1 |
| 39 | Clipboard API | async clipboard access |
| 40 | Gamepad API | Controller input |

---

## Performance Milestones

| Milestone | Target | Current |
|-----------|--------|---------|
| DOM parse 100KB | < 10ms | TBD |
| Layout 1000 boxes | < 5ms | TBD |
| First paint | < 100ms | TBD |
| Time to interactive | < 500ms | TBD |
| Memory per tab | < 50MB | TBD |

---

## Crate Structure

```
rustkit-common     # Shared utilities, logging, retry
rustkit-viewhost   # Win32 window hosting
rustkit-compositor # GPU rendering
rustkit-core       # Task scheduling, navigation, history
rustkit-dom        # HTML parsing, DOM tree, events
rustkit-css        # CSS parsing, style computation
rustkit-layout     # Layout algorithms (block, flex, grid)
rustkit-js         # JavaScript engine (Boa)
rustkit-bindings   # JS ↔ DOM bridge, Web APIs
rustkit-net        # HTTP, fetch, downloads, security
rustkit-image      # Image decoding and caching
rustkit-animation  # CSS animations and transitions
rustkit-svg        # SVG parsing and rendering
rustkit-canvas     # Canvas 2D API
rustkit-webgl      # WebGL API
rustkit-renderer   # GPU display list renderer
rustkit-media      # Audio/video playback
rustkit-sw         # Service workers
rustkit-engine     # Orchestration, multi-view
rustkit-test       # WPT harness
rustkit-bench      # Benchmarks
```

---

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `html5ever` | HTML parsing |
| `cssparser` | CSS parsing |
| `selectors` | CSS selector matching |
| `boa_engine` | JavaScript execution |
| `wgpu` | GPU rendering |
| `dwrote` | Text shaping (Windows) |
| `reqwest` | HTTP client |
| `tokio` | Async runtime |
| `image` | Image decoding |

---

## Threading Model

- **Main thread**: Event loop, Win32 messages
- **Layout thread**: Style/layout computation (planned)
- **Script thread**: JavaScript execution
- **Network pool**: Async HTTP requests
- **Compositor thread**: GPU operations (planned)

---

## Test Coverage

| Crate | Tests |
|-------|-------|
| rustkit-core | 27 |
| rustkit-dom | 48 |
| rustkit-css | 5 |
| rustkit-layout | 61 |
| rustkit-js | 11 |
| rustkit-bindings | 20 |
| rustkit-net | (varies) |
| rustkit-image | 17 |
| rustkit-animation | 12 |
| rustkit-svg | 12 |
| rustkit-canvas | 12 |
| rustkit-webgl | 10 |
| rustkit-common | 13 |
| rustkit-engine | 3 |
| rustkit-bench | 3 |
| rustkit-test | (harness) |
| **Total** | **250+** |

---

## Contributing

1. Pick a phase/task from the roadmap
2. Create a branch: `rustkit-phase-N-feature`
3. Implement with tests
4. Run `cargo clippy --workspace` (no warnings)
5. Run `cargo test --workspace`
6. Submit PR with acceptance test evidence

---

## Resources

- [WPT Tests](https://github.com/nicotordev/nicot.web-platform-tests)
- [CSS Specs](https://www.w3.org/Style/CSS/)
- [HTML Spec](https://html.spec.whatwg.org/)
- [DOM Spec](https://dom.spec.whatwg.org/)
- [Canvas 2D Spec](https://html.spec.whatwg.org/multipage/canvas.html)
- [WebGL Spec](https://www.khronos.org/registry/webgl/specs/latest/1.0/)
- [Servo Browser Engine](https://servo.org/) (architecture reference)
