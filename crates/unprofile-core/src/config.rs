use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Default)]
pub struct ProfileConfig {
    pub installation_id_file: Option<String>,
}
