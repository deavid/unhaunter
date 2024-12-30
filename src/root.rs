use bevy::{prelude::*, render::render_asset::RenderAssetUsages};

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum State {
    #[default]
    Loading,
    MainMenu,
    InGame,
    Summary,
    MapHub,
    UserManual,
    PreplayManual,
}

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    None,
    Truck,
    Pause,
    NpcHelp,
}

#[derive(Debug, Clone)]
pub struct LondrinaFontAssets {
    pub w100_thin: Handle<Font>,
    pub w300_light: Handle<Font>,
    pub w400_regular: Handle<Font>,
    pub w900_black: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct SyneFontAssets {
    pub w400_regular: Handle<Font>,
    pub w500_medium: Handle<Font>,
    pub w600_semibold: Handle<Font>,
    pub w700_bold: Handle<Font>,
    pub w800_extrabold: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct OverlockFontAssets {
    pub w400_regular: Handle<Font>,
    pub w700_bold: Handle<Font>,
    pub w900_black: Handle<Font>,
    pub w400i_regular: Handle<Font>,
    pub w700i_bold: Handle<Font>,
    pub w900i_black: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct ChakraPetchAssets {
    pub w300_light: Handle<Font>,
    pub w400_regular: Handle<Font>,
    pub w500_medium: Handle<Font>,
    pub w600_semibold: Handle<Font>,
    pub w700_bold: Handle<Font>,
    pub w300i_light: Handle<Font>,
    pub w400i_regular: Handle<Font>,
    pub w500i_medium: Handle<Font>,
    pub w600i_semibold: Handle<Font>,
    pub w700i_bold: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct TitilliumWebAssets {
    pub w200_extralight: Handle<Font>,
    pub w300_light: Handle<Font>,
    pub w400_regular: Handle<Font>,
    pub w600_semibold: Handle<Font>,
    pub w700_bold: Handle<Font>,
    pub w900_black: Handle<Font>,
    pub w200i_extralight: Handle<Font>,
    pub w300i_light: Handle<Font>,
    pub w400i_regular: Handle<Font>,
    pub w600i_semibold: Handle<Font>,
    pub w700i_bold: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct VictorMonoAssets {
    pub w100_thin: Handle<Font>,
    pub w200_extralight: Handle<Font>,
    pub w300_light: Handle<Font>,
    pub w400_regular: Handle<Font>,
    pub w500_medium: Handle<Font>,
    pub w600_semibold: Handle<Font>,
    pub w700_bold: Handle<Font>,
    pub w100i_thin: Handle<Font>,
    pub w200i_extralight: Handle<Font>,
    pub w300i_light: Handle<Font>,
    pub w400i_regular: Handle<Font>,
    pub w500i_medium: Handle<Font>,
    pub w600i_semibold: Handle<Font>,
    pub w700i_bold: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct KodeMonoAssets {
    pub w400_regular: Handle<Font>,
    pub w500_medium: Handle<Font>,
    pub w600_semibold: Handle<Font>,
    pub w700_bold: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct FontAssets {
    pub londrina: LondrinaFontAssets,
    pub syne: SyneFontAssets,
    pub overlock: OverlockFontAssets,
    pub chakra: ChakraPetchAssets,
    pub titillium: TitilliumWebAssets,
    pub victormono: VictorMonoAssets,
    pub kodemono: KodeMonoAssets,
}

#[derive(Debug, Clone)]
pub struct ImageAssets {
    pub title: Handle<Image>,
    pub character1: Handle<Image>,
    pub gear: Handle<Image>,
    pub character1_atlas: Handle<TextureAtlasLayout>,
    pub gear_atlas: Handle<TextureAtlasLayout>,
    pub vignette: Handle<Image>,
    // --- Manual Images ---
    // Chapter 1: Page 1:
    pub manual_investigate: Handle<Image>,
    pub manual_locate_ghost: Handle<Image>,
    pub manual_identify_ghost: Handle<Image>,
    pub manual_craft_repellent: Handle<Image>,
    pub manual_expel_ghost: Handle<Image>,
    pub manual_end_mission: Handle<Image>,
    // Chapter 1: Page 2:
    pub manual_movement_wasd: Handle<Image>,
    pub manual_interacting_objects: Handle<Image>,
    pub manual_flashlight: Handle<Image>,
    pub manual_activate_equipment: Handle<Image>,
    pub manual_switch_item: Handle<Image>,
    pub manual_quick_evidence: Handle<Image>,
    // Chapter 1: Page 3:
    pub manual_emf_reader: Handle<Image>,
    pub manual_thermometer: Handle<Image>,
    pub manual_truck_sanity: Handle<Image>,
    pub manual_ghost_attack: Handle<Image>,
    pub manual_truck_journal: Handle<Image>,
    pub manual_truck_exterior: Handle<Image>,
    // --- Chapter 2 images ---
    pub manual_left_hand_videocam: Handle<Image>,
    pub manual_uv_ghost: Handle<Image>,
    pub manual_uv_object: Handle<Image>,
    pub manual_uv_breach: Handle<Image>,
    pub manual_floating_orbs: Handle<Image>,
    pub manual_inventory_all: Handle<Image>,
    pub manual_ghost_red: Handle<Image>,
    pub manual_ghost_roar: Handle<Image>,
    pub manual_hide_table: Handle<Image>,
    pub manual_truck_loadout: Handle<Image>,
    pub manual_truck_endmission: Handle<Image>,
    pub manual_truck_refuge: Handle<Image>,
    // -- Chapter 3 Images
    pub manual_recorder_evp: Handle<Image>,
    pub manual_geiger_counter: Handle<Image>,
    pub manual_locating_ghost: Handle<Image>,
    pub manual_sanity_management: Handle<Image>,
    pub manual_emf_fluctuations: Handle<Image>,
    // -- Chapter 4 Images
    pub manual_object_interaction: Handle<Image>,
    pub manual_object_interaction_2: Handle<Image>,
    pub manual_spirit_box: Handle<Image>,
    pub manual_red_torch: Handle<Image>,
    // -- Chapter 5 Images
    pub manual_salt: Handle<Image>,
    pub manual_quartz: Handle<Image>,
    pub manual_sage: Handle<Image>,
}

#[derive(Debug, Clone)]
pub struct Anchors {
    pub base: Vec2,
    pub grid1x1: Vec2,
    pub grid1x1x4: Vec2,
    pub character: Vec2,
}

impl Anchors {
    /// Computes the anchors for the given sprite in pixels
    pub fn calc(pos_x: i32, pos_y: i32, size_x: i32, size_y: i32) -> Vec2 {
        Anchors::calc_f32(pos_x as f32, pos_y as f32, size_x as f32, size_y as f32)
    }

    /// Computes the anchors for the given sprite in pixels, f32 variant
    pub fn calc_f32(pos_x: f32, pos_y: f32, size_x: f32, size_y: f32) -> Vec2 {
        let x = pos_x / size_x - 0.5;
        let y = 0.5 - pos_y / size_y;
        Vec2::new(x, y)
    }
}

/// A rectangle on the `XY` plane with custom center.
#[derive(Debug, Copy, Clone)]
pub struct QuadCC {
    /// Full width and height of the rectangle.
    pub size: Vec2,
    /// Horizontally-flip the texture coordinates of the resulting mesh.
    pub flip: bool,
    /// Center point of the quad
    pub center: Vec2,
}

impl Default for QuadCC {
    fn default() -> Self {
        QuadCC::new(Vec2::ONE, Vec2::default())
    }
}

impl QuadCC {
    pub fn new(size: Vec2, center: Vec2) -> Self {
        Self {
            size,
            flip: false,
            center,
        }
    }
}

impl From<QuadCC> for Mesh {
    fn from(quad: QuadCC) -> Self {
        let left_x = -quad.center.x;
        let right_x = quad.size.x - quad.center.x;
        let bottom_y = quad.center.y - quad.size.y;
        let top_y = quad.center.y;
        let (u_left, u_right) = if quad.flip { (1.0, 0.0) } else { (0.0, 1.0) };
        let vertices = [
            ([left_x, bottom_y, 0.0], [0.0, 0.0, 1.0], [u_left, 1.0]),
            ([left_x, top_y, 0.0], [0.0, 0.0, 1.0], [u_left, 0.0]),
            ([right_x, top_y, 0.0], [0.0, 0.0, 1.0], [u_right, 0.0]),
            ([right_x, bottom_y, 0.0], [0.0, 0.0, 1.0], [u_right, 1.0]),
        ];
        let indices = bevy::render::mesh::Indices::U32(vec![0, 2, 1, 0, 3, 2]);
        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();
        let mut mesh = Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        );
        mesh.insert_indices(indices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}

#[derive(Debug, Clone, Resource)]
pub struct GameAssets {
    pub images: ImageAssets,
    pub fonts: FontAssets,
    pub anchors: Anchors,
}

pub fn load_assets(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let ch1 = "manual/images/chapter1";
    let ch2 = "manual/images/chapter2";
    let ch3 = "manual/images/chapter3";
    let ch4 = "manual/images/chapter4";
    let ch5 = "manual/images/chapter5";
    let ch1 = |path: &str| -> Handle<Image> { server.load(format!("{ch1}/{path}")) };
    let ch2 = |path: &str| -> Handle<Image> { server.load(format!("{ch2}/{path}")) };
    let ch3 = |path: &str| -> Handle<Image> { server.load(format!("{ch3}/{path}")) };
    let ch4 = |path: &str| -> Handle<Image> { server.load(format!("{ch4}/{path}")) };
    let ch5 = |path: &str| -> Handle<Image> { server.load(format!("{ch5}/{path}")) };

    commands.insert_resource(GameAssets {
        images: ImageAssets {
            title: server.load("img/title.png"),
            character1: server.load("img/characters-model1-demo.png"),
            gear: server.load("img/gear_spritesheetA_48x48.png"),
            character1_atlas: texture_atlases.add(TextureAtlasLayout::from_grid(
                UVec2::new(32 * 2, 32 * 2),
                16,
                4,
                Some(UVec2::new(0, 0)),
                Some(UVec2::new(0, 0)),
            )),
            gear_atlas: texture_atlases.add(TextureAtlasLayout::from_grid(
                UVec2::new(48 * 2, 48 * 2),
                10,
                10,
                Some(UVec2::new(0, 0)),
                Some(UVec2::new(0, 0)),
            )),
            vignette: server.load("img/vignette.png"),
            // --- Manual Images ---
            // Chapter 1: Page 1:
            manual_investigate: ch1("investigate.png"),
            manual_locate_ghost: ch1("locate_ghost.png"),
            manual_identify_ghost: ch1("identify_ghost.png"),
            manual_craft_repellent: ch1("craft_repellent.png"),
            manual_expel_ghost: ch1("expel_ghost.png"),
            manual_end_mission: ch1("end_mission.png"),
            // Chapter 1: Page 2:
            manual_movement_wasd: ch1("movement_wasd.png"),
            manual_interacting_objects: ch1("interacting_objects.png"),
            manual_flashlight: ch1("flashlight.png"),
            manual_activate_equipment: ch1("activate_equipment.png"),
            manual_switch_item: ch1("switch_item.png"),
            manual_quick_evidence: ch1("quick_evidence.png"),
            // Chapter 1: Page 3:
            manual_emf_reader: ch1("emf_reader.png"),
            manual_thermometer: ch1("thermometer.png"),
            manual_truck_sanity: ch1("truck_sanity.png"),
            manual_ghost_attack: ch1("ghost_attack.png"),
            manual_truck_journal: ch1("identify_ghost.png"),
            manual_truck_exterior: ch1("truck_exterior.png"),
            // -- Chapter 2 images --
            manual_left_hand_videocam: ch2("left_hand_videocam.png"),
            manual_uv_ghost: ch2("uv_ghost.png"),
            manual_uv_object: ch2("uv_object.png"),
            manual_uv_breach: ch2("uv_breach.png"),
            manual_floating_orbs: ch2("floating_orbs.png"),
            manual_inventory_all: ch2("inventory_all.png"),
            manual_ghost_red: ch2("ghost_red.png"),
            manual_ghost_roar: ch2("ghost_roar.png"),
            manual_hide_table: ch2("hide_table.png"),
            manual_truck_loadout: ch2("truck_loadout.png"),
            manual_truck_endmission: ch2("truck_endmission.png"),
            manual_truck_refuge: ch2("truck_refuge.png"),
            // -- Chapter 3 images --
            manual_recorder_evp: ch3("recorder_evp.png"),
            manual_geiger_counter: ch3("geiger_counter.png"),
            manual_locating_ghost: ch2("ghost_red.png"),
            manual_sanity_management: ch3("sanity_management.png"),
            manual_emf_fluctuations: ch1("emf_reader.png"),
            // -- Chapter 4 images --
            manual_object_interaction: ch4("object_interaction.png"),
            manual_object_interaction_2: ch4("object_interaction_2.png"),
            manual_spirit_box: ch4("spirit_box.png"),
            manual_red_torch: ch4("red_torch.png"),
            // -- Chapter 5 images --
            manual_salt: ch5("salt.png"),
            manual_quartz: ch5("quartz.png"),
            manual_sage: ch5("sage.png"),
        },
        fonts: FontAssets {
            londrina: LondrinaFontAssets {
                w100_thin: server.load("fonts/londrina_solid/LondrinaSolid-Thin.ttf"),
                w300_light: server.load("fonts/londrina_solid/LondrinaSolid-Light.ttf"),
                w400_regular: server.load("fonts/londrina_solid/LondrinaSolid-Regular.ttf"),
                w900_black: server.load("fonts/londrina_solid/LondrinaSolid-Black.ttf"),
            },
            syne: SyneFontAssets {
                w400_regular: server.load("fonts/syne/static/Syne-Regular.ttf"),
                w500_medium: server.load("fonts/syne/static/Syne-Medium.ttf"),
                w600_semibold: server.load("fonts/syne/static/Syne-SemiBold.ttf"),
                w700_bold: server.load("fonts/syne/static/Syne-Bold.ttf"),
                w800_extrabold: server.load("fonts/syne/static/Syne-ExtraBold.ttf"),
            },
            overlock: OverlockFontAssets {
                w400_regular: server.load("fonts/overlock/Overlock-Regular.ttf"),
                w700_bold: server.load("fonts/overlock/Overlock-Bold.ttf"),
                w900_black: server.load("fonts/overlock/Overlock-Black.ttf"),
                w400i_regular: server.load("fonts/overlock/Overlock-Italic.ttf"),
                w700i_bold: server.load("fonts/overlock/Overlock-BoldItalic.ttf"),
                w900i_black: server.load("fonts/overlock/Overlock-BlackItalic.ttf"),
            },
            chakra: ChakraPetchAssets {
                w300_light: server.load("fonts/chakra_petch/ChakraPetch-Light.ttf"),
                w400_regular: server.load("fonts/chakra_petch/ChakraPetch-Regular.ttf"),
                w500_medium: server.load("fonts/chakra_petch/ChakraPetch-Medium.ttf"),
                w600_semibold: server.load("fonts/chakra_petch/ChakraPetch-SemiBold.ttf"),
                w700_bold: server.load("fonts/chakra_petch/ChakraPetch-Bold.ttf"),
                w300i_light: server.load("fonts/chakra_petch/ChakraPetch-LightItalic.ttf"),
                w400i_regular: server.load("fonts/chakra_petch/ChakraPetch-Italic.ttf"),
                w500i_medium: server.load("fonts/chakra_petch/ChakraPetch-MediumItalic.ttf"),
                w600i_semibold: server.load("fonts/chakra_petch/ChakraPetch-SemiBoldItalic.ttf"),
                w700i_bold: server.load("fonts/chakra_petch/ChakraPetch-BoldItalic.ttf"),
            },
            titillium: TitilliumWebAssets {
                w200_extralight: server.load("fonts/titillium_web/TitilliumWeb-ExtraLight.ttf"),
                w300_light: server.load("fonts/titillium_web/TitilliumWeb-Light.ttf"),
                w400_regular: server.load("fonts/titillium_web/TitilliumWeb-Regular.ttf"),
                w600_semibold: server.load("fonts/titillium_web/TitilliumWeb-SemiBold.ttf"),
                w700_bold: server.load("fonts/titillium_web/TitilliumWeb-Bold.ttf"),
                w900_black: server.load("fonts/titillium_web/TitilliumWeb-Black.ttf"),
                w200i_extralight: server
                    .load("fonts/titillium_web/TitilliumWeb-ExtraLightItalic.ttf"),
                w300i_light: server.load("fonts/titillium_web/TitilliumWeb-LightItalic.ttf"),
                w400i_regular: server.load("fonts/titillium_web/TitilliumWeb-Italic.ttf"),
                w600i_semibold: server.load("fonts/titillium_web/TitilliumWeb-SemiBoldItalic.ttf"),
                w700i_bold: server.load("fonts/titillium_web/TitilliumWeb-BoldItalic.ttf"),
            },
            victormono: VictorMonoAssets {
                w100_thin: server.load("fonts/victor_mono/static/VictorMono-Thin.ttf"),
                w200_extralight: server.load("fonts/victor_mono/static/VictorMono-ExtraLight.ttf"),
                w300_light: server.load("fonts/victor_mono/static/VictorMono-Light.ttf"),
                w400_regular: server.load("fonts/victor_mono/static/VictorMono-Regular.ttf"),
                w500_medium: server.load("fonts/victor_mono/static/VictorMono-Medium.ttf"),
                w600_semibold: server.load("fonts/victor_mono/static/VictorMono-SemiBold.ttf"),
                w700_bold: server.load("fonts/victor_mono/static/VictorMono-Bold.ttf"),
                w100i_thin: server.load("fonts/victor_mono/static/VictorMono-ThinItalic.ttf"),
                w200i_extralight: server
                    .load("fonts/victor_mono/static/VictorMono-ExtraLightItalic.ttf"),
                w300i_light: server.load("fonts/victor_mono/static/VictorMono-LightItalic.ttf"),
                w400i_regular: server.load("fonts/victor_mono/static/VictorMono-Italic.ttf"),
                w500i_medium: server.load("fonts/victor_mono/static/VictorMono-MediumItalic.ttf"),
                w600i_semibold: server
                    .load("fonts/victor_mono/static/VictorMono-SemiBoldItalic.ttf"),
                w700i_bold: server.load("fonts/victor_mono/static/VictorMono-BoldItalic.ttf"),
            },
            kodemono: KodeMonoAssets {
                w400_regular: server.load("fonts/kode_mono/static/KodeMono-Regular.ttf"),
                w500_medium: server.load("fonts/kode_mono/static/KodeMono-Medium.ttf"),
                w600_semibold: server.load("fonts/kode_mono/static/KodeMono-SemiBold.ttf"),
                w700_bold: server.load("fonts/kode_mono/static/KodeMono-Bold.ttf"),
            },
        },
        anchors: Anchors {
            base: Anchors::calc(63, 95, 128, 128),
            grid1x1: Anchors::calc(18, 31, 36, 44),
            grid1x1x4: Anchors::calc(18, 85, 36, 98),
            character: Anchors::calc(13, 43, 26, 48),
        },
    });
}

#[derive(Clone, Debug)]
pub struct Map {
    pub name: String,
    pub path: String,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct Maps {
    pub maps: Vec<Map>,
}

pub fn finish_loading(mut next_state: ResMut<NextState<State>>) {
    next_state.set(State::MainMenu);
}

pub fn app_setup(app: &mut App) {
    app.init_state::<State>()
        .init_state::<GameState>()
        .init_resource::<Maps>()
        .add_systems(
            Startup,
            (load_assets, arch::init_maps, finish_loading).chain(),
        );
}

#[cfg(not(target_arch = "wasm32"))]
mod arch {
    use super::*;
    use crate::tiledmap::naive_tmx_loader;
    use glob::Pattern;
    use walkdir::WalkDir;

    /// Scans the "assets/maps/" directory for files matching "*.tmx" and returns their
    /// paths.
    pub fn find_tmx_files() -> Vec<String> {
        let mut paths = Vec::new();
        let pattern = Pattern::new("*.tmx").unwrap();
        let base_path = "assets/maps/";
        info!("Loading maps...");
        for entry in WalkDir::new(base_path).into_iter() {
            let Ok(entry) = entry else {
                error!("Error loading: {:?}", entry);
                continue;
            };
            let path = entry.path();
            info!("Found {:?}", path);

            // Check if the path matches the "*.tmx" pattern and is a file
            if path.is_file() && pattern.matches_path(path) {
                // Convert the path to a String and store it in the vector
                if let Some(str_path) = path.to_str() {
                    paths.push(str_path.to_string());
                }
            }
        }
        paths.sort();
        paths
    }

    pub fn init_maps(mut maps: ResMut<Maps>) {
        // Scan for maps:
        let tmx_files = find_tmx_files();
        for path in tmx_files {
            // Loading a map can take 100ms or more. Therefore we do a naive load instead
            let (classname, display_name) = match naive_tmx_loader(&path) {
                Ok(m) => m,
                Err(e) => {
                    warn!("Cannot load map {path:?}: {e}");
                    continue;
                }
            };
            if classname != Some("UnhaunterMap1".to_string()) {
                warn!(
                    "Unrecognized Class {:?} for map {:?} (Should be 'UnhaunterMap1')",
                    classname, path
                );
                continue;
            }
            let default_name = format!("Unnamed ({})", path.replace("assets/maps/", ""));
            let display_name = display_name.unwrap_or(default_name);
            info!("Found map {display_name:?} at path {path:?}");
            maps.maps.push(Map {
                name: display_name,
                path,
            });
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod arch {
    use super::*;

    pub fn find_tmx_files() -> Vec<(String, String)> {
        // WASM does not support scanning folders it seems...
        vec![
            (
                "assets/maps/map_house1.tmx".to_string(),
                "123 Acorn Lane Street House".to_string(),
            ),
            (
                "assets/maps/map_house2.tmx".to_string(),
                "4567 Chocolate Boulevard Street House".to_string(),
            ),
            (
                "assets/maps/map_school1.tmx".to_string(),
                "99 Unicorn Way University".to_string(),
            ),
            (
                "assets/maps/tut01_basics.tmx".to_string(),
                "Tutorial 01: Basics".to_string(),
            ),
            (
                "assets/maps/tut02_glass_house.tmx".to_string(),
                "Tutorial 02: Glass House".to_string(),
            ),
        ]
    }

    pub fn init_maps(mut maps: ResMut<Maps>) {
        // Scan for maps:
        let tmx_files = find_tmx_files();
        for (path, display_name) in tmx_files {
            // let display_name = path.replace("assets/maps/", "");
            info!("Found map {display_name:?} at path {path:?}");
            maps.maps.push(Map {
                name: display_name,
                path,
            });
        }
    }
}
