use std::cmp::Ordering;

use bevy::{
    asset::Assets,
    ecs::system::{Commands, ResMut},
    math::{primitives::Plane3d, Ray3d, Vec3},
    pbr::MaterialMeshBundle,
    render::{
        color::Color,
        mesh::{Mesh, VertexAttributeValues},
    },
    transform::components::Transform,
};

use curvo::prelude::{NurbsCurve3D, Transformable};
use nalgebra::Point3;

use crate::{LineMaterial, ProfileCurve};

pub fn spawn_interp_curve(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    line_materials: &mut ResMut<Assets<LineMaterial>>,
    color: Color,
    points: &[Point3<f32>],
    degree: usize,
) {
    let interpolated = NurbsCurve3D::try_interpolate(points, degree, None, None).unwrap();
    spawn_curve(interpolated, commands, meshes, line_materials, None, color);
}

pub fn spawn_curve(
    curve: NurbsCurve3D<f32>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    line_materials: &mut ResMut<Assets<LineMaterial>>,
    transform: Option<Transform>,
    color: Color,
) {
    let mut line = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::LineStrip,
        Default::default(),
    );
    let line_vertices = curve
        .tessellate(Some(1e-4))
        .iter()
        .map(|p| [p.x, p.y, p.z])
        .collect();
    line.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(line_vertices),
    );
    commands.spawn((
        ProfileCurve(curve),
        MaterialMeshBundle {
            mesh: meshes.add(line),
            material: line_materials.add(LineMaterial { color }),
            transform: transform.unwrap_or_default(),
            ..Default::default()
        },
    ));
}

pub fn find_closest_curve<'a>(
    cursor_ray: Ray3d,
    curves: &[(&'a ProfileCurve, &'a Transform)],
    threshold: f32,
) -> Option<(&'a ProfileCurve, &'a Transform, NurbsCurve3D<f32>)> {
    let origin = Vec3::ZERO;
    let direction = Vec3::Y;
    let transformed = curves.iter().map(|c| {
        let transformed = c.0 .0.transformed(&c.1.compute_matrix().into());
        (c, transformed)
    });

    let closest = transformed
        .filter_map(|(c, transformed)| {
            let o = c.1.transform_point(origin);
            let d = c.1.rotation * direction;
            cursor_ray.intersect_plane(o, Plane3d::new(d)).map(|d| {
                let p = cursor_ray.get_point(d);
                let pt = Point3::from(p);
                let closest = transformed.closest_point(&pt);
                let distance = (closest - pt).norm();
                (distance, c, transformed)
            })
        })
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    closest.and_then(|(d, c, transformed)| {
        if d < threshold {
            Some((c.0, c.1, transformed))
        } else {
            None
        }
    })
}
