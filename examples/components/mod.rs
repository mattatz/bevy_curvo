use bevy::{ecs::component::Component, math::Vec3, utils::Uuid};
use curvo::prelude::NurbsCurve3D;

#[derive(Component, Default, Debug)]
pub struct InterpolateCurve {
    pub points: Vec<Vec3>,
}

#[derive(Component, Debug)]
pub struct ProfileCurve(pub (Uuid, NurbsCurve3D<f32>));

impl ProfileCurve {
    pub fn new(curve: NurbsCurve3D<f32>) -> Self {
        Self((Uuid::new_v4(), curve))
    }

    pub fn id(&self) -> Uuid {
        self.0 .0
    }

    pub fn curve(&self) -> &NurbsCurve3D<f32> {
        &self.0 .1
    }
}

#[derive(Component, Debug)]
pub struct ExtrudeCurve(pub NurbsCurve3D<f32>);

#[derive(Component, Debug)]
pub struct SelectedCurve(pub NurbsCurve3D<f32>);
