//! A platform an image is for, as a string.

use serde::Deserialize;
use serde::de::{Deserializer, Error as _};

/// The platform an OCI index entry, or an image's own config, states,
/// as the one string everything here names a platform by. The rules:
///
/// 1. **`os/architecture`**, the two as the manifest states them, with
///    one slash between: `linux/amd64`, `linux/arm64`.
/// 2. **`/variant` after, where there is one**: `linux/arm/v7`. A
///    platform with no variant, or an empty one, has no third part.
/// 3. **As stated, and compared whole.** Nothing is normalized:
///    `linux/arm64` and `linux/arm64/v8` are two platforms, because
///    two manifests said two things.
pub fn of(os: &str, architecture: &str, variant: Option<&str>) -> String {
    match variant {
        Some(variant) if !variant.is_empty() => format!("{os}/{architecture}/{variant}"),
        _ => format!("{os}/{architecture}"),
    }
}

/// Whether `platform` is a platform string a request may carry: two
/// parts or three, each one or more of lowercase letters, digits,
/// `.`, `_` and `-`. What a manifest states is not held to this — a
/// listing reports whatever a manifest said — but what a caller asks
/// for is, so that a platform asked for is one a manifest could state.
pub fn valid(platform: &str) -> bool {
    let parts: Vec<&str> = platform.split('/').collect();
    (parts.len() == 2 || parts.len() == 3)
        && parts.iter().all(|part| {
            !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-'))
        })
}

/// A list of platforms that names at least one, for a member that is
/// required to: `#[serde(deserialize_with = "platform::some")]`. An
/// empty list does not decode, and neither does one naming a platform
/// twice or one that is not [`valid`]: a list of the platforms asked
/// for, or held out, says each once.
pub fn some<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    let platforms = Vec::<String>::deserialize(deserializer)?;
    if platforms.is_empty() {
        return Err(D::Error::custom("no platform is named: at least one is required"));
    }
    for (index, platform) in platforms.iter().enumerate() {
        if !valid(platform) {
            return Err(D::Error::custom(format!("`{platform}` is not a platform: `os/architecture`, or `os/architecture/variant`")));
        }
        if platforms[..index].contains(platform) {
            return Err(D::Error::custom(format!("the platform `{platform}` is named twice")));
        }
    }
    Ok(platforms)
}
