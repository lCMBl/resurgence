use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::builtins::{
    TnuaBuiltinCrouch, TnuaBuiltinDash,
};
use bevy_tnua::control_helpers::{
    TnuaCrouchEnforcer, TnuaSimpleAirActionsCounter,
    TnuaSimpleFallThroughPlatformsHelper,
};
use bevy_tnua::prelude::*;
use bevy_tnua::{
    TnuaAnimatingState, TnuaGhostSensor,
    TnuaProximitySensor, TnuaToggle,
};
use bevy_tnua_rapier3d::*;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 16.0, 40.0)
            .looking_at(Vec3::new(0.0, 10.0, 3.0), Vec3::Y),
        ..Default::default()
    });

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(5.0, 5.0, 5.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.0,
            // For some reason in Bevy 0.12 shadows no longer work in WASM
            shadows_enabled: !cfg!(target_arch = "wasm32"),
            ..Default::default()
        },
        transform: Transform::default().looking_at(-Vec3::Y, Vec3::Z),
        ..Default::default()
    });
}

pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut cmd = commands.spawn_empty();

    cmd.insert(SpatialBundle {
        transform: Transform::from_xyz(0.0, 10.0, 0.0),
        ..Default::default()
    });

    cmd.insert(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Capsule {
            depth: 0.8,
            radius: 0.5,
            ..Default::default()
        })),
        material: materials.add(Color::ALICE_BLUE.into()),
        ..Default::default()
    });

    cmd.insert(RigidBody::Dynamic);
    cmd.insert(Collider::capsule_y(0.5, 0.5));
    cmd.insert(TnuaRapier3dIOBundle::default());
    cmd.insert(TnuaControllerBundle::default());

    cmd.insert(CharacterMotionConfigForPlatformerExample {
        speed: 20.0,
        walk: TnuaBuiltinWalk {
            float_height: 2.0,
            ..Default::default()
        },
        actions_in_air: 1,
        jump: TnuaBuiltinJump {
            height: 4.0,
            ..Default::default()
        },
        crouch: TnuaBuiltinCrouch {
            float_offset: -0.9,
            ..Default::default()
        },
        dash_distance: 10.0,
        dash: Default::default(),
    });
    cmd.insert(TnuaToggle::default());
    cmd.insert(TnuaCrouchEnforcer::new(0.5 * Vec3::Y, |cmd| {
        cmd.insert(TnuaRapier3dSensorShape(Collider::cylinder(0.0, 0.5)));
    }));
    cmd.insert(TnuaGhostSensor::default());
    cmd.insert(TnuaSimpleFallThroughPlatformsHelper::default());
    cmd.insert(TnuaSimpleAirActionsCounter::default());
    cmd.insert(FallingThroughControlScheme::default());
    cmd.insert(TnuaAnimatingState::<AnimationState>::default());
    
}

#[derive(Component)]
pub struct CharacterMotionConfigForPlatformerExample {
    pub speed: f32,
    pub walk: TnuaBuiltinWalk,
    pub actions_in_air: usize,
    pub jump: TnuaBuiltinJump,
    pub crouch: TnuaBuiltinCrouch,
    pub dash_distance: f32,
    pub dash: TnuaBuiltinDash,
}

#[derive(Component, Debug, PartialEq, Default)]
pub enum FallingThroughControlScheme {
    JumpThroughOnly,
    WithoutHelper,
    #[default]
    SingleFall,
    KeepFalling,
}

impl FallingThroughControlScheme {

    #[allow(dead_code)]
    pub fn perform_and_check_if_still_crouching(
        &self,
        crouch: bool,
        crouch_just_pressed: bool,
        fall_through_helper: &mut TnuaSimpleFallThroughPlatformsHelper,
        proximity_sensor: &mut TnuaProximitySensor,
        ghost_sensor: &TnuaGhostSensor,
        min_proximity: f32,
    ) -> bool {
        match self {
            FallingThroughControlScheme::JumpThroughOnly => {
                for ghost_platform in ghost_sensor.iter() {
                    if min_proximity <= ghost_platform.proximity {
                        proximity_sensor.output = Some(ghost_platform.clone());
                        break;
                    }
                }
                crouch
            }
            FallingThroughControlScheme::WithoutHelper => {
                for ghost_platform in ghost_sensor.iter() {
                    if min_proximity <= ghost_platform.proximity {
                        if crouch {
                            return false;
                        } else {
                            proximity_sensor.output = Some(ghost_platform.clone());
                        }
                    }
                }
                crouch
            }
            FallingThroughControlScheme::SingleFall => {
                let mut fall_through_helper =
                    fall_through_helper.with(proximity_sensor, ghost_sensor, min_proximity);
                if crouch {
                    !fall_through_helper.try_falling(crouch_just_pressed)
                } else {
                    fall_through_helper.dont_fall();
                    false
                }
            }
            FallingThroughControlScheme::KeepFalling => {
                let mut fall_through_helper =
                    fall_through_helper.with(proximity_sensor, ghost_sensor, min_proximity);
                if crouch {
                    !fall_through_helper.try_falling(true)
                } else {
                    fall_through_helper.dont_fall();
                    false
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum AnimationState {
    Standing,
    Running(f32),
    Jumping,
    Falling,
    Crouching,
    Crawling(f32),
    Dashing,
}