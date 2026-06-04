//! Nord theme palette and contrast helpers.

/// Nord theme colors for Talos terminal surfaces.
///
/// Reference: <https://www.nordtheme.com/docs/colors-and-palettes>
pub mod nord {
    use ratatui::style::Color;

    /// Polar Night darkest background.
    pub const NORD0: Color = Color::Rgb(46, 52, 64);
    /// Polar Night elevated background.
    pub const NORD1: Color = Color::Rgb(59, 66, 82);
    /// Polar Night selected background.
    pub const NORD2: Color = Color::Rgb(67, 76, 94);
    /// Polar Night muted foreground.
    pub const NORD3: Color = Color::Rgb(76, 86, 106);

    /// Snow Storm primary foreground.
    pub const NORD4: Color = Color::Rgb(216, 222, 233);
    /// Snow Storm brighter foreground.
    pub const NORD5: Color = Color::Rgb(229, 233, 240);
    /// Snow Storm brightest foreground.
    pub const NORD6: Color = Color::Rgb(236, 239, 244);

    /// Frost green-blue accent.
    pub const NORD7: Color = Color::Rgb(143, 188, 187);
    /// Frost cyan accent.
    pub const NORD8: Color = Color::Rgb(136, 192, 208);
    /// Frost blue accent.
    pub const NORD9: Color = Color::Rgb(129, 161, 193);
    /// Frost dark blue accent.
    pub const NORD10: Color = Color::Rgb(94, 129, 172);

    /// Aurora red error color.
    pub const NORD11: Color = Color::Rgb(191, 97, 106);
    /// Aurora orange warning color.
    pub const NORD12: Color = Color::Rgb(208, 135, 112);
    /// Aurora yellow warning color.
    pub const NORD13: Color = Color::Rgb(235, 203, 139);
    /// Aurora green success color.
    pub const NORD14: Color = Color::Rgb(163, 190, 140);
    /// Aurora purple accent color.
    pub const NORD15: Color = Color::Rgb(180, 142, 173);
}

#[cfg(test)]
pub(crate) fn rgb_components(color: ratatui::style::Color) -> Option<(u8, u8, u8)> {
    use ratatui::style::Color;

    match color {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        _ => None,
    }
}

#[cfg(test)]
fn relative_luminance(color: ratatui::style::Color) -> Option<f64> {
    let (r, g, b) = rgb_components(color)?;
    let channel = |value: u8| {
        let normalized = f64::from(value) / 255.0;
        if normalized <= 0.04045 {
            normalized / 12.92
        } else {
            ((normalized + 0.055) / 1.055).powf(2.4)
        }
    };
    Some(0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b))
}

#[cfg(test)]
pub(crate) fn contrast_ratio(
    foreground: ratatui::style::Color,
    background: ratatui::style::Color,
) -> Option<f64> {
    let fg = relative_luminance(foreground)?;
    let bg = relative_luminance(background)?;
    let (lighter, darker) = if fg >= bg { (fg, bg) } else { (bg, fg) };
    Some((lighter + 0.05) / (darker + 0.05))
}
