use bevy::prelude::*;
use bevy_persistent::prelude::*;
use std::path::Path;

pub struct UnhaunterSettingsPlugin;

impl Plugin for UnhaunterSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(create_persistent::<crate::game::GameplaySettings>(
            "gameplay_settings.ron",
        ))
        .insert_resource(create_persistent::<crate::video::VideoSettings>(
            "video_settings.ron",
        ))
        .insert_resource(create_persistent::<crate::audio::AudioSettings>(
            "audio_settings.ron",
        ))
        .insert_resource(create_persistent::<crate::profile::ProfileSettings>(
            "profile_settings.ron",
        ))
        .insert_resource(create_persistent::<crate::controls::ControlKeys>(
            "control_settings.ron",
        ));
    }
}

fn create_persistent<
    T: serde::Serialize + serde::de::DeserializeOwned + Resource + Default + Send + Sync + 'static,
>(
    file_path: &str,
) -> Persistent<T> {
    let config_dir = dirs::config_dir()
        .map(|native_config_dir| native_config_dir.join("unhaunter-game").join("config"))
        .unwrap_or(Path::new("local").join("config"));

    Persistent::<T>::builder()
        .name(file_path.trim_end_matches(".ron"))
        .format(StorageFormat::RonPrettyWithStructNames)
        .path(config_dir.join(file_path))
        .default(T::default())
        .revertible(true)
        .revert_to_default_on_deserialization_errors(true)
        .build()
        .unwrap_or_else(|_| panic!("Could not read or create the file for '{file_path}'."))
}
