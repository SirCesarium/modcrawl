use zipcrawl::ZipManager;

use super::super::super::dep::types::DepEntry;
use crate::error::Result;

#[allow(clippy::unnecessary_wraps)]
pub fn extract(_: &mut ZipManager) -> Result<Vec<DepEntry>> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use zip::ZipWriter;

    use super::*;

    fn make_zip_bytes(contents: &[(&str, &str)]) -> Vec<u8> {
        use std::io::Write;
        let mut buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(&mut buf);
        for (name, content) in contents {
            zip.start_file::<&str, ()>(name, Default::default())
                .unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
        buf.into_inner()
    }

    #[test]
    fn extract_always_empty() {
        let yaml = r#"
name: TestPlugin
version: "1.0.0"
main: com.example.TestPlugin
"#;
        let bytes = make_zip_bytes(&[("paper-plugin.yml", yaml)]);
        let mut mng = ZipManager::from_reader(&mut Cursor::new(bytes)).unwrap();
        let deps = extract(&mut mng).unwrap();
        assert!(deps.is_empty());
    }
}
