use bevy::{ecs::component::Component, math::Vec3};
use curvo::prelude::NurbsCurve3D;

#[derive(Component, Default, Debug)]
pub struct InterpolateCurve {
    pub points: Vec<Vec3>,
}

#[derive(Component, Debug)]
pub struct ProfileCurve(pub NurbsCurve3D<f32>);

#[derive(Component, Debug)]
pub struct ExtrudeCurve(pub NurbsCurve3D<f32>);

#[derive(Component, Debug)]
pub struct SelectedCurve(pub NurbsCurve3D<f32>);
