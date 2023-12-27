use tiled::{Loader, TileLayer};

fn main() {
    let mut loader = Loader::new();
    let map = loader
        .load_tmx_map("assets/maps/map_house1_3x.tmx")
        .unwrap();
    // println!("{:?}", map);
    // println!("{:?}", map.tilesets()[0].get_tile(0).unwrap().probability);
    // let tileset = loader
    //     .load_tsx_tileset("assets/maps/unhaunter_spritesheet2.tsx")
    //     .unwrap();
    // assert_eq!(*map.tilesets()[0], tileset, "The tileset of the map should match with the expected tileset. If not this means that the map is loading the tileset from elsewhere");
    dbg!(map.width, map.height);
    dbg!(map.tile_width, map.tile_height);
    dbg!(map.infinite());
    assert!(!map.infinite(), "The tileset cannot be infinite!");
    dbg!(map.orientation);
    for (n, tileset) in map.tilesets().iter().enumerate() {
        dbg!(n, &tileset.name);
    }
    load_tile_layer_iter(map.layers());

    println!("LOAD OK");
}

fn load_tile_layer_iter<'a>(layer_iter: impl ExactSizeIterator<Item = tiled::Layer<'a>>) {
    for (n, layer) in layer_iter.enumerate() {
        dbg!(n, &layer.name);
        load_tile_layer(layer);
    }
}

fn load_tile_layer(layer: tiled::Layer) {
    match layer.layer_type() {
        tiled::LayerType::Tiles(tile_layer) => {
            load_tile_layer_tiles(tile_layer);
        }
        tiled::LayerType::Objects(_) => {
            dbg!("Objects");
        }
        tiled::LayerType::Image(_) => {
            dbg!("Image");
        }
        tiled::LayerType::Group(grp_layer) => load_tile_group_layer(grp_layer),
    };
}

fn load_tile_group_layer(layer: tiled::GroupLayer) {
    dbg!("Group");
    load_tile_layer_iter(layer.layers())
}

fn load_tile_layer_tiles(layer: tiled::TileLayer) {
    dbg!("Tiles");
    dbg!(layer.width(), layer.height());
    for y in 0..layer.height().unwrap() as i32 {
        for x in 0..layer.width().unwrap() as i32 {
            let maybe_tile = layer.get_tile(x, y);

            if let Some(tile) = maybe_tile {
                // tile.flip_h
                dbg!(x, y, &tile.get_tileset().name, tile.id());
            }
        }
    }
}
