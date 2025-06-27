use syntect::highlighting::{Color as SyntectColor, Theme, ThemeSettings, UnderlineOption};

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
                    a: 255,
                }),
                foreground: Some(SyntectColor {
                    r: 220,
                    g: 220,
                    b: 220,
                    a: 255,
                }),
                caret: Some(SyntectColor {
                    r: 80,
                    g: 160,
                    b: 255,
                    a: 255,
                }),
                selection: Some(SyntectColor {
                    r: 60,
                    g: 80,
                    b: 120,
                    a: 255,
                }),
                line_highlight: Some(SyntectColor {
                    r: 30,
                    g: 34,
                    b: 41,
                    a: 255,
                }), // Slightly lighter background
                misspelling: Some(SyntectColor {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }), // Red
                minimap_border: Some(SyntectColor {
                    r: 50,
                    g: 50,
                    b: 50,
                    a: 255,
                }), // Dark gray
                accent: Some(SyntectColor {
                    r: 80,
                    g: 160,
                    b: 255,
                    a: 255,
                }), // Blue (same as caret)
                popup_css: Some("".to_string()),
                phantom_css: Some("".to_string()),
                bracket_contents_foreground: Some(SyntectColor { r: 220, g: 220, b: 220, a: 255 }),
                bracket_contents_options: Some(UnderlineOption::default()),
                brackets_foreground: Some(SyntectColor { r: 220, g: 220, b: 220, a: 255 }),
                brackets_background: Some(SyntectColor { r: 20, g: 24, b: 31, a: 255 }),
                brackets_options: Some(UnderlineOption::default()),
                tags_foreground: Some(SyntectColor { r: 220, g: 220, b: 220, a: 255 }),
                tags_options: Some(UnderlineOption::default()),
                highlight: Some(SyntectColor { r: 100, g: 180, b: 255, a: 255 }), // Light blue
                find_highlight: Some(SyntectColor { r: 255, g: 255, b: 0, a: 255 }), // Yellow
                find_highlight_foreground: Some(SyntectColor { r: 0, g: 0, b: 0, a: 255 }), // Black
                gutter: Some(SyntectColor { r: 15, g: 19, b: 26, a: 255 }), // Slightly darker background
                gutter_foreground: Some(SyntectColor { r: 100, g: 100, b: 100, a: 255 }), // Muted gray
                selection_foreground: Some(SyntectColor { r: 0, g: 0, b: 0, a: 255 }), // Black (contrasting with blue selection)
                selection_border: Some(SyntectColor { r: 40, g: 60, b: 100, a: 255 }), // Darker selection
                inactive_selection: Some(SyntectColor { r: 60, g: 80, b: 120, a: 100 }), // Muted selection
                inactive_selection_foreground: Some(SyntectColor { r: 150, g: 150, b: 150, a: 255 }), // Muted foreground
                guide: Some(SyntectColor { r: 70, g: 70, b: 70, a: 255 }), // Dark gray
                active_guide: Some(SyntectColor { r: 120, g: 120, b: 120, a: 255 }), // Lighter gray
                stack_guide: Some(SyntectColor { r: 70, g: 70, b: 70, a: 255 }), // Dark gray
                shadow: Some(SyntectColor { r: 0, g: 0, b: 0, a: 100 }), // Black with transparency
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
                    a: 255,
                }),
                foreground: Some(SyntectColor {
                    r: 235,
                    g: 219,
                    b: 178,
                    a: 255,
                }),
                caret: Some(SyntectColor {
                    r: 250,
                    g: 189,
                    b: 47,
                    a: 255,
                }),
                selection: Some(SyntectColor {
                    r: 60,
                    g: 56,
                    b: 54,
                    a: 255,
                }),
                line_highlight: Some(SyntectColor {
                    r: 50,
                    g: 50,
                    b: 50,
                    a: 255,
                }), // Slightly lighter background
                misspelling: Some(SyntectColor {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }), // Red
                minimap_border: Some(SyntectColor {
                    r: 70,
                    g: 70,
                    b: 70,
                    a: 255,
                }), // Dark gray
                accent: Some(SyntectColor {
                    r: 250,
                    g: 189,
                    b: 47,
                    a: 255,
                }), // Orange (same as caret)
                popup_css: Some("".to_string()),
                phantom_css: Some("".to_string()),
                bracket_contents_foreground: Some(SyntectColor { r: 235, g: 219, b: 178, a: 255 }),
                bracket_contents_options: Some(UnderlineOption::default()),
                brackets_foreground: Some(SyntectColor { r: 235, g: 219, b: 178, a: 255 }),
                brackets_background: Some(SyntectColor { r: 40, g: 40, b: 40, a: 255 }),
                brackets_options: Some(UnderlineOption::default()),
                tags_foreground: Some(SyntectColor { r: 235, g: 219, b: 178, a: 255 }),
                tags_options: Some(UnderlineOption::default()),
                highlight: Some(SyntectColor { r: 255, g: 255, b: 150, a: 255 }), // Light yellow
                find_highlight: Some(SyntectColor { r: 255, g: 255, b: 0, a: 255 }), // Yellow
                find_highlight_foreground: Some(SyntectColor { r: 0, g: 0, b: 0, a: 255 }), // Black
                gutter: Some(SyntectColor { r: 35, g: 35, b: 35, a: 255 }), // Slightly darker background
                gutter_foreground: Some(SyntectColor { r: 120, g: 120, b: 120, a: 255 }), // Muted gray
                selection_foreground: Some(SyntectColor { r: 0, g: 0, b: 0, a: 255 }), // Black
                selection_border: Some(SyntectColor { r: 50, g: 46, b: 44, a: 255 }), // Darker selection
                inactive_selection: Some(SyntectColor { r: 60, g: 56, b: 54, a: 100 }), // Muted selection
                inactive_selection_foreground: Some(SyntectColor { r: 180, g: 160, b: 130, a: 255 }), // Muted foreground
                guide: Some(SyntectColor { r: 80, g: 80, b: 80, a: 255 }), // Dark gray
                active_guide: Some(SyntectColor { r: 130, g: 130, b: 130, a: 255 }), // Lighter gray
                stack_guide: Some(SyntectColor { r: 80, g: 80, b: 80, a: 255 }), // Dark gray
                shadow: Some(SyntectColor { r: 0, g: 0, b: 0, a: 100 }), // Black with transparency
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
                misspelling: Some(SyntectColor {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }), // Red
                minimap_border: Some(SyntectColor {
                    r: 70,
                    g: 70,
                    b: 70,
                    a: 255,
                }), // Dark gray
                accent: Some(SyntectColor {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }), // Red (same as caret)
                popup_css: Some("".to_string()),
                phantom_css: Some("".to_string()),
                bracket_contents_foreground: Some(SyntectColor { r: 255, g: 255, b: 255, a: 255 }),
                bracket_contents_options: Some(UnderlineOption::default()),
                brackets_foreground: Some(SyntectColor { r: 255, g: 255, b: 255, a: 255 }),
                brackets_background: Some(SyntectColor { r: 0, g: 0, b: 0, a: 255 }),
                brackets_options: Some(UnderlineOption::default()),
                tags_foreground: Some(SyntectColor { r: 255, g: 255, b: 255, a: 255 }),
                tags_options: Some(UnderlineOption::default()),
                highlight: Some(SyntectColor { r: 0, g: 255, b: 255, a: 255 }), // Cyan
                find_highlight: Some(SyntectColor { r: 255, g: 255, b: 0, a: 255 }), // Yellow
                find_highlight_foreground: Some(SyntectColor { r: 0, g: 0, b: 0, a: 255 }), // Black
                gutter: Some(SyntectColor { r: 20, g: 20, b: 20, a: 255 }), // Slightly darker background
                gutter_foreground: Some(SyntectColor { r: 150, g: 150, b: 150, a: 255 }), // Muted gray
                selection_foreground: Some(SyntectColor { r: 255, g: 255, b: 255, a: 255 }), // White (contrasting with blue selection)
                selection_border: Some(SyntectColor { r: 0, g: 0, b: 150, a: 255 }), // Darker selection
                inactive_selection: Some(SyntectColor { r: 0, g: 0, b: 255, a: 50 }), // Muted selection
                inactive_selection_foreground: Some(SyntectColor { r: 200, g: 200, b: 200, a: 255 }), // Muted foreground
                guide: Some(SyntectColor { r: 80, g: 80, b: 80, a: 255 }), // Dark gray
                active_guide: Some(SyntectColor { r: 130, g: 130, b: 130, a: 255 }), // Lighter gray
                stack_guide: Some(SyntectColor { r: 80, g: 80, b: 80, a: 255 }), // Dark gray
                shadow: Some(SyntectColor { r: 0, g: 0, b: 0, a: 100 }), // Black with transparency
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
                misspelling: Some(SyntectColor {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }), // Red
                minimap_border: Some(SyntectColor {
                    r: 60,
                    g: 60,
                    b: 60,
                    a: 255,
                }), // Dark gray
                accent: Some(SyntectColor {
                    r: 255,
                    g: 100,
                    b: 100,
                    a: 255,
                }), // Soft red (same as caret)
                popup_css: Some("".to_string()),
                phantom_css: Some("".to_string()),
                bracket_contents_foreground: Some(SyntectColor { r: 220, g: 220, b: 220, a: 255 }),
                bracket_contents_options: Some(UnderlineOption::default()),
                brackets_foreground: Some(SyntectColor { r: 220, g: 220, b: 220, a: 255 }),
                brackets_background: Some(SyntectColor { r: 20, g: 20, b: 20, a: 255 }),
                brackets_options: Some(UnderlineOption::default()),
                tags_foreground: Some(SyntectColor { r: 220, g: 220, b: 220, a: 255 }),
                tags_options: Some(UnderlineOption::default()),
                highlight: Some(SyntectColor { r: 100, g: 100, b: 200, a: 255 }), // Light blue
                find_highlight: Some(SyntectColor { r: 255, g: 255, b: 0, a: 255 }), // Yellow
                find_highlight_foreground: Some(SyntectColor { r: 0, g: 0, b: 0, a: 255 }), // Black
                gutter: Some(SyntectColor { r: 15, g: 15, b: 15, a: 255 }), // Slightly darker background
                gutter_foreground: Some(SyntectColor { r: 100, g: 100, b: 100, a: 255 }), // Muted gray
                selection_foreground: Some(SyntectColor { r: 0, g: 0, b: 0, a: 255 }), // Black
                selection_border: Some(SyntectColor { r: 50, g: 50, b: 120, a: 255 }), // Darker selection
                inactive_selection: Some(SyntectColor { r: 70, g: 70, b: 150, a: 75 }), // Muted selection
                inactive_selection_foreground: Some(SyntectColor { r: 180, g: 180, b: 180, a: 255 }), // Muted foreground
                guide: Some(SyntectColor { r: 80, g: 80, b: 80, a: 255 }), // Dark gray
                active_guide: Some(SyntectColor { r: 130, g: 130, b: 130, a: 255 }), // Lighter gray
                stack_guide: Some(SyntectColor { r: 80, g: 80, b: 80, a: 255 }), // Dark gray
                shadow: Some(SyntectColor { r: 0, g: 0, b: 0, a: 100 }), // Black with transparency
            },
            ..Theme::default()
        },
    )
}

fn oceanic_theme() -> (String, Theme) {
    (
        "Oceanic".to_string(),
        Theme {
            name: Some("Oceanic".to_string()),
            settings: ThemeSettings {
                background: Some(SyntectColor {
                    r: 23,
                    g: 32,
                    b: 42,
                    a: 255,
                }), // Dark Teal/Navy
                foreground: Some(SyntectColor {
                    r: 171,
                    g: 205,
                    b: 239,
                    a: 255,
                }), // Light Blue
                caret: Some(SyntectColor {
                    r: 255,
                    g: 230,
                    b: 0,
                    a: 255,
                }), // Bright Yellow
                selection: Some(SyntectColor {
                    r: 60,
                    g: 90,
                    b: 120,
                    a: 150,
                }), // Semi-transparent darker blue
                line_highlight: Some(SyntectColor {
                    r: 35,
                    g: 45,
                    b: 55,
                    a: 255,
                }), // Slightly lighter background
                misspelling: Some(SyntectColor {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }), // Red
                minimap_border: Some(SyntectColor {
                    r: 70,
                    g: 80,
                    b: 90,
                    a: 255,
                }), // Dark gray
                accent: Some(SyntectColor {
                    r: 255,
                    g: 230,
                    b: 0,
                    a: 255,
                }), // Bright Yellow (same as caret)
                popup_css: Some("".to_string()),
                phantom_css: Some("".to_string()),
                bracket_contents_foreground: Some(SyntectColor { r: 171, g: 205, b: 239, a: 255 }),
                bracket_contents_options: Some(UnderlineOption::default()),
                brackets_foreground: Some(SyntectColor { r: 171, g: 205, b: 239, a: 255 }),
                brackets_background: Some(SyntectColor { r: 23, g: 32, b: 42, a: 255 }),
                brackets_options: Some(UnderlineOption::default()),
                tags_foreground: Some(SyntectColor { r: 171, g: 205, b: 239, a: 255 }),
                tags_options: Some(UnderlineOption::default()),
                highlight: Some(SyntectColor { r: 150, g: 255, b: 150, a: 255 }), // Light green
                find_highlight: Some(SyntectColor { r: 255, g: 165, b: 0, a: 255 }), // Orange
                find_highlight_foreground: Some(SyntectColor { r: 0, g: 0, b: 0, a: 255 }), // Black
                gutter: Some(SyntectColor {
                    r: 28,
                    g: 38,
                    b: 48,
                    a: 255,
                }), // Even darker background
                gutter_foreground: Some(SyntectColor {
                    r: 100,
                    g: 120,
                    b: 140,
                    a: 255,
                }), // Muted blue-gray
                selection_foreground: Some(SyntectColor { r: 0, g: 0, b: 0, a: 255 }), // Black
                selection_border: Some(SyntectColor { r: 40, g: 70, b: 100, a: 255 }), // Darker selection
                inactive_selection: Some(SyntectColor { r: 60, g: 90, b: 120, a: 75 }), // Muted selection
                inactive_selection_foreground: Some(SyntectColor { r: 120, g: 150, b: 180, a: 255 }), // Muted foreground
                guide: Some(SyntectColor { r: 80, g: 100, b: 120, a: 255 }), // Muted blue-gray
                active_guide: Some(SyntectColor { r: 130, g: 150, b: 170, a: 255 }), // Lighter blue-gray
                stack_guide: Some(SyntectColor { r: 80, g: 100, b: 120, a: 255 }), // Muted blue-gray
                shadow: Some(SyntectColor { r: 0, g: 0, b: 0, a: 100 }), // Black with transparency
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
        oceanic_theme(), // 新しいテーマを追加
    ]
}
