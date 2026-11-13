//! Farb-Helfer und Hex-Parser für `slint::Color`.
//!
//! Da Slint zur Laufzeit standardmäßig keine dynamische Konvertierung von
//! Hex-Farben (z. B. `#RRGGBB` oder `#AARRGGBB`) anbietet, stellt dieses
//! Modul Parsing, Alpha-Modifikation, Formatierung und einen `ColorCache` bereit.

use std::collections::HashMap;
use slint::Color;

/// Gibt eine vollständig transparente Farbe (Alpha = 0) zurück.
#[inline]
pub fn transparent() -> Color {
    Color::from_argb_u8(0, 0, 0, 0)
}

/// Ändert die Transparenz (Alpha: 0..=255) einer bestehenden `slint::Color`.
#[inline]
pub fn with_alpha(color: Color, alpha: u8) -> Color {
    Color::from_argb_u8(alpha, color.red(), color.green(), color.blue())
}

/// Berechnet die relative Luminanz (0.0 ..= 1.0) einer `slint::Color` gemäß WCAG 2.x.
pub fn relative_luminance(c: Color) -> f32 {
    fn linearize(channel: u8) -> f32 {
        let v = channel as f32 / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * linearize(c.red()) + 0.7152 * linearize(c.green()) + 0.0722 * linearize(c.blue())
}

/// Wählt für eine gegebene Hintergrundfarbe die kontrastreichere Textfarbe:
/// entweder dunkles Slate (`#0f172a`) oder reines Weiß (`#ffffff`).
///
/// Bei halbtransparentem oder transparentem Hintergrund (`alpha < 128`) wird
/// `fallback` zurückgegeben, da der effektive Hintergrund vom darunterliegenden
/// Theme bestimmt wird.
pub fn contrast_text(bg: Color, fallback: Color) -> Color {
    if bg.alpha() < 128 {
        return fallback;
    }
    let dark = Color::from_rgb_u8(0x0f, 0x17, 0x2a);
    let light = Color::from_rgb_u8(0xff, 0xff, 0xff);

    let l_bg = relative_luminance(bg);
    let l_dark = relative_luminance(dark);
    let l_light = 1.0_f32;

    let contrast_dark = (l_bg.max(l_dark) + 0.05) / (l_bg.min(l_dark) + 0.05);
    let contrast_light = (l_light + 0.05) / (l_bg + 0.05);

    if contrast_dark >= contrast_light {
        dark
    } else {
        light
    }
}

/// Formatiert eine `slint::Color` als 6-stelligen Hex-String `#RRGGBB`.
pub fn to_hex(color: &Color) -> String {
    format!("#{:02x}{:02x}{:02x}", color.red(), color.green(), color.blue())
}

/// Formatiert eine `slint::Color` als 8-stelligen Hex-String `#AARRGGBB`.
pub fn to_hex_argb(color: &Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        color.alpha(),
        color.red(),
        color.green(),
        color.blue()
    )
}

/// Parst einen Hex-Farbstring (`#RGB`, `#RRGGBB` oder `#AARRGGBB`) mit vollem Alpha (255).
pub fn parse_hex(hex: &str) -> Option<Color> {
    parse_hex_with_alpha(hex, 255)
}

/// Parst einen Hex-Farbstring mit vorgegebenem Alpha (0..=255).
///
/// Unterstützte Formate (mit oder ohne `#`):
/// - 3 Zeichen (`#RGB`): RGB verdoppelt (`#f00` -> `#ff0000`), Alpha wird übernommen.
/// - 6 Zeichen (`#RRGGBB`): Standard-RGB, Alpha wird übernommen.
/// - 8 Zeichen (`#AARRGGBB`): Volles ARGB; das im Hex angegebene Alpha wird verwendet,
///   sofern `default_alpha == 255`, andernfalls überschreibt `default_alpha`.
pub fn parse_hex_with_alpha(hex: &str, default_alpha: u8) -> Option<Color> {
    let clean = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
    match clean.len() {
        3 => {
            let r = u8::from_str_radix(&clean[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&clean[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&clean[2..3], 16).ok()? * 17;
            Some(Color::from_argb_u8(default_alpha, r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
            let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
            let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
            Some(Color::from_argb_u8(default_alpha, r, g, b))
        }
        8 => {
            let a = u8::from_str_radix(&clean[0..2], 16).ok()?;
            let r = u8::from_str_radix(&clean[2..4], 16).ok()?;
            let g = u8::from_str_radix(&clean[4..6], 16).ok()?;
            let b = u8::from_str_radix(&clean[6..8], 16).ok()?;
            let final_a = if default_alpha != 255 {
                default_alpha
            } else {
                a
            };
            Some(Color::from_argb_u8(final_a, r, g, b))
        }
        _ => None,
    }
}

/// In-Memory Cache für Farben und Brushes.
///
/// Cacht registrierte Farbnamen (z. B. `DirtyCellBrush`) sowie dynamisch geparste
/// Hex-Werte und deren Alpha-Varianten (`{name}_{alpha}`).
#[derive(Debug, Clone, Default)]
pub struct ColorCache {
    cache: HashMap<String, Color>,
}

impl ColorCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registriert einen benannten Farbwert (z. B. "Highlight", "Error").
    pub fn register(&mut self, key: impl Into<String>, color: Color) {
        let k = key.into().to_lowercase();
        self.cache.insert(k, color);
    }

    /// Parst und registriert einen Hex-Farbwert unter einem Namen.
    pub fn register_hex(&mut self, key: impl Into<String>, hex: &str, alpha: u8) -> Option<Color> {
        let c = parse_hex_with_alpha(hex, alpha)?;
        self.register(key, c);
        Some(c)
    }

    /// Ruft eine registrierte Farbe ab.
    pub fn get(&self, key: &str) -> Option<Color> {
        self.cache.get(&key.to_lowercase()).copied()
    }

    /// Holt eine Farbe aus dem Cache oder parst sie als Hex-Wert.
    /// Erkennt automatisch "transparent", "none", "keine" und Leerstrings.
    pub fn get_or_parse(&mut self, key_or_hex: &str, alpha: u8) -> Color {
        let trimmed = key_or_hex.trim();
        if trimmed.is_empty()
            || trimmed.eq_ignore_ascii_case("keine")
            || trimmed.eq_ignore_ascii_case("none")
            || trimmed.eq_ignore_ascii_case("transparent")
        {
            return transparent();
        }

        let lower = trimmed.to_lowercase();
        let cache_key = if alpha == 255 {
            lower.clone()
        } else {
            format!("{}_{}", lower, alpha)
        };

        if let Some(&c) = self.cache.get(&cache_key) {
            return c;
        }

        // Falls wir den Basis-Farbwert (alpha 255) kennen, modifizieren wir ihn
        if alpha != 255 {
            if let Some(&base) = self.cache.get(&lower) {
                if base.alpha() > 0 {
                    let derived = with_alpha(base, alpha);
                    self.cache.insert(cache_key, derived);
                    return derived;
                }
            }
        }

        let parsed = parse_hex_with_alpha(trimmed, alpha).unwrap_or_else(transparent);
        self.cache.insert(cache_key, parsed);
        parsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_rgb() {
        let c = parse_hex("#ff0000").unwrap();
        assert_eq!(c.red(), 255);
        assert_eq!(c.green(), 0);
        assert_eq!(c.blue(), 0);
        assert_eq!(c.alpha(), 255);

        let c_short = parse_hex("#0f0").unwrap();
        assert_eq!(c_short.red(), 0);
        assert_eq!(c_short.green(), 255);
        assert_eq!(c_short.blue(), 0);
        assert_eq!(c_short.alpha(), 255);
    }

    #[test]
    fn test_parse_hex_with_alpha() {
        let c = parse_hex_with_alpha("#123456", 200).unwrap();
        assert_eq!(c.red(), 0x12);
        assert_eq!(c.green(), 0x34);
        assert_eq!(c.blue(), 0x56);
        assert_eq!(c.alpha(), 200);

        let c_argb = parse_hex("#80123456").unwrap();
        assert_eq!(c_argb.red(), 0x12);
        assert_eq!(c_argb.green(), 0x34);
        assert_eq!(c_argb.blue(), 0x56);
        assert_eq!(c_argb.alpha(), 0x80);
    }

    #[test]
    fn test_to_hex() {
        let c = Color::from_argb_u8(255, 0x1a, 0x2b, 0x3c);
        assert_eq!(to_hex(&c), "#1a2b3c");
        assert_eq!(to_hex_argb(&c), "#ff1a2b3c");
    }

    #[test]
    fn test_color_cache() {
        let mut cache = ColorCache::new();
        cache.register("primary", Color::from_argb_u8(255, 10, 20, 30));

        let c1 = cache.get_or_parse("primary", 255);
        assert_eq!(c1.red(), 10);
        assert_eq!(c1.alpha(), 255);

        let c1_alpha = cache.get_or_parse("primary", 128);
        assert_eq!(c1_alpha.red(), 10);
        assert_eq!(c1_alpha.alpha(), 128);

        let transparent_c = cache.get_or_parse("Keine", 255);
        assert_eq!(transparent_c.alpha(), 0);

        let hex_c = cache.get_or_parse("#aabbcc", 200);
        assert_eq!(hex_c.red(), 0xaa);
        assert_eq!(hex_c.alpha(), 200);
    }

    #[test]
    fn test_relative_luminance() {
        let black = Color::from_rgb_u8(0, 0, 0);
        let white = Color::from_rgb_u8(255, 255, 255);
        assert!((relative_luminance(black) - 0.0).abs() < 1e-6);
        assert!((relative_luminance(white) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_contrast_text() {
        let dark_expected = Color::from_rgb_u8(0x0f, 0x17, 0x2a);
        let light_expected = Color::from_rgb_u8(0xff, 0xff, 0xff);
        let fallback = Color::from_rgb_u8(0x94, 0xa3, 0xb8);

        // Weiß -> dunkel
        assert_eq!(
            contrast_text(Color::from_rgb_u8(255, 255, 255), fallback),
            dark_expected
        );
        // Schwarz -> weiß
        assert_eq!(
            contrast_text(Color::from_rgb_u8(0, 0, 0), fallback),
            light_expected
        );
        // Gelb #fde047 -> dunkel
        assert_eq!(
            contrast_text(parse_hex("#fde047").unwrap(), fallback),
            dark_expected
        );
        // Dunkelblau #1e3a8a -> weiß
        assert_eq!(
            contrast_text(parse_hex("#1e3a8a").unwrap(), fallback),
            light_expected
        );
        // Alpha < 128 -> fallback
        assert_eq!(
            contrast_text(Color::from_argb_u8(100, 255, 255, 255), fallback),
            fallback
        );
    }
}
