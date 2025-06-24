use syntect::highlighting::{Color as SyntectColor, Theme, ThemeSettings};

fn midnight_theme() -> (String, Theme) {
    (
        "Midnight".to_string(),
        Theme {
            name: Some("Midnight".to_string()),
            settings: ThemeSettings {
                background: Some(SyntectColor {
                    r: 20,
                    g: 24,
                    b: 31,
                    a: 0xFF,
                }),
                foreground: Some(SyntectColor {
                    r: 220,
                    g: 220,
                    b: 220,
                    a: 0xFF,
                }),
                caret: Some(SyntectColor {
                    r: 80,
                    g: 160,
                    b: 255,
                    a: 0xFF,
                }),
                selection: Some(SyntectColor {
                    r: 60,
                    g: 80,
                    b: 120,
                    a: 0xFF,
                }),
                ..ThemeSettings::default()
            },
            ..Theme::default()
        },
    )
}

fn gruvbox_dark_theme() -> (String, Theme) {
    (
        "Gruvbox Dark".to_string(),
        Theme {
            name: Some("Gruvbox Dark".to_string()),
            settings: ThemeSettings {
                background: Some(SyntectColor {
                    r: 40,
                    g: 40,
                    b: 40,
                    a: 0xFF,
                }),
                foreground: Some(SyntectColor {
                    r: 235,
                    g: 219,
                    b: 178,
                    a: 0xFF,
                }),
                caret: Some(SyntectColor {
                    r: 250,
                    g: 189,
                    b: 47,
                    a: 0xFF,
                }),
                selection: Some(SyntectColor {
                    r: 60,
                    g: 56,
                    b: 54,
                    a: 0xFF,
                }),
                ..ThemeSettings::default()
            },
            ..Theme::default()
        },
    )
}
pub fn themes() -> Vec<(String, Theme)> {
    return vec![gruvbox_dark_theme(), midnight_theme()];
}
