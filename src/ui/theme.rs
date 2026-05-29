use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub accent: Color,
    pub success: Color,
    pub muted: Color,
    pub text: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            accent: Color::Cyan,
            success: Color::Green,
            muted: Color::Gray,
            text: Color::White,
        }
    }
}
