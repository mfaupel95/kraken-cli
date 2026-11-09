//! Shared validation for names that become filesystem paths.

/// Accepts one `[A-Za-z0-9._-]` path segment, excluding trailing dots and
/// Windows device names (including device names with extensions) on every platform.
///
/// Rejection preserves an injective, traversal-free name-to-path mapping.
pub fn is_safe_name_segment(s: &str) -> bool {
    let safe_bytes = !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'));
    if !safe_bytes || s.ends_with('.') {
        return false;
    }
    // Win32 treats these stems as devices even with an extension, and strips
    // trailing dots. Reject them everywhere so artifacts remain portable.
    let stem = s.split('.').next().unwrap_or_default();
    let reserved = ["CON", "PRN", "AUX", "NUL"]
        .iter()
        .any(|name| stem.eq_ignore_ascii_case(name));
    let numbered_device = stem.len() == 4
        && (stem[..3].eq_ignore_ascii_case("COM") || stem[..3].eq_ignore_ascii_case("LPT"))
        && matches!(stem.as_bytes()[3], b'1'..=b'9');
    !reserved && !numbered_device
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_windows_device_names_and_trailing_dots_on_every_platform() {
        for bad in [
            "CON",
            "con",
            "Con.json",
            "PRN",
            "AUX",
            "NUL",
            "nul.log",
            "COM1",
            "com9.data",
            "LPT1",
            "lpt9.txt",
            "workspace.",
            "a..",
        ] {
            assert!(!is_safe_name_segment(bad), "{bad:?}");
        }
        for good in ["console", "null", "COM0", "COM10", "LPT0", "LPT10", "a.b"] {
            assert!(is_safe_name_segment(good), "{good:?}");
        }
    }

    #[test]
    fn accepts_single_safe_segments_and_rejects_traversal_shapes() {
        for good in ["btc-dip", "lab-m1-r2", "a.b_c-9", "v1.2"] {
            assert!(is_safe_name_segment(good), "{good}");
        }
        for bad in [
            "",
            ".",
            "..",
            "...",
            "a/b",
            "a b",
            "a\\b",
            "ünïcode",
            "a\0b",
        ] {
            assert!(!is_safe_name_segment(bad), "{bad:?}");
        }
    }
}
