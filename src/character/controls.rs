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
) {

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::W) {
        direction -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::S) {
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::A) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::D) {
        direction += Vec3::X;
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