use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    Dark,
    Light,
    Dracula,
    Nord,
    Catppuccin,
    TokyoNight,
}

impl ThemeName {
    pub const ALL: [Self; 6] = [
        Self::Dark,
        Self::Light,
        Self::Dracula,
        Self::Nord,
        Self::Catppuccin,
        Self::TokyoNight,
    ];

    pub fn from_setting(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "light" => Self::Light,
            "dracula" => Self::Dracula,
            "nord" => Self::Nord,
            "catppuccin" => Self::Catppuccin,
            "tokyo-night" | "tokyo night" | "tokyonight" => Self::TokyoNight,
            _ => Self::Dark,
        }
    }

    pub fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|theme| *theme == self)
            .unwrap_or_default();
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Dracula => "Dracula",
            Self::Nord => "Nord",
            Self::Catppuccin => "Catppuccin",
            Self::TokyoNight => "Tokyo Night",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub background: Color,
    pub panel: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub muted: Color,
    pub text: Color,
}

impl Palette {
    pub fn for_theme(theme: ThemeName) -> Self {
        match theme {
            ThemeName::Dark => Self {
                background: Color::Black,
                panel: Color::DarkGray,
                accent: Color::Cyan,
                success: Color::Green,
                warning: Color::Yellow,
                danger: Color::Red,
                muted: Color::Gray,
                text: Color::White,
            },
            ThemeName::Light => Self {
                background: Color::White,
                panel: Color::Gray,
                accent: Color::Blue,
                success: Color::Green,
                warning: Color::Yellow,
                danger: Color::Red,
                muted: Color::DarkGray,
                text: Color::Black,
            },
            ThemeName::Dracula => Self {
                background: Color::Rgb(40, 42, 54),
                panel: Color::Rgb(68, 71, 90),
                accent: Color::Rgb(255, 121, 198),
                success: Color::Rgb(80, 250, 123),
                warning: Color::Rgb(241, 250, 140),
                danger: Color::Rgb(255, 85, 85),
                muted: Color::Rgb(189, 147, 249),
                text: Color::Rgb(248, 248, 242),
            },
            ThemeName::Nord => Self {
                background: Color::Rgb(46, 52, 64),
                panel: Color::Rgb(59, 66, 82),
                accent: Color::Rgb(136, 192, 208),
                success: Color::Rgb(163, 190, 140),
                warning: Color::Rgb(235, 203, 139),
                danger: Color::Rgb(191, 97, 106),
                muted: Color::Rgb(129, 161, 193),
                text: Color::Rgb(236, 239, 244),
            },
            ThemeName::Catppuccin => Self {
                background: Color::Rgb(30, 30, 46),
                panel: Color::Rgb(49, 50, 68),
                accent: Color::Rgb(137, 180, 250),
                success: Color::Rgb(166, 227, 161),
                warning: Color::Rgb(249, 226, 175),
                danger: Color::Rgb(243, 139, 168),
                muted: Color::Rgb(180, 190, 254),
                text: Color::Rgb(205, 214, 244),
            },
            ThemeName::TokyoNight => Self {
                background: Color::Rgb(26, 27, 38),
                panel: Color::Rgb(36, 40, 59),
                accent: Color::Rgb(125, 207, 255),
                success: Color::Rgb(158, 206, 106),
                warning: Color::Rgb(224, 175, 104),
                danger: Color::Rgb(247, 118, 142),
                muted: Color::Rgb(122, 162, 247),
                text: Color::Rgb(192, 202, 245),
            },
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::for_theme(ThemeName::Dark)
    }
}

#[cfg(test)]
mod tests {
    use super::ThemeName;

    #[test]
    fn parses_documented_themes() {
        assert_eq!(ThemeName::from_setting("nord"), ThemeName::Nord);
        assert_eq!(
            ThemeName::from_setting("Tokyo Night"),
            ThemeName::TokyoNight
        );
        assert_eq!(ThemeName::from_setting("unknown"), ThemeName::Dark);
    }
}
