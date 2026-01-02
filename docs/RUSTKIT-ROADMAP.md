# RustKit Engine Roadmap

## Overview

RustKit is a Rust-native browser engine designed to replace WebKit's WinCairo port for the HiWave browser. This roadmap outlines the development phases beyond the initial MVP implementation.

**Current Status:** Phase 13 In Progress (Text Rendering & Fonts)
**Branch:** `rustkit`

---

## Completed Phases (1-11)

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

---

## Upcoming Phases (12-20)

### Phase 12: CSS Box Model & Visual Formatting ✅
**Priority:** Critical | **Status:** Complete

Complete the CSS box model implementation for proper layout:

- [x] **Margin collapse** - Implement vertical margin collapsing rules
- [x] **Padding & borders** - Full box model with correct painting order
- [x] **Positioned elements** - `relative`, `absolute`, `fixed`, `sticky`
- [x] **Float layout** - Basic float behavior and clearance
- [x] **Z-index stacking** - Stacking contexts and paint order

---

### Phase 13: Text Rendering & Fonts 🔄
**Priority:** Critical | **Status:** In Progress

Improve text rendering quality:

- [x] **Font fallback chain** - System font fallback for missing glyphs
- [x] **Unicode shaping** - Complex script support via DirectWrite
- [x] **Text decoration** - Underline, strikethrough, overline
- [x] **Line height** - Proper `line-height` calculation
- [x] **Font variants** - Bold, italic, weights, stretches
- [ ] **Web fonts** - `@font-face` loading (basic - placeholder)

**Acceptance Criteria:**
- Render mixed-language content (Latin + CJK)
- Pass WPT `css/css-text/` basic tests

---

### Phase 14: Event Handling
**Priority:** Critical | **Est. Duration:** 2 weeks

Complete input event pipeline:

- [ ] **Mouse events** - click, dblclick, mousedown/up/move, hover
- [ ] **Keyboard events** - keydown, keyup, keypress, input
- [ ] **Focus management** - Tab navigation, focus/blur events
- [ ] **Hit testing** - Accurate element hit detection
- [ ] **Event bubbling** - Capture and bubble phases
- [ ] **Touch events** - Basic touch support (pointer events)

**Acceptance Criteria:**
- Interactive buttons and links work
- Form inputs accept text input
- Tab navigation works correctly

---

### Phase 15: Forms & Input
**Priority:** High | **Est. Duration:** 2-3 weeks

Implement HTML form controls:

- [ ] **Text inputs** - `<input type="text">`, `<textarea>`
- [ ] **Buttons** - `<button>`, `<input type="submit/button">`
- [ ] **Checkboxes & radios** - State management
- [ ] **Select dropdowns** - `<select>` with option list
- [ ] **Form submission** - GET/POST with encoding
- [ ] **Validation** - HTML5 constraint validation

**Acceptance Criteria:**
- Login forms work (username/password/submit)
- Search boxes accept input and submit

---

### Phase 16: Images & Media
**Priority:** High | **Est. Duration:** 2 weeks

Support visual media:

- [ ] **Image decoding** - PNG, JPEG, GIF, WebP (via `image` crate)
- [ ] **Image sizing** - `width`, `height`, `object-fit`
- [ ] **Background images** - `background-image`, `background-size`
- [ ] **Lazy loading** - `loading="lazy"` support
- [ ] **Favicons** - Load and expose favicon to HiWave

**Acceptance Criteria:**
- Render pages with images correctly
- Background images display
- Favicons appear in tab UI

---

### Phase 17: CSS Flexbox
**Priority:** High | **Est. Duration:** 3 weeks

Implement Flexbox layout:

- [ ] **Flex container** - `display: flex`, direction, wrap
- [ ] **Flex items** - `flex-grow`, `flex-shrink`, `flex-basis`
- [ ] **Alignment** - `justify-content`, `align-items`, `align-self`
- [ ] **Order** - `order` property
- [ ] **Gap** - `gap`, `row-gap`, `column-gap`

**Acceptance Criteria:**
- Pass 40% of WPT `css/css-flexbox/` tests
- HiWave chrome UI layouts correctly

---

### Phase 18: Scrolling & Overflow
**Priority:** High | **Est. Duration:** 2 weeks

Implement scrolling:

- [ ] **Overflow handling** - `overflow: scroll/auto/hidden`
- [ ] **Scroll containers** - Proper scrolling regions
- [ ] **Smooth scroll** - `scroll-behavior: smooth`
- [ ] **Wheel events** - Mouse wheel scrolling
- [ ] **Scroll position** - Expose `scrollTop`, `scrollLeft`
- [ ] **Scroll snapping** - Basic scroll snap (optional)

**Acceptance Criteria:**
- Long pages scroll correctly
- Scrollable divs work
- Scroll position persists on resize

---

### Phase 19: Navigation & History
**Priority:** High | **Est. Duration:** 2 weeks

Complete navigation:

- [ ] **History API** - `pushState`, `replaceState`, `popstate`
- [ ] **Back/Forward** - Browser back/forward navigation
- [ ] **Hash navigation** - `#fragment` scrolling
- [ ] **Page lifecycle** - `beforeunload`, `unload` events
- [ ] **Navigation timing** - Performance.timing API

**Acceptance Criteria:**
- SPA navigation works
- Back/forward buttons work
- Hash links scroll to elements

---

### Phase 20: Security & Isolation
**Priority:** Critical | **Est. Duration:** 3 weeks

Security hardening:

- [ ] **Content Security Policy** - CSP header parsing & enforcement
- [ ] **Same-origin policy** - Cross-origin restrictions
- [ ] **CORS** - Preflight and response headers
- [ ] **Secure contexts** - HTTPS detection
- [ ] **Sandboxing** - Process isolation (future)

**Acceptance Criteria:**
- CSP blocks inline scripts when configured
- Cross-origin XHR/fetch blocked correctly
- CORS preflight works

---

## Future Phases (21+)

### Phase 21: CSS Grid
- Grid container and items
- Template rows/columns
- Grid areas
- Auto-placement

### Phase 22: Animations & Transitions
- CSS transitions
- CSS animations
- requestAnimationFrame
- Web Animations API

### Phase 23: SVG Support
- Basic SVG elements
- SVG rendering
- SVG filters (basic)

### Phase 24: Canvas 2D
- CanvasRenderingContext2D
- Drawing primitives
- Image manipulation

### Phase 25: Audio/Video
- `<audio>` and `<video>` elements
- Media controls
- Media Source Extensions (MSE)

### Phase 26: WebGL
- WebGL 1.0 context
- Shader compilation
- Texture loading

### Phase 27: Service Workers
- Registration and lifecycle
- Fetch interception
- Cache API

### Phase 28: IndexedDB
- Database creation
- Object stores
- Transactions

### Phase 29: WebRTC
- RTCPeerConnection
- MediaStream
- Data channels

### Phase 30: Accessibility
- ARIA attributes
- Screen reader support
- Focus indicators

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

## Architecture Decisions

### Crate Structure

```
rustkit-common     # Shared utilities, logging, retry
rustkit-viewhost   # Win32 window hosting
rustkit-compositor # GPU rendering
rustkit-core       # Task scheduling, navigation
rustkit-dom        # HTML parsing, DOM tree
rustkit-css        # CSS parsing, style computation
rustkit-layout     # Layout algorithms
rustkit-js         # JavaScript engine (Boa)
rustkit-bindings   # JS ↔ DOM bridge
rustkit-net        # HTTP, fetch, downloads
rustkit-engine     # Orchestration, multi-view
rustkit-test       # WPT harness
rustkit-bench      # Benchmarks
```

### Key Dependencies

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

### Threading Model

- **Main thread**: Event loop, Win32 messages
- **Layout thread**: Style/layout computation (planned)
- **Script thread**: JavaScript execution
- **Network pool**: Async HTTP requests
- **Compositor thread**: GPU operations (planned)

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
- [Servo Browser Engine](https://servo.org/) (architecture reference)

