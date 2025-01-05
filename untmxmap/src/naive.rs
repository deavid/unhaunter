use std::io::BufRead;
use std::io::Result;

pub fn naive_tmx_loader(reader: impl BufRead) -> Result<(Option<String>, Option<String>)> {
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

/// Loads a TMX as text file and inspects the first lines to obtain class and
/// display_name.
#[cfg(not(target_arch = "wasm32"))]
pub fn naive_tmx_file_loader(path: &str) -> Result<(Option<String>, Option<String>)> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    naive_tmx_loader(reader)
}
