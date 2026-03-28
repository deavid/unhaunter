use bevy::prelude::*;
use unboard_core::entity::ResolutionFactor;
use unrender_std::board::tiledata::PreMesh;

/// Convert mapload placeholder mesh data into actual Mesh2d components.
///
/// This is render-only behavior and must live in a client-side presentation plugin.
fn process_pre_meshes(
    mut commands: Commands,
    query: Query<(Entity, &PreMesh, &ResolutionFactor)>,
    images: Res<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, pre_mesh, rf) in query.iter() {
        match pre_mesh {
            PreMesh::Mesh(mesh2d) => {
                commands
                    .entity(entity)
                    .insert(mesh2d.clone())
                    .remove::<PreMesh>();
            }
            PreMesh::Image {
                sprite_anchor,
                image_handle,
            } => {
                let Some(image) = images.get(image_handle) else {
                    debug!(
                        "process_pre_meshes: image handle not loaded yet for entity {:?}",
                        entity
                    );
                    continue;
                };

                let sz = image.texture_descriptor.size;
                trace!(
                    "Physical image size: {} x {} (Resolution Factor: {})",
                    sz.width, sz.height, rf.0
                );
                let sprite_size = Vec2::new(sz.width as f32, sz.height as f32);
                let sprite_anchor = Vec2::new(
                    sprite_size.x * sprite_anchor.x,
                    sprite_size.y * sprite_anchor.y,
                );

                let base_quad = Mesh::from(unrender_std::utils::quadcc::QuadCC::new(
                    sprite_size,
                    sprite_anchor,
                ));
                let mesh_handle = meshes.add(base_quad);
                let mesh2d = Mesh2d::from(mesh_handle);

                commands.entity(entity).insert(mesh2d).remove::<PreMesh>();
                trace!("Processed entity: {:?}", entity);
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, process_pre_meshes);
}
