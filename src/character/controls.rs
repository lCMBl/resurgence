use bevy::prelude::*;
use bevy_tnua::builtins::{
    TnuaBuiltinCrouch, TnuaBuiltinCrouchState, TnuaBuiltinDash,
};
use bevy_tnua::control_helpers::{
    TnuaCrouchEnforcer, TnuaSimpleAirActionsCounter,
    TnuaSimpleFallThroughPlatformsHelper,
};
use bevy_tnua::prelude::*;
use bevy_tnua::{
    TnuaGhostSensor,
    TnuaProximitySensor,
};

use super::{
    CharacterMotionConfigForPlatformerExample,
    FallingThroughControlScheme,
    PlayerCamTarget,
    PlayerFollowCam,
};

use bevy_panorbit_camera::PanOrbitCamera;

pub fn apply_controls(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(
        &CharacterMotionConfigForPlatformerExample,
        &mut TnuaController,
        &mut TnuaCrouchEnforcer,
        &mut TnuaProximitySensor,
        &TnuaGhostSensor,
        &mut TnuaSimpleFallThroughPlatformsHelper,
        &FallingThroughControlScheme,
        &mut TnuaSimpleAirActionsCounter,
    )>,
    cam_query: Query<&Transform, With<PlayerFollowCam>>,
) {

    let mut direction = Vec3::ZERO;

    let mut flat_forward = -Vec3::Z;
    let mut flat_right = Vec3::X;

    if let Ok(cam_transform) = cam_query.get_single() {
        flat_forward = cam_transform.forward();
        flat_forward.y = 0.0;
        flat_forward = flat_forward.normalize();
        
        flat_right = cam_transform.right();
        flat_right.y = 0.0;
        flat_right = flat_right.normalize();
    }

    if keyboard.pressed(KeyCode::W) {
        direction += flat_forward;
    }
    if keyboard.pressed(KeyCode::S) {
        direction -= flat_forward;
    }
    if keyboard.pressed(KeyCode::A) {
        direction -= flat_right;
    }
    if keyboard.pressed(KeyCode::D) {
        direction += flat_right;
    }



    direction = direction.clamp_length_max(1.0);

    let jump = keyboard.pressed(KeyCode::Space);
    let dash = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    let turn_in_place = keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);

    let crouch_buttons = [KeyCode::ControlLeft, KeyCode::ControlRight];
    let crouch = keyboard.any_pressed(crouch_buttons);
    let crouch_just_pressed = keyboard.any_just_pressed(crouch_buttons);

    for (
        config,
        mut controller,
        mut crouch_enforcer,
        mut sensor,
        ghost_sensor,
        mut fall_through_helper,
        falling_through_control_scheme,
        mut air_actions_counter,
    ) in query.iter_mut()
    {
        air_actions_counter.update(controller.as_mut());

        let crouch = falling_through_control_scheme.perform_and_check_if_still_crouching(
            crouch,
            crouch_just_pressed,
            fall_through_helper.as_mut(),
            sensor.as_mut(),
            ghost_sensor,
            1.0,
        );

        let speed_factor =
            if let Some((_, state)) = controller.concrete_action::<TnuaBuiltinCrouch>() {
                if matches!(state, TnuaBuiltinCrouchState::Rising) {
                    1.0
                } else {
                    0.2
                }
            } else {
                1.0
            };

        controller.basis(TnuaBuiltinWalk {
            desired_velocity: if turn_in_place {
                Vec3::ZERO
            } else {
                direction * speed_factor * config.speed
            },
            desired_forward: direction.normalize_or_zero(),
            ..config.walk.clone()
        });

        if crouch {
            controller.action(crouch_enforcer.enforcing(config.crouch.clone()));
        }

        if jump {
            controller.action(TnuaBuiltinJump {
                allow_in_air: air_actions_counter.air_count_for(TnuaBuiltinJump::NAME)
                    <= config.actions_in_air,
                ..config.jump.clone()
            });
        }

        if dash {
            controller.action(TnuaBuiltinDash {
                displacement: direction.normalize() * config.dash_distance,
                desired_forward: direction.normalize(),
                allow_in_air: air_actions_counter.air_count_for(TnuaBuiltinDash::NAME)
                    <= config.actions_in_air,
                ..config.dash.clone()
            });
        }
    }
}

pub fn camera_follow(
    player_query: Query<&Transform, With<PlayerCamTarget>>,
    mut camera_query: Query<&mut PanOrbitCamera, With<PlayerFollowCam>>,
) {
    if let Ok(player) = player_query.get_single() {
        if let Ok(mut camera) = camera_query.get_single_mut() {
            camera.target_focus = player.translation;
            // camera.target_radius = 15.0; // target radius only necessary if zoom is disabled.
        }
    }
}