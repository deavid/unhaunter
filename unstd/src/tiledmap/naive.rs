
/// Loads a TMX as text file and inspects the first lines to obtain class and
/// display_name.
#[cfg(not(target_arch = "wasm32"))]
pub fn naive_tmx_loader(path: &str) -> anyhow::Result<(Option<String>, Option<String>)> {
    use std::io::BufRead as _;

    // ` <map version="1.10" tiledversion="1.10.2" class="UnhaunterMap1" orientation="isometric" renderorder="right-down" width="42" height="42" tilewidth="24" tileheight="12" infinite="0" nextlayerid="18" nextobjectid="15"> <properties> <property name="display_name" value="123 Acorn Lane Street House"/> </properties>`
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut class: Option<String> = None;
    let mut display_name: Option<String> = None;
    for line in reader.lines().take(10) {
        let line = line?.trim().to_owned();
        if line.starts_with("<map") {
            const CLASS_STR: &str = " class=\"";
            if let Some(classpos) = line.find(CLASS_STR) {
                let p_line = &line[classpos + CLASS_STR.len()..];
                if let Some(rpos) = p_line.find('"') {
                    class = Some(p_line[..rpos].to_string());
                }
            }
        }
        if line.starts_with("<property name=\"display_name\"") {
            const VALUE_STR: &str = " value=\"";
            if let Some(valpos) = line.find(VALUE_STR) {
                let p_line = &line[valpos + VALUE_STR.len()..];
                if let Some(rpos) = p_line.find('"') {
                    display_name = Some(p_line[..rpos].to_string());
                }
            }
        }
    }
    Ok((class, display_name))
}
