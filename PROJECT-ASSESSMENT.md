# HiWave / RustKit Project Assessment

**Date:** January 2, 2026
**Branch:** `rustkit`

---

## Executive Summary

HiWave is building a Windows browser with a custom rendering engine called RustKit. 13 phases completed, 18 remaining. This document provides an honest assessment of custom vs third-party code.

---

## Project Metrics

| Metric | Value |
|--------|-------|
| RustKit engine code | ~10,700 lines Rust |
| HiWave app code | ~27,200 lines Rust |
| RustKit crates | 13 |
| Completed phases | 13 (0-12) |
| Remaining phases | 18 (13-30) |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│             Chrome WebView (WRY/WebView2)                   │
│  Tabs, toolbar, sidebar - stable, production-ready         │
├─────────────────────────────────────────────────────────────┤
│         Content Area: RustKit (Pure Rust Engine)            │
│    HTML → DOM → CSS → Layout → Paint → Composite            │
├─────────────────────────────────────────────────────────────┤
│           Shelf WebView (WRY/WebView2)                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Honest Assessment: Custom vs Third-Party

### Reality Check

**RustKit is an integration layer on top of heavyweight third-party libraries.** This is pragmatic, not a criticism.

### Custom Implementation (~20-30%)

| Component | Lines | What's Custom |
|-----------|-------|---------------|
| DOM Tree | ~400 | Node structure, traversal, queries |
| CSS System | ~700 | ComputedStyle, property parsing, inheritance |
| Layout Engine | ~1,400 | Box model, margin collapse, floats, z-index |
| Text Integration | ~1,000 | DirectWrite wrapper, font chains |
| Compositor | ~350 | Surface lifecycle, resize handling |
| JS-DOM Bridge | ~550 | window/document stubs, event registry |

### Third-Party Libraries (~70-80%)

| Function | Library | Provides |
|----------|---------|----------|
| HTML Parsing | `html5ever` | Full HTML5 algorithm |
| CSS Tokenizing | `cssparser` | CSS syntax parsing |
| JavaScript | `boa_engine` | Complete JS interpreter |
| Text Shaping | `dwrote`/DirectWrite | Unicode layout, fonts |
| GPU Rendering | `wgpu` | Cross-platform GPU |
| HTTP | `reqwest` | Full networking stack |
| Async | `tokio` | Task scheduling |
| UI Chrome | `wry`/WebView2 | Browser UI |

---

## Crate-by-Crate Analysis

### rustkit-dom (519 lines)
- **Custom:** Tree structure, traversal, query methods
- **Delegated:** html5ever does ALL parsing
- **Ratio:** ~85% is html5ever

### rustkit-css (719 lines)
- **Custom:** Color/length parsing, ComputedStyle (50+ properties), inheritance
- **Delegated:** cssparser available but lightly used
- **Ratio:** ~60% custom (simplified, no full cascade)

### rustkit-layout (2,565 lines)
- **Custom:** Box model, margin collapse, floats, positioning, stacking contexts
- **Delegated:** dwrote handles text measurement
- **Ratio:** ~70% custom — **most original work**

### rustkit-js (451 lines)
- **Custom:** Timer management, console shim
- **Delegated:** boa_engine does ALL JS execution
- **Ratio:** ~95% is boa

### rustkit-compositor (373 lines)
- **Custom:** Surface management, resize logic
- **Delegated:** wgpu does ALL GPU operations
- **Ratio:** ~90% is wgpu

### rustkit-bindings (563 lines)
- **Custom:** window/document stubs, location parsing
- **Note:** These are JS shims, not real DOM bindings
- **Ratio:** ~80% custom glue

---

## Strengths

1. **Smart Library Selection** - Using html5ever, boa, wgpu is correct
2. **Real Layout Work** - Margin collapse, floats, stacking are genuine spec work
3. **Practical Architecture** - WebView2 for UI, RustKit for content
4. **Clear Roadmap** - Phases 14-30 well-defined
5. **Clean Crate Structure** - 13 focused, testable crates

## Weaknesses

1. **CSS Parser Underused** - Simplified parser, no full cascade/specificity
2. **JS APIs Are Stubs** - `document.getElementById()` doesn't query real DOM
3. **No Rendering Pipeline** - Layout generates display list, but no painting
4. **Windows-Only** - Heavy DirectWrite dependency
5. **Not Connected** - DOM, styles, resources not wired together

---

## Phase Status

### Completed (0-12)

| Phase | Name | Status |
|-------|------|--------|
| 0 | Requirements | ✅ |
| 1 | Fork & Harness | ✅ |
| 2 | Engine Skeleton | ✅ |
| 3 | HTML/DOM/CSS/Layout | ✅ |
| 4 | JavaScript | ✅ |
| 5 | Networking | ✅ |
| 6 | Multi-view | ✅ |
| 7 | Observability | ✅ |
| 8 | HiWave Integration | ✅ |
| 9 | WPT Testing | ✅ |
| 10 | Benchmarks | ✅ |
| 11 | WinCairo Removal | ✅ |
| 12 | CSS Box Model | ✅ |

### In Progress (13)

**Phase 13: Text Rendering**
- [x] Font fallback chain
- [x] Unicode shaping via DirectWrite
- [x] Text decoration (underline, strikethrough)
- [x] Line height calculation
- [x] Font variants (bold, italic, weights)
- [ ] Web fonts (@font-face) - placeholder

### Upcoming (14-20)

#### Phase 14: Event Handling
- Mouse events (click, hover, etc.)
- Keyboard events
- Focus management
- Hit testing
- Event bubbling

#### Phase 15: Forms & Input
- Text inputs, buttons, checkboxes
- Select dropdowns
- Form submission
- HTML5 validation

#### Phase 16: Images & Media
- Image decoding (PNG, JPEG, WebP)
- Background images
- Lazy loading
- Favicons

#### Phase 17: CSS Flexbox
- Flex container/items
- Alignment properties
- Gap

#### Phase 18: Scrolling & Overflow
- Overflow handling
- Scroll containers
- Wheel events

#### Phase 19: Navigation & History
- History API
- Back/forward
- Hash navigation

#### Phase 20: Security & Isolation
- CSP
- Same-origin policy
- CORS

### Future (21-30)

CSS Grid, Animations, SVG, Canvas 2D, Audio/Video, WebGL, Service Workers, IndexedDB, WebRTC, Accessibility

---

## What's Missing for a Real Browser

| Category | Missing |
|----------|---------|
| CSS | Full cascade, specificity, @rules, calc(), media queries |
| Layout | Flexbox, Grid, tables, images |
| Rendering | GPU text painting, image compositing |
| Events | Complete event loop, hit testing |
| Resources | CSS/JS/image loading tied to DOM |
| Security | CSP, CORS, same-origin, sandboxing |
| Storage | Real localStorage, cookies, IndexedDB |

---

## Conclusion

### Current State

- ✅ Working HTML parser (html5ever)
- ✅ Basic CSS property system
- ✅ Layout engine with box model
- ✅ JavaScript interpreter (boa)
- ✅ GPU compositor framework (wgpu)
- ❌ No complete render pipeline
- ❌ No interactivity (events)
- ❌ No standards compliance (simplified CSS)
- ❌ Not secure

### Realistic Assessment

Building a browser engine is a multi-decade effort. Chrome has thousands of engineers. RustKit is pragmatic:

- Leverages Rust ecosystem effectively
- Windows-first with DirectWrite/DirectComposition
- ~70-80% of hard work done by libraries
- Custom value in integration and layout

### Path Forward

**Phases 14-20** would make a minimally viable renderer.
**Phases 21-30** would approach Servo's capabilities.

### Recommendation

Focus Phases 14-16 on completing the render pipeline end-to-end:
**events → DOM → style → layout → paint → composite**

A working simple page beats a broken complex one.

---

## Technology Stack

| Component | Technology |
|-----------|-----------|
| Browser Core | Rust + Tokio |
| HTML/DOM | html5ever |
| CSS | cssparser + custom |
| Layout | Custom Rust |
| JavaScript | Boa |
| GPU | wgpu + DirectComposition |
| Text | DirectWrite |
| Network | reqwest |
| UI Chrome | WebView2 (WRY) |
