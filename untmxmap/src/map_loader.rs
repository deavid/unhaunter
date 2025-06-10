use bevy::prelude::*;
use std::io::Cursor;
use uncore::{
    assets::{tmxmap::TmxMap, tsxsheet::TsxSheet},
    resources::maps::Maps,
};

struct TmxMemoryReader<'a> {
    maps: &'a Res<'a, Maps>,
    tmx_assets: &'a Res<'a, Assets<TmxMap>>,
    tsx_assets: &'a Res<'a, Assets<TsxSheet>>,
}

impl<'a> tiled::ResourceReader for TmxMemoryReader<'a> {
    type Resource = Cursor<&'a [u8]>;
    type Error = std::io::Error;

    fn read_from(
        &mut self,
        path: &std::path::Path,
    ) -> std::result::Result<Self::Resource, Self::Error> {
        let path = path.to_str().unwrap();
        info!("Tiled - loading {path:?}");
        if let Some(map) = self.maps.maps.iter().find(|m| m.path == path) {
            let Some(map) = self.tmx_assets.get(&map.handle) else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("TMX file {path:?} not loaded yet!"),
                ));
            };
            return Ok(Cursor::new(&map.bytes));
        }

        if let Some(sheet) = self.maps.sheets.iter().find(|s| s.path == path) {
            let Some(sheet) = self.tsx_assets.get(&sheet.handle) else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("TSX file {path:?} not loaded yet!"),
                ));
            };
            return Ok(Cursor::new(&sheet.bytes));
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ))
    }
}

pub struct UnhaunterMapLoader;

impl UnhaunterMapLoader {
    pub fn load(
        path: impl AsRef<std::path::Path>,
        maps: &Res<Maps>,
        tmx_assets: &Res<Assets<TmxMap>>,
        tsx_assets: &Res<Assets<TsxSheet>>,
    ) -> tiled::Map {
        let now = bevy_platform::time::Instant::now();
        let mut loader =
            tiled::Loader::<tiled::DefaultResourceCache, TmxMemoryReader>::with_cache_and_reader(
                tiled::DefaultResourceCache::new(),
                TmxMemoryReader {
                    maps,
                    tmx_assets,
                    tsx_assets,
                },
            );
        let ret = loader.load_tmx_map(path).unwrap();
        warn!("Loaded map in {:?}", now.elapsed());
        ret
    }
}
