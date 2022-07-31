extern crate xcf;
use std::{collections::HashMap, fs::create_dir, path::Path};

use xcf::{Layer, PropertyIdentifier, Xcf};

// Layer group information:
// https://testing.developer.gimp.org/core/standards/xcf/#layer

fn is_group(l: &Layer) -> bool {
    /*
        PROP_GROUP_ITEM (since version 3)

        uint32  29       Type identification
        uint32  0        `PROP_GROUP_ITEM` has no payload

        PROP_GROUP_ITEM indicates that the layer is a layer group. It appears in the property list if the layer is a layer group.
    */
    l.properties
        .iter()
        .any(|p| matches!(p.kind, PropertyIdentifier::Unknown(29)))
}

fn get_path_info_u32(l: &Layer) -> Vec<u32> {
    /*
        PROP_ITEM_PATH (since version 3)

        uint32  30       Type identification
        uint32  plength  Total length of the following payload in bytes
        item-path        List of pointers, represented as uint32 values

        PROP_ITEM_PATH indicates the path of the layer if inside a group, i.e. its
        position within the group (last element of the list), but also the position
        of the group itself within its own level, up to the top-level position (first element).
    */
    let mut out = vec![];
    for prop in l.properties.iter() {
        if let PropertyIdentifier::Unknown(30) = prop.kind {
            if let xcf::PropertyPayload::Unknown(v) = &prop.payload {
                // This is an array of u32.
                for ch in v.chunks_exact(4) {
                    let num = u32::from_be_bytes(ch.try_into().unwrap());
                    out.push(num);
                }
            }
        }
    }
    out
}

fn layers_hashmap(layers: &[Layer]) -> HashMap<Vec<u32>, &Layer> {
    let mut ret = HashMap::new();
    let mut next_gimp_id: u32 = 0;
    for layer in layers.iter() {
        let path = get_path_info_u32(layer);
        let gimp_id: Vec<u32> = match path.is_empty() {
            true => {
                let r = vec![next_gimp_id];
                next_gimp_id += 1;
                r
            }
            false => path.clone(),
        };
        ret.insert(gimp_id, layer);
    }
    ret
}

fn layers_path(
    hmap: &HashMap<Vec<u32>, &Layer>,
) -> (HashMap<Vec<u32>, String>, HashMap<String, Vec<u32>>) {
    let mut ret = HashMap::new();
    let mut revret = HashMap::new();
    for (k, v) in hmap.iter() {
        let mut path = vec![];
        for n in 0..(k.len() - 1) {
            let pname = hmap.get(&k[..=n]).map(|l| l.name.as_str()).unwrap_or("???");
            path.push(pname.to_owned());
        }
        // Sanitization to prevent weird issues on the filesystem when saving.
        if v.name.contains('.') {
            eprintln!(
                "Error: XCF layers cannot contain '.' in the name: {:?}",
                v.name
            );
            continue;
        }
        if v.name.contains('/') {
            eprintln!(
                "Error: XCF layers cannot contain '/' in the name: {:?}",
                v.name
            );
            continue;
        }
        path.push(v.name.to_owned());
        let path = path.join("/");
        ret.insert(k.to_owned(), path.clone());
        revret.insert(path, k.to_owned());
    }
    (ret, revret)
}

fn layers_offset(l: &Layer) -> (i32, i32) {
    match l
        .properties
        .iter()
        .find(|p| matches!(p.kind, PropertyIdentifier::PropOffsets))
    {
        Some(p) => match &p.payload {
            xcf::PropertyPayload::Unknown(u) => {
                if u.len() >= 8 {
                    let x = i32::from_be_bytes(u[0..4].try_into().unwrap());
                    let y = i32::from_be_bytes(u[4..8].try_into().unwrap());
                    (x, y)
                } else {
                    (0, 0)
                }
            }
            _ => (0, 0),
        },
        None => (0, 0),
    }
}

fn try_layer_pixel(l: &Layer, x: i32, y: i32) -> Option<[u8; 4]> {
    l.pixel(x.try_into().ok()?, y.try_into().ok()?).map(|p| p.0)
}

fn layer_pixel(l: &Layer, x: i32, y: i32) -> [u8; 4] {
    try_layer_pixel(l, x, y).unwrap_or([0, 0, 0, 0])
}

fn main() {
    println!("XCF loader!");

    let img =
        Xcf::open("assets/img/src/base-tiles.xcf").expect("Failed to open and parse XCF file.");

    let (sx, sy) = img.dimensions();
    let hmap = layers_hashmap(&img.layers);
    let (hmap_path, _hmap_path_rev) = layers_path(&hmap);

    let dstpath = Path::new("./assets/img/base-tiles");

    if !dstpath.exists() {
        panic!(
            "Path {:?} does not exist - check if you are on the correct folder",
            dstpath
        );
    }
    use regex::Regex;
    let re = Regex::new(r"^[.a-z0-9/-]+$").unwrap();
    let mut keys: Vec<_> = hmap.keys().collect();
    keys.sort();

    for k in keys.iter() {
        let layerpath = hmap_path.get(*k).unwrap();
        let l = hmap.get(*k).unwrap();
        let group = is_group(l);
        let ext = match group {
            true => "",
            false => ".png",
        };
        let fullpath = dstpath.join(format!("{}{}", layerpath, ext));
        if !re.is_match(&fullpath.to_string_lossy()) {
            // discard potentially invalid folder names.
            continue;
        }
        if group && !fullpath.exists() {
            eprintln!("Creating folder {:?}", fullpath);
            create_dir(&fullpath).unwrap();
        }
        if !group {
            use image::ImageBuffer;
            let (ox, oy) = layers_offset(l);
            // Construct a new by repeated calls to the supplied closure.
            let img = ImageBuffer::from_fn(sx, sy, |x, y| {
                let px = x as i32 - ox;
                let py = y as i32 - oy;
                image::Rgba(layer_pixel(l, px, py))
            });
            eprintln!("Saving {:?}", fullpath);
            img.save(&fullpath).unwrap();
        }
    }
    //

    // let v = hmap
    //     .get(hmap_path_rev.get("base/wall-left").unwrap())
    //     .unwrap();
    // let (ox, oy) = layers_offset(v);
    // dbg!(layers_offset(v));

    // use image::ImageBuffer;
    // // Construct a new by repeated calls to the supplied closure.
    // let img = ImageBuffer::from_fn(sx, sy, |x, y| {
    //     let px = x as i32 - ox;
    //     let py = y as i32 - oy;
    //     image::Rgba(layer_pixel(v, px, py))
    // });
    // img.save("test-xcfloader.png").unwrap();

    // for (k, v) in hmap.iter() {
    //     let path = hmap_path.get(k).unwrap();
    //     println!("{:?} -> {:?}", k, path);
    //     dbg!(v.dimensions());
    //     dbg!(layers_offset(v));
    // }

    // for layer in img.layers.iter() {
    //     dbg!(&layer.name);
    //     // no group layer information!
    //     // dbg!(layer.dimensions());
    //     // no layer position?
    //     // layer.pixel(x, y)
    //     // dbg!(&layer.);
    // }
    // let layer: &Layer = img.layer("characters").unwrap(); // groups don't seem to contain any useful data.

    // let layer: &Layer = img.layer("wall-left").unwrap();
    // dbg!(&layer.name);
    // dbg!(layer.dimensions());
    // dbg!(&layer.properties);
    // // PropOffsets  -> 2x u32 that offset the layer
    // dbg!(layer.pixel(0, 0));

    /*


    Property {
        kind: Unknown(
            30,
        ),
        length: 8,
        payload: Unknown(
            [
                0,
                0,
                0,
                4,
                0,
                0,
                0,
                6,
            ],
        ),
    },

    Problem: The pointers we don't know how are they set.

    */
}
