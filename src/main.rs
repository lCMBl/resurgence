use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_rapier3d::*;

use bevy_panorbit_camera::PanOrbitCameraPlugin;

mod character;
mod map;

use character::{setup_camera, setup_player, apply_controls,};
use map::setup_level;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(TnuaRapier3dPlugin)
        .add_plugins(TnuaControllerPlugin)
        .add_plugins(PanOrbitCameraPlugin)

        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_level)
        .add_systems(Startup, setup_player)
        .add_systems(Update, apply_controls.in_set(TnuaUserControlsSystemSet))
        .run();
}
