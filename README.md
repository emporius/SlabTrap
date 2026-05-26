# Slab Trap 🪚

A lightweight, zero-dependency, and 2D polygon triangulation solution fully in Rust, based on horizontal slab (trapezoidal) decomposition for fast and robust post-boolean op rendering. 

## Package features

- **Zero dependencies** 
- **Robustness**
- **Naturally handles nested holes and islands** without requiring complex polygon boundary nesting trees or hole-connection bridges.
- **Configurable grid and epsilon values** to adapt perfectly to your graphics model or application constraints.
- **Simple and readable** (~140 lines total and actual code is under 100 lines) for quick review and implementation into your custom editors or game engines.

---
## Use cases

- Post boolean operations rendering where 
    a) triangle count/quality does not matter
    b) winding agnostic 
    c) input cannot be controlled

- Where robustness, latency and speed are vital.

Real world use cases: 
    a) Art and Editing studios with boolean opeartions. 
    b) Game Engine: real time procedural terrain generation. 
    c) Font rasterisation 
    d) Map Renderer: Filled Polygons
---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
slab_tessellator = { path = "dev_docs/OSS" } # Or standard crates.io import when published
```

---

## Usage Example

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

---

## Customizing Options

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

## License

Licensed under the [MIT License](LICENSE). Feel free to use, modify, and distribute this in personal or commercial projects.

---

## Creator's notes

This package was created primarily because I could not find anything in Rust that could do boolean operations efficiently and quickly. I needed something that could create a fast, robust mesh from arbitrary 2D shapes in a vector editor studio. Basically, a situation where input cannot be controlled and where ultimately, traingle quality does not matter. 

Well-known packages like Lyon or iTriangle assume that input is controlled and that triangle quality would matter and rely on sweep-line algorithms. This is a technique where an imaginary horizontal line scans from top to bottom while maintaining a dynamically sorted list of "active" edges as it goes. To achieve $O(N \log N)$ worst-case efficiency, these libraries need sophisticated data structures (active-edge trees, double-connected edge lists) that track which edges are currently being swept and how they connect to each other. This is elegant but this approach demands thousands of lines of intricate, stateful code that is difficult to audit, debug, or maintain.


I had 3 main issues with other libraries: 

a) Preprocessing. 

Lyon or iTriangle requires preprocessing: 

1.	Detect and fix winding — iterate all contours, compute signed area, flip if wrong
2.	Remove duplicate/near-coincident vertices — O(n²) or requires spatial indexing
3.	Classify rings as outer vs. hole — requires point-in-polygon tests across all ring pairs
*** and also build the library’s input format — Lyon’s  PathBuilder or iTriangle’s integer coordinate system.

This means more latency between the user's input and output. For example, in an art editor: 

User action
  → Boolean op (Clipper2)        ~0.1–1ms
  → Preprocessing (if needed)    ~0.5–2ms  ← this is the problem
  → Triangulation                ~0.1–0.5ms
  → Upload to GPU                ~0.1ms
  → Render                       ~1ms

In the example of an art editor: 

Boolean ops are rarely done once. A user might:
•	Select 10 shapes and union them all → 9 sequential boolean ops
•	Use a “stamp” tool that subtracts a shape 30 times along a path
•	Have a live boolean preview updating every pointer-move event at 60fps
At 60fps you have 16ms total per frame. If preprocessing costs 2ms per op and you’re doing 10 ops, that’s 20ms. A frame is already missed before triangulation even starts. Zero preprocessing means that cost simply would not exist 

b) Breaking failure.

When Lyon encounters a problematic band such as two nearly-coincident vertices, a zero-length edge or inconsistent winding, it either panics or produces incorrect geometry silently. iTriangle is more robust but still expects topologically valid input.

A triple-sample check that simply skips bad bands is a far better solution. The result is a mesh with a microscopic gap nobody can see. This is better than crashing or wasting resources triangle-poking through the wrong region. rIn a real-time editor where the user is actively manipulating shapes, a skip is always preferable to a panic.


When Lyon encounters a problematic band, like say two nearly-coincident vertices, a zero-length edge or inconsistent winding, it either panics or produces incorrect geometry silently. iTriangle is more robust but still expects topologically valid input.  They also require a specific format before they can run, which in boolean terms, means needing to flatten and/or rasterise a shape before hand. It just felt terribly inefficient to me. 

c) Winding 

What Winding Is
Every polygon ring has a direction: the vertices go either clockwise or counter-clockwise. That direction is called winding, and most triangulators use it to determine inside vs. outside:

Counter-clockwise = outer ring (filled)
Clockwise         = hole ring (empty)

A simple shape like a rectangle is unambiguous. But the moment you have multiple rings, the triangulator needs to know which rings are which.

Why Boolean Ops Break This
Consider subtracting shape B from shape A. A function computes the correct geometric result — the right region is described. But the output might look like this:
Ring 0: [p1, p2, p3, p4...]  ← outer boundary (CCW) ✅
Ring 1: [p5, p6, p7, p8...]  ← should be a hole, but came out CCW too ❌

Both rings are counter-clockwise. To Lyon, both look like outer fills, it would fill the hole solid instead of leaving it empty. The geometry is correct but the winding label is wrong, and Lyon trusts the label.

This happens because a boolean operation function is computing intersections and region membership, not managing winding consistency. It guarantees the right points in the right order — not that every ring’s direction matches the convention a downstream library expects.

The thing is, you can sidestep all of this with a simple even-odd question. Instead of asking "Is this ring a hole?", we can simply ask "How many edges of the shape did I cross to reach this X position from the left?”. The winding direction of each ring is completely irrelevant to this count. A ring that’s clockwise or counter-clockwise contributes exactly one crossing either way. The nesting structure — outer, hole, island — falls out automatically from the parity of the crossing count, with zero classification work.



## Core Concept: Horizontal Slabs

Traditional triangulation libraries (like `Lyon` or `iTriangle`) rely on **sweep-line algorithms** — a technique where an imaginary horizontal line scans from top to bottom, maintaining a dynamically sorted list of "active" edges as it goes. To achieve $O(N \log N)$ worst-case efficiency, these libraries need sophisticated data structures (active-edge trees, double-connected edge lists) that track which edges are currently being swept and how they connect to each other.

While elegant in theory, this approach demands thousands of lines of intricate, stateful code that is difficult to audit, debug, or maintain.

`slab_tessellator` makes a deliberate trade-off: it abandons $O(N \log N)$ worst-case complexity in favor of a **much simpler $O(M \cdot N)$ slab-decomposition approach** ($O(N^2)$ worst case, where $M$ is the number of unique Y-coordinates and $N$ is the number of edges).

1. **Slice:** All unique Y-coordinates of all vertices are collected and sorted to establish horizontal "slabs".
2. **Slab Invariance:** Since there are no vertices strictly *between* the Y-levels of a slab, all boundary segments crossing the slab are guaranteed to be straight, non-intersecting line segments.
3. **Sample and Match:** Coordinate intersections are sampled at the slab's top, bottom, and midpoints. The shape interior in each slab corresponds exactly to the alternating intervals between sorted intersections.
4. **Triangulate:** Every active interval forms a simple trapezoid (or a triangle when a boundary width is zero). Then, each trapezoid is split into two triangles and pushed to the mesh.

For vector graphics, UI designs, and game paths (which typically have under 10,000 vertices), this approach runs in microseconds while being **under 100 lines of logic**! This beats using a big library and works in raw f32 approaches. However, for anything whre triangle count matters, such as for fills (not just strokes) or physics engines, it is better to use a full library like Lyon or iTriangle. 


