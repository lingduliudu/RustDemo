use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn try_load_file(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    Some(decode_text(&bytes))
}

fn decode_text(bytes: &[u8]) -> String {
    if let Some((encoding, bom_length)) = encoding_rs::Encoding::for_bom(bytes) {
        return encoding
            .decode_without_bom_handling(&bytes[bom_length..])
            .0
            .into_owned();
    }

    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }

    let mut detector = chardetng::EncodingDetector::new(chardetng::Iso2022JpDetection::Allow);
    detector.feed(bytes, true);
    let encoding = detector.guess(None, chardetng::Utf8Detection::Allow);
    encoding.decode_without_bom_handling(bytes).0.into_owned()
}

pub(crate) fn get_startup_file() -> (String, Option<PathBuf>) {
    if let Some(arg) = std::env::args().nth(1) {
        let path = PathBuf::from(&arg);
        if path.is_file() {
            if let Some(content) = try_load_file(&path) {
                return (content, Some(path));
            }
        }
    }
    ("".to_owned(), None)
}

#[cfg(test)]
mod tests {
    use super::decode_text;

    #[test]
    fn decodes_utf8_and_utf8_bom() {
        assert_eq!(decode_text("中文".as_bytes()), "中文");
        let mut utf8_bom = vec![0xEF, 0xBB, 0xBF];
        utf8_bom.extend_from_slice("中文".as_bytes());
        assert_eq!(decode_text(&utf8_bom), "中文");
    }

    #[test]
    fn decodes_gbk_chinese() {
        assert_eq!(decode_text(&[0xD6, 0xD0, 0xCE, 0xC4]), "中文");
    }

    #[test]
    fn decodes_utf16_bom() {
        assert_eq!(decode_text(&[0xFF, 0xFE, 0x2D, 0x4E, 0x87, 0x65]), "中文");
    }
}
