//! App theming: brand palettes layered onto the gpui-kit `Theme` system.
//!
//! The kit ships its own light/dark presets; we keep their long tail
//! (component-specific colors we never touch) and overwrite the semantic
//! tokens with the app's palette — warm paper whites for light, warm ink
//! blacks for dark — so every kit component (Dialog, Settings, toasts…)
//! follows the app's look in both modes.
//!
//! The mode preference (light / dark / system) persists to
//! `~/.boundless/theme.json` and is applied at startup.

use gpui::App;
use gpui_component::theme::{Theme, ThemeColor, ThemeMode};
use serde::{Deserialize, Serialize};

/// User-facing mode choice. `System` resolves through the OS appearance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePref {
    #[default]
    Light,
    Dark,
    System,
}

impl ThemePref {
    pub fn label(self) -> &'static str {
        match self {
            ThemePref::Light => "浅色",
            ThemePref::Dark => "深色",
            ThemePref::System => "跟随系统",
        }
    }

    fn resolve(self, cx: &App) -> ThemeMode {
        match self {
            ThemePref::Light => ThemeMode::Light,
            ThemePref::Dark => ThemeMode::Dark,
            ThemePref::System => ThemeMode::from(cx.window_appearance()),
        }
    }
}

fn pref_path() -> std::path::PathBuf {
    crate::ai::store::data_dir().join("theme.json")
}

fn load_pref() -> ThemePref {
    std::fs::read_to_string(pref_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_pref(pref: ThemePref) {
    let path = pref_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&pref) {
        let _ = std::fs::write(path, json);
    }
}

/// The brand palette for one mode, layered over the kit preset: keep every
/// component-specific field we don't name, overwrite the semantic ones.
fn brand_palette(mode: ThemeMode) -> ThemeColor {
    let mut c = match mode {
        ThemeMode::Light => *ThemeColor::light(),
        ThemeMode::Dark => *ThemeColor::dark(),
    };
    match mode {
        ThemeMode::Light => {
            c.background = gpui::rgb(0xfbfaf9).into(); // warm paper
            c.popover = gpui::rgb(0xffffff).into();
            c.title_bar = gpui::rgb(0xfbfaf9).into();
            c.sidebar = gpui::rgb(0xfbfaf9).into();
            c.border = gpui::rgb(0xe3e2df).into();
            c.foreground = gpui::rgb(0x1e1e1e).into();
            c.secondary = gpui::rgb(0xefeeec).into();
            c.muted = gpui::rgb(0xf1f0ee).into();
            c.muted_foreground = gpui::rgb(0x777777).into();
            c.accent = gpui::rgb(0xdce8ff).into();
            c.accent_foreground = gpui::rgb(0x1a5fd7).into();
            c.primary = gpui::rgb(0x1a5fd7).into();
            c.primary_foreground = gpui::rgb(0xffffff).into();
            c.primary_hover = gpui::rgb(0x1550b8).into();
            // Button::primary paints from the button_* tokens (which default
            // to the shadcn ink/white), not from `primary` — pin them to the
            // brand blue so chrome buttons stay blue in both modes.
            c.button_primary = gpui::rgb(0x1a5fd7).into();
            c.button_primary_foreground = gpui::rgb(0xffffff).into();
            c.button_primary_hover = gpui::rgb(0x1550b8).into();
            c.button_primary_active = gpui::rgb(0x154bb0).into();
            c.link = gpui::rgb(0x1a5fd7).into();
            c.danger = gpui::rgb(0xc92a2a).into();
            c.success = gpui::rgb(0x2f9e44).into();
        }
        ThemeMode::Dark => {
            c.background = gpui::rgb(0x1e1d1b).into(); // warm ink
            c.popover = gpui::rgb(0x262523).into();
            c.title_bar = gpui::rgb(0x1e1d1b).into();
            c.sidebar = gpui::rgb(0x1e1d1b).into();
            c.border = gpui::rgb(0x3a3835).into();
            c.foreground = gpui::rgb(0xeceae6).into();
            c.secondary = gpui::rgb(0x333230).into();
            c.muted = gpui::rgb(0x2b2a28).into();
            c.muted_foreground = gpui::rgb(0x9a9894).into();
            c.accent = gpui::rgb(0x2c3b55).into();
            c.accent_foreground = gpui::rgb(0x9ec1ff).into();
            c.primary = gpui::rgb(0x3d7dd8).into();
            c.primary_foreground = gpui::rgb(0xffffff).into();
            c.primary_hover = gpui::rgb(0x5b93e6).into();
            c.button_primary = gpui::rgb(0x3d7dd8).into();
            c.button_primary_foreground = gpui::rgb(0xffffff).into();
            c.button_primary_hover = gpui::rgb(0x5b93e6).into();
            c.button_primary_active = gpui::rgb(0x6ba1eb).into();
            c.link = gpui::rgb(0x9ec1ff).into();
            c.danger = gpui::rgb(0xf0605e).into();
            c.success = gpui::rgb(0x51cf66).into();
        }
    }
    c
}

/// Apply the resolved mode + brand palette to the live theme. `Theme::change`
/// loads the kit preset for the mode (and syncs derived state); the follow-up
/// update rewrites the semantic colors with ours.
fn apply(mode: ThemeMode, cx: &mut App) {
    Theme::change(mode, None, cx);
    let palette = brand_palette(mode);
    Theme::update(cx, |theme| theme.colors = palette);
}

/// Startup: read the persisted preference and theme the app before the first
/// window renders.
pub fn init(cx: &mut App) {
    let pref = load_pref();
    apply(pref.resolve(cx), cx);
}

/// Switch the mode, repaint, and persist the choice.
pub fn set_pref(pref: ThemePref, cx: &mut App) {
    save_pref(pref);
    apply(pref.resolve(cx), cx);
}

/// Current persisted preference (for the settings UI's radio state).
pub fn current_pref() -> ThemePref {
    load_pref()
}

/// A Copy snapshot of the semantic theme colors the app chrome paints with.
/// Borrowing `cx.theme()` across a builder chain conflicts with `cx`'s later
/// mutable uses (listeners, notify), and cloning the whole `Theme` is
/// wasteful — so renderers take one of these up front.
#[derive(Clone, Copy)]
pub struct Chrome {
    pub background: gpui::Hsla,
    pub popover: gpui::Hsla,
    pub border: gpui::Hsla,
    pub foreground: gpui::Hsla,
    pub muted: gpui::Hsla,
    pub muted_foreground: gpui::Hsla,
    pub accent: gpui::Hsla,
    pub accent_foreground: gpui::Hsla,
    pub primary: gpui::Hsla,
    pub primary_hover: gpui::Hsla,
    pub danger: gpui::Hsla,
    pub success: gpui::Hsla,
}

pub fn chrome(cx: &App) -> Chrome {
    let t = gpui_component::ActiveTheme::theme(cx);
    Chrome {
        background: t.background,
        popover: t.popover,
        border: t.border,
        foreground: t.foreground,
        muted: t.muted,
        muted_foreground: t.muted_foreground,
        accent: t.accent,
        accent_foreground: t.accent_foreground,
        primary: t.primary,
        primary_hover: t.primary_hover,
        danger: t.danger,
        success: t.success,
    }
}
