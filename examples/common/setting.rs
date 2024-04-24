use bevy::ecs::{entity::Entity, system::Resource};

#[derive(Resource, Default)]
pub struct Setting {
    pub interpolate_curve_degree: usize,
    pub loft_curves_target: Vec<Entity>,
}
