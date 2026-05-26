# Slab Trap 🪚

A lightweight, zero-dependency, and 2D polygon triangulation solution fully in Rust, based on horizontal slab (trapezoidal) decomposition for fast and robust post-boolean op rendering.

---

## Table of Contents

- [Features](#features)
- [Use Cases](#use-cases)
- [Installation](#installation)
- [Usage](#usage)
  - [Basic Example](#basic-example)
  - [Customizing Options](#customizing-options)
- [How It Works](#how-it-works)
  - [Core Concept: Horizontal Slabs](#core-concept-horizontal-slabs)
  - [Why Slab Tessellator](#why-slab-tessellator)
- [License](#license)

---

## Features

- **Zero dependencies**
- **Robustness**
- **Naturally handles nested holes and islands** without requiring complex polygon boundary nesting trees or hole-connection bridges.
- **Configurable grid and epsilon values** to adapt perfectly to your graphics model or application constraints.
- **Simple and readable** (~140 lines total and actual code is under 100 lines) for quick review and implementation into your custom editors or game engines.

---

## Use Cases

### When to Use

- Post boolean operations rendering where:
  - Triangle count/quality does not matter
  - Winding agnostic
  - Input cannot be controlled
- Where robustness, latency, and speed are vital

### Real-World Applications

- Art and Editing studios with boolean operations
- Game Engine: real-time procedural terrain generation
- Font rasterization
- Map Renderer: Filled Polygons

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
slab_tessellator = { path = "dev_docs/OSS" } # Or standard crates.io import when published
```

---

## Usage

### Basic Example

```rust
use slab_tessellator::{Point, triangulate_polygon};

fn main() {
    // Define a square contour (vertices must represent a closed loop)
    let outer = vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(0.0, 10.0),
    ];

    // Triangulate the shape (returns a flat list of Points in triangle-triple groupings)
    let triangles = triangulate_polygon(&[outer]);

    println!("Generated {} triangles!", triangles.len() / 3);
    for chunk in triangles.chunks_exact(3) {
        println!("Triangle: {:?}, {:?}, {:?}", chunk[0], chunk[1], chunk[2]);
    }
}
```

### Customizing Options

If you need a specific precision threshold or snap grid for complex geometry, use `triangulate_polygon_with_options`:

```rust
use slab_tessellator::{Point, TessellationOptions, triangulate_polygon_with_options};

let options = TessellationOptions {
    grid_unit: 0.001,       // Distance resolution to safely inset coordinate sampling
    epsilon: 0.000001,     // Float comparison epsilon
};

let triangles = triangulate_polygon_with_options(&[/* contours */], options);
```

---

## How It Works

### Core Concept: Horizontal Slabs

Traditional triangulation libraries (like `Lyon` or `iTriangle`) rely on **sweep-line algorithms** — a technique where an imaginary horizontal line scans from top to bottom, maintaining a dynamically updated data structure of active segments.

While elegant in theory, this approach demands thousands of lines of intricate, stateful code that is difficult to audit, debug, or maintain.

`slab_tessellator` makes a deliberate trade-off: it abandons $O(N \log N)$ worst-case complexity in favor of a **much simpler $O(M \cdot N)$ slab-decomposition approach** ($O(N^2)$ worst case, where $M$ is the number of unique Y-coordinates).

#### Algorithm Steps

1. **Slice:** All unique Y-coordinates of all vertices are collected and sorted to establish horizontal "slabs".
2. **Slab Invariance:** Since there are no vertices strictly *between* the Y-levels of a slab, all boundary segments crossing the slab are guaranteed to be straight, non-intersecting line segments.
3. **Sample and Match:** Coordinate intersections are sampled at the slab's top, bottom, and midpoints. The shape interior in each slab corresponds exactly to the alternating intervals between sorted X-coordinates.
4. **Triangulate:** Every active interval forms a simple trapezoid (or a triangle when a boundary width is zero). Then, each trapezoid is split into two triangles and pushed to the mesh.

For vector graphics, UI designs, and game paths (which typically have under 10,000 vertices), this approach runs in microseconds while being **under 100 lines of logic**!

---

## Why Slab Tessellator

This package was created because existing solutions in Rust couldn't efficiently handle the requirements of boolean operations. Well-known packages like Lyon or iTriangle assume controlled input and prioritize triangle quality, relying on complex sweep-line algorithms.

### Problem 1: Preprocessing Overhead

Lyon or iTriangle requires preprocessing:

1. Detect and fix winding — iterate all contours, compute signed area, flip if wrong
2. Remove duplicate/near-coincident vertices — O(n²) or requires spatial indexing
3. Classify rings as outer vs. hole — requires point-in-polygon tests across all ring pairs
4. Build the library's input format — Lyon's PathBuilder or iTriangle's integer coordinate system

**The Cost:** More latency between user input and output. In an art editor:

```
User action
  → Boolean op (Clipper2)        ~0.1–1ms
  → Preprocessing (if needed)    ~0.5–2ms  ← this is the problem
  → Triangulation                ~0.1–0.5ms
  → Upload to GPU                ~0.1ms
  → Render                       ~1ms
```

At 60fps you have 16ms total per frame. If preprocessing costs 2ms per operation and you're doing multiple operations, you've lost the frame before triangulation even starts. Zero preprocessing is critical for interactive applications.

**Boolean ops are rarely done once.** A user might:
- Select 10 shapes and union them all → 9 sequential boolean ops
- Use a "stamp" tool that subtracts a shape 30 times along a path
- Have a live boolean preview updating every pointer-move event at 60fps

### Problem 2: Robustness and Failure Modes

When Lyon encounters problematic geometry such as:
- Nearly-coincident vertices
- Zero-length edges
- Inconsistent winding

It either panics or produces incorrect geometry silently.

**The Solution:** A triple-sample check that simply skips bad bands. The result is a mesh with a microscopic gap nobody can see. This is far better than crashing or wasting resources on complex repairs.

### Problem 3: Winding Assumptions

**What Winding Is:**

Every polygon ring has a direction: vertices go either clockwise or counter-clockwise. Most triangulators use winding to determine inside vs. outside:

- Counter-clockwise = outer ring (filled)
- Clockwise = hole ring (empty)

**Why Boolean Ops Break This:**

Consider subtracting shape B from shape A. The boolean function computes the correct geometric result, but the output might have:

```
Ring 0: [p1, p2, p3, p4...]  ← outer boundary (CCW) ✅
Ring 1: [p5, p6, p7, p8...]  ← should be a hole, but came out CCW too ❌
```

Both rings are counter-clockwise. To Lyon, both look like outer fills — it would fill the hole solid instead of leaving it empty. The geometry is correct but the winding label is wrong.

This happens because boolean operations compute intersections and region membership, not winding consistency.

**The Solution:** Use the **even-odd rule**. Instead of asking "Is this ring a hole?", ask "How many edges of the shape did I cross to reach this X position?" This is winding-agnostic and works with any polygon configuration.

---

## License

Licensed under the [MIT License](LICENSE). Feel free to use, modify, and distribute this in personal or commercial projects.
