
use std::io::Cursor;
struct MemoryReader;

impl tiled::ResourceReader for MemoryReader {
    type Resource = Cursor<&'static [u8]>;
    type Error = std::io::Error;

    fn read_from(
        &mut self,
        path: &std::path::Path,
    ) -> std::result::Result<Self::Resource, Self::Error> {
        let path = path.to_str().unwrap();
        match path {
            "maps/tut01_basics.tmx" => Ok(Cursor::new(include_bytes!(
                "../../assets/maps/tut01_basics.tmx"
            ))),
            "maps/tut02_glass_house.tmx" => Ok(Cursor::new(include_bytes!(
                "../../assets/maps/tut02_glass_house.tmx"
            ))),
            "maps/map_house1.tmx" => Ok(Cursor::new(include_bytes!(
                "../../assets/maps/map_house1.tmx"
            ))),
            "maps/map_house2.tmx" => Ok(Cursor::new(include_bytes!(
                "../../assets/maps/map_house2.tmx"
            ))),
            "maps/map_school1.tmx" => Ok(Cursor::new(include_bytes!(
                "../../assets/maps/map_school1.tmx"
            ))),
            "maps/unhaunter_custom_tileset.tsx" => Ok(Cursor::new(include_bytes!(
                "../../assets/maps/unhaunter_custom_tileset.tsx"
            ))),
            "maps/unhaunter_spritesheet2.tsx" => Ok(Cursor::new(include_bytes!(
                "../../assets/maps/unhaunter_spritesheet2.tsx"
            ))),
            "maps/unhaunter_spritesheetA_3x3x3.tsx" => Ok(Cursor::new(include_bytes!(
                "../../assets/maps/unhaunter_spritesheetA_3x3x3.tsx"
            ))),
            "maps/unhaunter_spritesheetA_6x6x10.tsx" => Ok(Cursor::new(include_bytes!(
                "../../assets/maps/unhaunter_spritesheetA_6x6x10.tsx"
            ))),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            )),
        }
    }
}

pub fn map_loader(path: impl AsRef<std::path::Path>) -> tiled::Map {
    let mut loader =
        tiled::Loader::<tiled::DefaultResourceCache, MemoryReader>::with_cache_and_reader(
            tiled::DefaultResourceCache::new(),
            MemoryReader,
        );
    loader.load_tmx_map(path).unwrap()
}
