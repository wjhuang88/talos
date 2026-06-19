//! Theme palette, contrast helpers, and built-in themes (Nord, Solarized Dark).

use crossterm::style::Color as CColor;
use ratatui::style::Color;

/// Nord theme colors for Talos terminal surfaces.
///
/// Reference: <https://www.nordtheme.com/docs/colors-and-palettes>
pub mod nord {
    use super::Color;

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

/// Solarized Dark palette.
///
/// Reference: <https://ethanschoonover.com/solarized/>
#[allow(dead_code)]
pub mod solarized_dark {
    use super::Color;

    pub const BASE03: Color = Color::Rgb(0, 43, 54);
    pub const BASE02: Color = Color::Rgb(7, 54, 66);
    pub const BASE01: Color = Color::Rgb(88, 110, 117);
    pub const BASE00: Color = Color::Rgb(101, 123, 131);
    pub const BASE0: Color = Color::Rgb(131, 148, 150);
    pub const BASE1: Color = Color::Rgb(147, 161, 161);
    pub const BASE2: Color = Color::Rgb(238, 232, 213);
    pub const BASE3: Color = Color::Rgb(253, 246, 227);
    pub const YELLOW: Color = Color::Rgb(181, 137, 0);
    pub const ORANGE: Color = Color::Rgb(203, 75, 22);
    pub const RED: Color = Color::Rgb(220, 50, 47);
    pub const MAGENTA: Color = Color::Rgb(211, 54, 130);
    pub const VIOLET: Color = Color::Rgb(108, 113, 196);
    pub const BLUE: Color = Color::Rgb(38, 139, 210);
    pub const CYAN: Color = Color::Rgb(42, 161, 152);
    pub const GREEN: Color = Color::Rgb(133, 153, 0);
}

/// A built-in theme defining all semantic color roles for TUI rendering.
pub struct Theme {
    pub text_primary: Color,
    pub text_accent: Color,
    pub text_secondary_accent: Color,
    pub text_success: Color,
    pub text_error: Color,
    pub text_warning: Color,
    pub text_special: Color,
    pub input_bg: Color,
    pub preview_fg: Color,
    pub dim_text: Color,
    pub status_value: Color,
    pub border_default: Color,
    pub border_accent: Color,
    pub prefix_user: Color,
    pub prefix_assistant: Color,
    pub prefix_system: Color,
    pub prefix_error: Color,
    pub tip_success: Color,
    pub tip_result: Color,
    pub tip_error: Color,
    pub tip_info: Color,
    pub hold_preview: [Color; 2],
    pub processing_spinner: [Color; 10],
    pub markdown_code: Color,
    pub markdown_text_strong: Color,
    pub markdown_text_emphasis: Color,
    pub markdown_heading: Color,
    pub markdown_link: Color,
    pub markdown_link_url: Color,
    pub markdown_quote_marker: Color,
    pub markdown_quote_text: Color,
    pub markdown_list_marker: Color,
    pub markdown_table_header: Color,
    pub approval_button: Color,
    pub approval_button_bg: Color,
    pub approval_prompt: Color,
    pub logo_gradient: [Color; 6],
    pub logo_badge_1: Color,
    pub logo_badge_2: Color,
    pub logo_badge_3: Color,
    pub logo_version: Color,
    pub logo_subtitle: Color,
    pub logo_separator: Color,
}

/// Default built-in theme: Nord.
pub const NORD_THEME: &Theme = &Theme {
    text_primary: nord::NORD4,
    text_accent: nord::NORD8,
    text_secondary_accent: nord::NORD9,
    text_success: nord::NORD14,
    text_error: nord::NORD11,
    text_warning: nord::NORD13,
    text_special: nord::NORD15,
    input_bg: nord::NORD1,
    preview_fg: nord::NORD5,
    dim_text: nord::NORD3,
    status_value: nord::NORD9,
    border_default: nord::NORD2,
    border_accent: nord::NORD9,
    prefix_user: nord::NORD14,
    prefix_assistant: nord::NORD8,
    prefix_system: nord::NORD15,
    prefix_error: nord::NORD11,
    tip_success: nord::NORD14,
    tip_result: nord::NORD15,
    tip_error: nord::NORD11,
    tip_info: nord::NORD8,
    hold_preview: [nord::NORD8, nord::NORD7],
    processing_spinner: [
        nord::NORD8,
        nord::NORD9,
        nord::NORD10,
        nord::NORD9,
        nord::NORD8,
        nord::NORD7,
        nord::NORD14,
        nord::NORD7,
        nord::NORD8,
        nord::NORD9,
    ],
    markdown_code: Color::Rgb(0xE5, 0xC0, 0x7B),
    markdown_text_strong: nord::NORD5,
    markdown_text_emphasis: nord::NORD4,
    markdown_heading: nord::NORD8,
    markdown_link: nord::NORD8,
    markdown_link_url: nord::NORD3,
    markdown_quote_marker: nord::NORD8,
    markdown_quote_text: nord::NORD4,
    markdown_list_marker: nord::NORD14,
    markdown_table_header: nord::NORD4,
    approval_button: nord::NORD0,
    approval_button_bg: nord::NORD8,
    approval_prompt: nord::NORD14,
    logo_gradient: [
        nord::NORD10,
        nord::NORD9,
        nord::NORD8,
        nord::NORD7,
        nord::NORD8,
        nord::NORD7,
    ],
    logo_badge_1: nord::NORD7,
    logo_badge_2: nord::NORD8,
    logo_badge_3: nord::NORD9,
    logo_version: nord::NORD9,
    logo_subtitle: nord::NORD4,
    logo_separator: nord::NORD3,
};

/// Alternative built-in theme: Solarized Dark.
///
/// Change `THEME` to `&SOLARIZED_DARK_THEME` to switch the active theme.
#[allow(dead_code)]
pub const SOLARIZED_DARK_THEME: &Theme = &Theme {
    text_primary: solarized_dark::BASE2,
    text_accent: solarized_dark::CYAN,
    text_secondary_accent: solarized_dark::BLUE,
    text_success: solarized_dark::GREEN,
    text_error: solarized_dark::RED,
    text_warning: solarized_dark::YELLOW,
    text_special: solarized_dark::VIOLET,
    input_bg: solarized_dark::BASE02,
    preview_fg: solarized_dark::BASE1,
    dim_text: solarized_dark::BASE01,
    status_value: solarized_dark::BLUE,
    border_default: solarized_dark::BASE01,
    border_accent: solarized_dark::CYAN,
    prefix_user: solarized_dark::GREEN,
    prefix_assistant: solarized_dark::CYAN,
    prefix_system: solarized_dark::VIOLET,
    prefix_error: solarized_dark::RED,
    tip_success: solarized_dark::GREEN,
    tip_result: solarized_dark::VIOLET,
    tip_error: solarized_dark::RED,
    tip_info: solarized_dark::CYAN,
    hold_preview: [solarized_dark::CYAN, solarized_dark::BLUE],
    processing_spinner: [
        solarized_dark::CYAN,
        solarized_dark::BLUE,
        solarized_dark::VIOLET,
        solarized_dark::BLUE,
        solarized_dark::CYAN,
        solarized_dark::GREEN,
        solarized_dark::YELLOW,
        solarized_dark::GREEN,
        solarized_dark::CYAN,
        solarized_dark::BLUE,
    ],
    markdown_code: solarized_dark::ORANGE,
    markdown_text_strong: solarized_dark::BASE3,
    markdown_text_emphasis: solarized_dark::BASE2,
    markdown_heading: solarized_dark::CYAN,
    markdown_link: solarized_dark::BLUE,
    markdown_link_url: solarized_dark::BASE01,
    markdown_quote_marker: solarized_dark::CYAN,
    markdown_quote_text: solarized_dark::BASE2,
    markdown_list_marker: solarized_dark::GREEN,
    markdown_table_header: solarized_dark::BASE2,
    approval_button: solarized_dark::BASE03,
    approval_button_bg: solarized_dark::CYAN,
    approval_prompt: solarized_dark::GREEN,
    logo_gradient: [
        solarized_dark::BLUE,
        solarized_dark::CYAN,
        solarized_dark::GREEN,
        solarized_dark::YELLOW,
        solarized_dark::CYAN,
        solarized_dark::BLUE,
    ],
    logo_badge_1: solarized_dark::GREEN,
    logo_badge_2: solarized_dark::CYAN,
    logo_badge_3: solarized_dark::BLUE,
    logo_version: solarized_dark::BLUE,
    logo_subtitle: solarized_dark::BASE2,
    logo_separator: solarized_dark::BASE01,
};

/// Active built-in theme. Change to `&SOLARIZED_DARK_THEME` to switch themes.
const THEME: &Theme = NORD_THEME;

/// Semantic color assignments — all TUI rendering paths route through these roles.
pub(crate) mod semantic {
    use super::{Color, THEME, nord};

    pub(crate) const TEXT_PRIMARY: Color = THEME.text_primary;
    pub(crate) const TEXT_ACCENT: Color = THEME.text_accent;
    pub(crate) const TEXT_SECONDARY_ACCENT: Color = THEME.text_secondary_accent;
    pub(crate) const TEXT_SUCCESS: Color = THEME.text_success;
    pub(crate) const TEXT_ERROR: Color = THEME.text_error;
    pub(crate) const TEXT_WARNING: Color = THEME.text_warning;
    pub(crate) const TEXT_SPECIAL: Color = THEME.text_special;
    pub(crate) const INPUT_BG: Color = THEME.input_bg;
    pub(crate) const NORD2: Color = nord::NORD2;
    pub(crate) const PREVIEW_FG: Color = THEME.preview_fg;
    pub(crate) const DIM_TEXT: Color = THEME.dim_text;
    pub(crate) const STATUS_VALUE: Color = THEME.status_value;
    pub(crate) const BORDER_DEFAULT: Color = THEME.border_default;
    pub(crate) const BORDER_ACCENT: Color = THEME.border_accent;
    pub(crate) const PREFIX_USER: Color = THEME.prefix_user;
    pub(crate) const PREFIX_ASSISTANT: Color = THEME.prefix_assistant;
    pub(crate) const PREFIX_SYSTEM: Color = THEME.prefix_system;
    pub(crate) const PREFIX_ERROR: Color = THEME.prefix_error;
    pub(crate) const TIP_SUCCESS: Color = THEME.tip_success;
    pub(crate) const TIP_RESULT: Color = THEME.tip_result;
    pub(crate) const TIP_ERROR: Color = THEME.tip_error;
    pub(crate) const TIP_INFO: Color = THEME.tip_info;
    pub(crate) const HOLD_PREVIEW: [Color; 2] = THEME.hold_preview;
    pub(crate) const PROCESSING_SPINNER: [Color; 10] = THEME.processing_spinner;
    pub(crate) const MARKDOWN_CODE: Color = THEME.markdown_code;
    pub(crate) const MARKDOWN_TEXT_STRONG: Color = THEME.markdown_text_strong;
    pub(crate) const MARKDOWN_TEXT_EMPHASIS: Color = THEME.markdown_text_emphasis;
    pub(crate) const MARKDOWN_HEADING: Color = THEME.markdown_heading;
    pub(crate) const MARKDOWN_LINK: Color = THEME.markdown_link;
    pub(crate) const MARKDOWN_LINK_URL: Color = THEME.markdown_link_url;
    pub(crate) const MARKDOWN_QUOTE_MARKER: Color = THEME.markdown_quote_marker;
    pub(crate) const MARKDOWN_QUOTE_TEXT: Color = THEME.markdown_quote_text;
    pub(crate) const MARKDOWN_LIST_MARKER: Color = THEME.markdown_list_marker;
    pub(crate) const MARKDOWN_TABLE_HEADER: Color = THEME.markdown_table_header;
    #[allow(dead_code)]
    pub(crate) const APPROVAL_BUTTON: Color = THEME.approval_button;
    #[allow(dead_code)]
    pub(crate) const APPROVAL_BUTTON_BG: Color = THEME.approval_button_bg;
    pub(crate) const APPROVAL_PROMPT: Color = THEME.approval_prompt;
    pub(crate) const LOGO_GRADIENT: [Color; 6] = THEME.logo_gradient;
    pub(crate) const LOGO_BADGE_1: Color = THEME.logo_badge_1;
    pub(crate) const LOGO_BADGE_2: Color = THEME.logo_badge_2;
    pub(crate) const LOGO_BADGE_3: Color = THEME.logo_badge_3;
    pub(crate) const LOGO_VERSION: Color = THEME.logo_version;
    pub(crate) const LOGO_SUBTITLE: Color = THEME.logo_subtitle;
    pub(crate) const LOGO_SEPARATOR: Color = THEME.logo_separator;
}

pub(crate) fn to_crossterm_color(color: Color) -> Option<CColor> {
    match color {
        Color::Rgb(r, g, b) => Some(CColor::Rgb { r, g, b }),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn rgb_components(color: ratatui::style::Color) -> Option<(u8, u8, u8)> {
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
