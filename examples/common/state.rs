use bevy::ecs::schedule::States;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Idle,
    InterpolateCurve,
    ExtrudeCurve,
    LoftCurves,
    Select,
}
