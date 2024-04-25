# bevy_curvo

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/mattatz/bevy_curvo#license)
[![Crates.io](https://img.shields.io/crates/v/bevy_curvo.svg)](https://crates.io/crates/bevy_curvo)
[![Docs](https://docs.rs/bevy_curvo/badge.svg)](https://docs.rs/bevy_curvo/latest/bevy_curvo/)
[![Test](https://github.com/mattatz/bevy_curvo/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/mattatz/bevy_curvo/actions/workflows/test.yml)

`bevy_curvo` is a helper library for rendering curves and surfaces modeled with [Curvo](https://github.com/mattatz/curvo) directly within the [Bevy](https://github.com/bevyengine/bevy) environment.

[Demo](https://github.com/mattatz/bevy_curvo/assets/1085910/5a6864c5-85fa-44ee-b194-23aaa8ff452a)

## Usage

```rust
// Create a set of points to interpolate
let points = vec![
    Point3::new(-1.0, -1.0, 0.),
    Point3::new(1.0, -1.0, 0.),
    Point3::new(1.0, 1.0, 0.),
    Point3::new(-1.0, 1.0, 0.),
];

// Create a NURBS curve that interpolates the given points with degree 3
let interpolated = NurbsCurve3D::<f64>::try_interpolate(&points, 3, None, None).unwrap();

// Create a NURBS surface by extruding the curve along the z-axis
let extrusion = NurbsSurface::extrude(&interpolated, Vector3::z() * 3.0);

// Create a SurfaceTessellation from the NURBS surface
let tess = extrusion.tessellate(Some(AdaptiveTessellationOptions {
    norm_tolerance: 1e-2 * 2.5,
    ..Default::default()
}));

// Create a bevy friendly data from the tessellation
let surface_mesh = NurbsSurfaceMesh::from(tess);

commands.spawn(PbrBundle {
  // Here you can use the mesh to render the surface
  mesh: surface_mesh,
  material: materials.add(StandardMaterial {
    ..default()
  }),
  ..default()
});

// or you can build a mesh for the surface
let tri: Mesh = surface_mesh.build_surface_triangle_list(Some(RenderAssetUsages::default()));

```

## Compatibility

| bevy | bevy_curvo |
| ---- | ---------- |
| 0.13 | 0.1        |
