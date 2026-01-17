use bevy::prelude::*;

/// A component that emits heat into the environment.
///
/// This is used by the thermal simulation system in `unthermal-plugin` to
/// update the temperature of the board.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ThermalEmitter {
    /// The temperature the emitter is trying to reach (in Kelvin).
    pub target_temp: f32,
    /// The power/strength of the heater/cooler.
    pub power: f32,
    /// If true, the effect is restricted to the same room as the emitter.
    pub room_restricted: bool,
}

/// A component that affects fluid pressure (e.g., miasma/fog).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct FluidEmitter {
    /// The pressure/influence this emitter contributes.
    pub pressure: f32,
}

/// A component that emits sound into the environment.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SoundEmitter {
    /// The volume/intensity of the sound being emitted.
    pub volume: f32,
}
