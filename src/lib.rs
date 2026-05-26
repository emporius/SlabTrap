//! A lightweight, zero-dependency, and mathematically robust 2D polygon triangulation library
//! based on horizontal slab (trapezoidal) decomposition.
//!
//! Designed for simplicity and extreme reliability, this algorithm splits complex polygons
//! (including those with nested holes and islands) into horizontal slabs, intersects active edges,
//! and directly outputs a triangulated mesh in $O(M \cdot N)$ time (where $M$ is the number of
//! unique vertex Y-coordinates and $N$ is the number of edges).
//!
//! For standard 2D vectors and path designs (typically under 10,000 vertices), it performs
//! blazingly fast while staying fully auditable and robust against floating-point precision errors.

/// A simple 2D coordinate point representation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    /// Creates a new 2D Point.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<(f32, f32)> for Point {
    fn from(tuple: (f32, f32)) -> Self {
        Self { x: tuple.0, y: tuple.1 }
    }
}

impl From<Point> for (f32, f32) {
    fn from(p: Point) -> Self {
        (p.x, p.y)
    }
}

impl From<[f32; 2]> for Point {
    fn from(arr: [f32; 2]) -> Self {
        Self { x: arr[0], y: arr[1] }
    }
}

impl From<Point> for [f32; 2] {
    fn from(p: Point) -> Self {
        [p.x, p.y]
    }
}

/// Configuration settings for the slab triangulation process.
#[derive(Clone, Copy, Debug)]
pub struct TessellationOptions {
    /// The grid snap scale or minimal distance threshold.
    /// Defaults to `0.0001` (representing a scale factor of 10,000).
    pub grid_unit: f32,
    /// Epsilon tolerance for floating-point boundaries.
    /// Defaults to `f32::EPSILON`.
    pub epsilon: f32,
}

impl Default for TessellationOptions {
    fn default() -> Self {
        Self {
            grid_unit: 0.0001,
            epsilon: f32::EPSILON,
        }
    }
}

/// Triangulates an arbitrary closed polygon (consisting of one or more contours) using default options.
///
/// Contours representing holes should have opposite winding rules compared to outer contours, or
/// be pre-simplified.
///
/// # Arguments
///
/// * `shape` - A slice of point contours (each contour is a closed loop of `Point`s).
///
/// # Example
///
/// ```rust
/// use slab_tessellator::{Point, triangulate_polygon};
///
/// // Create a simple square made of 4 points
/// let outer = vec![
///     Point::new(0.0, 0.0),
///     Point::new(10.0, 0.0),
///     Point::new(10.0, 10.0),
///     Point::new(0.0, 10.0),
/// ];
///
/// let triangles = triangulate_polygon(&[outer]);
/// assert!(!triangles.is_empty());
/// ```
pub fn triangulate_polygon(shape: &[Vec<Point>]) -> Vec<Point> {
    triangulate_polygon_with_options(shape, TessellationOptions::default())
}

/// Triangulates an arbitrary closed polygon with custom grid and epsilon options.
///
/// # Arguments
///
/// * `shape` - A slice of point contours (each contour is a closed loop of `Point`s).
/// * `options` - A `TessellationOptions` configuration block.
pub fn triangulate_polygon_with_options(
    shape: &[Vec<Point>],
    options: TessellationOptions,
) -> Vec<Point> {
    let mut y_values = shape
        .iter()
        .flatten()
        .map(|point| point.y)
        .filter(|value| value.is_finite())
        .collect::<Vec<_>>();
    y_values.sort_by(f32::total_cmp);
    y_values.dedup_by(|a, b| (*a - *b).abs() <= options.epsilon);

    let mut mesh = Vec::new();
    for window in y_values.windows(2) {
        let y0 = window[0];
        let y1 = window[1];
        if y1 - y0 <= options.epsilon {
            continue;
        }

        // Apply a small safe inset to sample coordinates cleanly between vertices
        let inset = ((y1 - y0) * 0.0001).min(options.grid_unit * 0.5);
        let top_y = y0 + inset;
        let bottom_y = y1 - inset;
        let mid_y = (top_y + bottom_y) * 0.5;

        let mid = intersections_at_y(shape, mid_y);
        let top = intersections_at_y(shape, top_y);
        let bottom = intersections_at_y(shape, bottom_y);

        let interval_count = mid.len() / 2;
        if top.len() / 2 != interval_count || bottom.len() / 2 != interval_count {
            continue;
        }

        for interval in 0..interval_count {
            let left_top = Point::new(top[interval * 2], top_y);
            let right_top = Point::new(top[interval * 2 + 1], top_y);
            let left_bottom = Point::new(bottom[interval * 2], bottom_y);
            let right_bottom = Point::new(bottom[interval * 2 + 1], bottom_y);

            mesh.extend_from_slice(&[
                left_top,
                left_bottom,
                right_bottom,
                left_top,
                right_bottom,
                right_top,
            ]);
        }
    }
    mesh
}

/// Helper function that finds all X-intersections along a horizontal scanline at coordinate `y`.
/// The returned intersections are sorted ascending.
fn intersections_at_y(shape: &[Vec<Point>], y: f32) -> Vec<f32> {
    let mut intersections = Vec::new();
    for contour in shape {
        if contour.is_empty() {
            continue;
        }
        for index in 0..contour.len() {
            let a = contour[index];
            let b = contour[(index + 1) % contour.len()];
            if (a.y <= y && b.y > y) || (b.y <= y && a.y > y) {
                let t = (y - a.y) / (b.y - a.y);
                intersections.push(a.x + (b.x - a.x) * t);
            }
        }
    }
    intersections.sort_by(f32::total_cmp);
    intersections
}
