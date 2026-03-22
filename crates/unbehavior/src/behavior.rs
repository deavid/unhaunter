//! Everything moved to unbehavior-core, except the dependency with Tiled. That remains here.
//! This is to be eventually moved to a better place - this crate is to be removed at some point.

use bevy::log::warn;
use bevy_platform::collections::HashMap;
use unbehavior_core::behavior::{
    BehaviorProperties, NpcHelpDialog, SpriteConfig, UnhaunterPropertyValue,
};

fn to_unhaunter_prop(value: &tiled::PropertyValue) -> UnhaunterPropertyValue {
    match value {
        tiled::PropertyValue::BoolValue(v) => UnhaunterPropertyValue::BoolValue(*v),
        tiled::PropertyValue::FloatValue(v) => UnhaunterPropertyValue::FloatValue(*v),
        tiled::PropertyValue::IntValue(v) => UnhaunterPropertyValue::IntValue(*v),
        tiled::PropertyValue::StringValue(v) => UnhaunterPropertyValue::StringValue(v.clone()),
        tiled::PropertyValue::ColorValue(v) => {
            UnhaunterPropertyValue::ColorValue([v.red, v.green, v.blue, v.alpha])
        }
        tiled::PropertyValue::FileValue(v) => UnhaunterPropertyValue::FileValue(v.clone()),
        tiled::PropertyValue::ObjectValue(v) => UnhaunterPropertyValue::ObjectValue(*v),
        tiled::PropertyValue::ClassValue { property_type, .. } => {
            UnhaunterPropertyValue::ClassValue {
                property_type: property_type.clone(),
            }
        }
    }
}

fn behavior_properties_from_tiled(tiled_tile: &tiled::Tile) -> BehaviorProperties {
    let properties = tiled_tile
        .properties
        .iter()
        .map(|(key, value)| (key.clone(), to_unhaunter_prop(value)))
        .collect();
    BehaviorProperties::from_map(properties)
}

pub fn sprite_config_from_tiled_auto(
    tset_name: String,
    tileuid: u32,
    tiled_tile: &tiled::Tile,
) -> SpriteConfig {
    let properties = behavior_properties_from_tiled(tiled_tile);
    SpriteConfig::from_class_str(
        tiled_tile.user_type.as_deref(),
        tset_name,
        tileuid,
        properties,
    )
}

pub fn npc_help_dialog_from_tiled(
    classname: &str,
    variant: &str,
    user_properties: &HashMap<String, tiled::PropertyValue>,
) -> NpcHelpDialog {
    let key = format!("{classname}:{variant}:dialog");
    let dialog = match user_properties.get(&key) {
        Some(p) => match p {
            tiled::PropertyValue::StringValue(v) => v.to_string(),
            _ => {
                warn!(
                    "NPCHelpDialog was expecting a user property named {key:?} in the layer but it had an unsupported type - it must be text"
                );
                "".to_string()
            }
        },
        None => {
            warn!(
                "NPCHelpDialog was expecting a user property named {key:?} in the layer but was not present"
            );
            "".to_string()
        }
    };
    NpcHelpDialog::new(dialog)
}
