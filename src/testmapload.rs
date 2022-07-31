use tiled::Loader;

fn main() {
    let mut loader = Loader::new();
    let map = loader
        .load_tmx_map("assets/maps/unhaunter_test2b.tmx")
        .unwrap();
    println!("{:?}", map);
    println!("{:?}", map.tilesets()[0].get_tile(0).unwrap().probability);

    let tileset = loader
        .load_tsx_tileset("assets/maps/unhaunter_spritesheet2.tsx")
        .unwrap();
    assert_eq!(*map.tilesets()[0], tileset, "The tileset of the map should match with the expected tileset. If not this means that the map is loading the tileset from elsewhere");
}
