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

fn unix_theme() -> (String, Theme) {
    (
        "Unix".to_string(),
        Theme {
            name: Some("Unix".to_string()),
            settings: ThemeSettings {
                background: Some(SyntectColor {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                }), // #000000
                foreground: Some(SyntectColor {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                }), // #FFFFFF
                caret: Some(SyntectColor {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }), // Red
                selection: Some(SyntectColor {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 100,
                }), // Blue with some transparency
                line_highlight: Some(SyntectColor {
                    r: 50,
                    g: 50,
                    b: 50,
                    a: 255,
                }), // Slightly lighter black for line highlight
                ..ThemeSettings::default()
            },
            ..Theme::default()
        },
    )
}

fn default_readable_theme() -> (String, Theme) {
    (
        "Default Readable".to_string(),
        Theme {
            name: Some("Default Readable".to_string()),
            settings: ThemeSettings {
                background: Some(SyntectColor {
                    r: 20,
                    g: 20,
                    b: 20,
                    a: 255,
                }), // Slightly lighter black
                foreground: Some(SyntectColor {
                    r: 220,
                    g: 220,
                    b: 220,
                    a: 255,
                }), // Light gray
                caret: Some(SyntectColor {
                    r: 255,
                    g: 100,
                    b: 100,
                    a: 255,
                }), // Soft red
                selection: Some(SyntectColor {
                    r: 70,
                    g: 70,
                    b: 150,
                    a: 150,
                }), // Semi-transparent blue
                line_highlight: Some(SyntectColor {
                    r: 40,
                    g: 40,
                    b: 40,
                    a: 255,
                }), // Slightly lighter than background
                ..ThemeSettings::default()
            },
            ..Theme::default()
        },
    )
}

pub fn themes() -> Vec<(String, Theme)> {
    vec![
        gruvbox_dark_theme(),
        midnight_theme(),
        unix_theme(),
        default_readable_theme(),
    ]
}
