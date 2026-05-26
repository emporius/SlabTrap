use slab_tessellator::{Point, triangulate_polygon};

fn mesh_area(mesh: &[Point]) -> f32 {
    mesh.chunks_exact(3)
        .map(|triangle| {
            let a = triangle[0];
            let b = triangle[1];
            let c = triangle[2];
            ((a.x * b.y - b.x * a.y) + (b.x * c.y - c.x * b.y) + (c.x * a.y - a.x * c.y)) * 0.5
        })
        .sum::<f32>()
        .abs()
}

#[test]
fn test_triangulate_simple_triangle() {
    let shape = vec![
        Point::new(0.0, 0.0),
        Point::new(4.0, 0.0),
        Point::new(0.0, 3.0),
    ];

    let mesh = triangulate_polygon(&[shape]);

    // A single triangle is decomposed as a slab (degenerate trapezoid), yielding 2 triangles (6 vertices)
    assert_eq!(mesh.len(), 6);
    assert!((mesh_area(&mesh) - 6.0).abs() < 0.001);
}

#[test]
fn test_triangulate_simple_square() {
    let shape = vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(0.0, 10.0),
    ];

    let mesh = triangulate_polygon(&[shape]);

    // Square should yield exactly 2 triangles (6 vertices)
    assert_eq!(mesh.len(), 6);
    assert!((mesh_area(&mesh) - 100.0).abs() < 0.001);
}

#[test]
fn test_triangulate_concave_shape() {
    // A concave L-shaped polygon
    let shape = vec![
        Point::new(0.0, 0.0),
        Point::new(4.0, 0.0),
        Point::new(4.0, 2.0),
        Point::new(2.0, 2.0),
        Point::new(2.0, 4.0),
        Point::new(0.0, 4.0),
    ];

    let mesh = triangulate_polygon(&[shape]);

    // Total area of the L-shape: 4 * 2 (bottom rect) + 2 * 2 (top rect) = 12.0
    assert!(!mesh.is_empty());
    assert_eq!(mesh.len() % 3, 0);
    assert!((mesh_area(&mesh) - 12.0).abs() < 0.01);
}

#[test]
fn test_triangulate_shape_with_hole() {
    // Outer square: 10x10, area = 100.0
    let outer = vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(0.0, 10.0),
    ];
    // Inner hole: 6x6, area = 36.0, opposite winding order
    let inner = vec![
        Point::new(2.0, 2.0),
        Point::new(2.0, 8.0),
        Point::new(8.0, 8.0),
        Point::new(8.0, 2.0),
    ];

    let mesh = triangulate_polygon(&[outer, inner]);

    // Expected area: 100.0 - 36.0 = 64.0
    assert!(!mesh.is_empty());
    assert_eq!(mesh.len() % 3, 0);
    assert!((mesh_area(&mesh) - 64.0).abs() < 0.01);
}
