//! Turning the bytes of a text file into text, whatever its character encoding.
//!
//! Order: a byte-order mark if there is one; then the encoding the file declares (HTML);
//! then UTF-8 if the bytes are valid UTF-8; otherwise a guess by `chardetng`, the
//! detector Firefox uses, which handles the legacy encodings common in old libraries
//! (Windows-1252 and Latin-1, Shift_JIS, GBK, Big5, KOI8-R and others).

use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use encoding_rs::{Encoding, UTF_8};

/// Decodes `bytes` to text. Bytes that are invalid in the chosen encoding become the
/// replacement character (�) instead of stopping the reading.
pub fn decode(bytes: &[u8], declared: Option<&'static Encoding>) -> String {
    if let Some((encoding, bom_length)) = Encoding::for_bom(bytes) {
        return encoding
            .decode_without_bom_handling(&bytes[bom_length..])
            .0
            .into_owned();
    }
    if let Some(encoding) = declared {
        return encoding.decode_without_bom_handling(bytes).0.into_owned();
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    let encoding = guess(bytes);
    encoding.decode_without_bom_handling(bytes).0.into_owned()
}

fn guess(bytes: &[u8]) -> &'static Encoding {
    let mut detector = EncodingDetector::new(Iso2022JpDetection::Deny);
    // A sample is enough to recognize the encoding and keeps large files fast.
    const SAMPLE: usize = 1024 * 1024;
    let sample = &bytes[..bytes.len().min(SAMPLE)];
    detector.feed(sample, sample.len() == bytes.len());
    let guess = detector.guess(None, Utf8Detection::Allow);
    // UTF-8 was ruled out above (the bytes are not valid UTF-8).
    if guess == UTF_8 {
        encoding_rs::WINDOWS_1252
    } else {
        guess
    }
}

/// The encoding an HTML file declares in a `<meta charset>` near its start, if any.
/// This is the simplified form of the browsers' "prescan": it looks for `charset=`
/// in the first 1024 bytes.
pub fn html_declared(bytes: &[u8]) -> Option<&'static Encoding> {
    let head = &bytes[..bytes.len().min(1024)];
    let lower: Vec<u8> = head.iter().map(u8::to_ascii_lowercase).collect();
    let needle = b"charset=";
    let start = lower.windows(needle.len()).position(|w| w == needle)? + needle.len();
    let rest = &lower[start..];
    let rest = rest
        .strip_prefix(b"\"")
        .or_else(|| rest.strip_prefix(b"'"))
        .unwrap_or(rest);
    let end = rest
        .iter()
        .position(|&b| matches!(b, b'"' | b'\'' | b';' | b'>' | b'/') || b.is_ascii_whitespace())
        .unwrap_or(rest.len());
    let encoding = Encoding::for_label(&rest[..end])?;
    // As browsers do: a UTF-16 label inside an ASCII-compatible file means UTF-8.
    if encoding == encoding_rs::UTF_16LE || encoding == encoding_rs::UTF_16BE {
        Some(UTF_8)
    } else {
        Some(encoding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_is_kept() {
        assert_eq!(decode("ação 目录".as_bytes(), None), "ação 目录");
    }

    #[test]
    fn byte_order_marks_are_honored_and_removed() {
        let mut utf16 = vec![0xFF, 0xFE];
        for unit in "Ação".encode_utf16() {
            utf16.extend_from_slice(&unit.to_le_bytes());
        }
        assert_eq!(decode(&utf16, None), "Ação");
        assert_eq!(decode(b"\xEF\xBB\xBFabc", None), "abc");
    }

    #[test]
    fn legacy_western_text_is_recognized() {
        // "A formação da literatura brasileira" in Windows-1252.
        let (bytes, _, _) = encoding_rs::WINDOWS_1252
            .encode("A formação da literatura brasileira, coração e razão.");
        assert_eq!(
            decode(&bytes, None),
            "A formação da literatura brasileira, coração e razão."
        );
    }

    #[test]
    fn legacy_chinese_text_is_recognized() {
        let original =
            "子曰：學而時習之，不亦說乎？有朋自遠方來，不亦樂乎？人不知而不慍，不亦君子乎？";
        let (bytes, _, _) = encoding_rs::BIG5.encode(original);
        assert_eq!(decode(&bytes, None), original);
    }

    #[test]
    fn declared_html_encoding_is_found() {
        let page = b"<html><head><meta charset=\"ISO-8859-1\"></head>";
        assert_eq!(html_declared(page), Some(encoding_rs::WINDOWS_1252));
        let page = b"<meta http-equiv=Content-Type content='text/html; charset=shift_jis'>";
        assert_eq!(html_declared(page), Some(encoding_rs::SHIFT_JIS));
        assert_eq!(html_declared(b"<p>no declaration</p>"), None);
        assert_eq!(html_declared(b"charset=nonsense"), None);
    }
}
