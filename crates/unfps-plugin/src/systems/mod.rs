#![cfg(not(target_arch = "wasm32"))]

mod limit;

use bevy::prelude::*;

pub(crate) fn app_setup(app: &mut App) {
    limit::app_setup(app);
}
