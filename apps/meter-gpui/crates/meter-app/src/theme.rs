use gpui::{rgb, Rgba, WindowAppearance};
use meter_core::ToolId;

/// Colors follow the system appearance. They mirror macOS system colors; meaning is never carried
/// by color alone (the views always add words or a glyph).
#[derive(Clone, Copy)]
pub struct Theme {
    pub window: Rgba,
    pub sidebar: Rgba,
    pub card: Rgba,
    pub text: Rgba,
    pub secondary: Rgba,
    pub line: Rgba,
    pub track: Rgba,
    pub accent: Rgba,
    pub selected: Rgba,
    pub green: Rgba,
    pub orange: Rgba,
    pub red: Rgba,
}

impl Theme {
    pub fn for_appearance(a: WindowAppearance) -> Theme {
        match a {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Theme {
                window: rgb(0x1E1E1E),
                sidebar: rgb(0x262628),
                card: rgb(0x2A2A2C),
                text: rgb(0xF2F2F7),
                secondary: rgb(0x9A9AA1),
                line: rgb(0x3A3A3C),
                track: rgb(0x3A3A3C),
                accent: rgb(0x0A84FF),
                selected: rgb(0x0A5FD0),
                green: rgb(0x30D158),
                orange: rgb(0xFF9F0A),
                red: rgb(0xFF453A),
            },
            _ => Theme {
                window: rgb(0xF5F5F7),
                sidebar: rgb(0xE9E9EC),
                card: rgb(0xFFFFFF),
                text: rgb(0x1D1D1F),
                secondary: rgb(0x6E6E73),
                line: rgb(0xD8D8DC),
                track: rgb(0xE3E3E8),
                accent: rgb(0x007AFF),
                selected: rgb(0xBFD9FF),
                green: rgb(0x1F9D3A),
                orange: rgb(0xC77700),
                red: rgb(0xD70015),
            },
        }
    }

    pub fn tool_color(&self, tool: ToolId) -> Rgba {
        match tool {
            ToolId::Codex => self.accent,
            ToolId::Grok => rgb(0xAF52DE),
            ToolId::ClaudeCode => rgb(0xFF9F0A),
            ToolId::Cursor => rgb(0x30B0C7),
            ToolId::Agy => rgb(0xFF2D55),
        }
    }
}
