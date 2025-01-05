use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use uncore::resources::maps::Maps;
use uncore::states::{GameState, AppState};
use uncore::types::root::anchors::Anchors;
use uncore::types::root::font_assets::{
    ChakraPetchAssets, FontAssets, KodeMonoAssets, LondrinaFontAssets, OverlockFontAssets,
    SyneFontAssets, TitilliumWebAssets, VictorMonoAssets,
};
use uncore::types::root::game_assets::GameAssets;
use uncore::types::root::image_assets::ImageAssets;

pub const FPS_DEBUG: bool = false;
pub struct UnhaunterRootPlugin;

impl Plugin for UnhaunterRootPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_state::<GameState>()
            .init_resource::<Maps>()
            .add_systems(
                Startup,
                (load_assets, finish_loading).chain(),
            );
        if FPS_DEBUG {
            app.add_plugins(FrameTimeDiagnosticsPlugin)
                .add_plugins(LogDiagnosticsPlugin::default());
        }

        arch_setup::app_setup(app);
    }
}

fn load_assets(
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

fn finish_loading(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::MainMenu);
}

#[cfg(not(target_arch = "wasm32"))]
mod arch_setup {
    use super::*;

    fn set_fps_limiter(mut settings: ResMut<bevy_framepace::FramepaceSettings>) {
        settings.limiter = bevy_framepace::Limiter::from_framerate(60.0);
    }

    pub fn app_setup(app: &mut App) {
        app.add_plugins(bevy_framepace::FramepacePlugin)
            .add_systems(Startup, set_fps_limiter);
        if FPS_DEBUG {
            app.add_plugins(bevy_framepace::debug::DiagnosticsPlugin);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod arch_setup {
    use super::*;

    pub fn app_setup(_app: &mut App) {}
}
