use egui::Color32;

pub struct Theme {
    pub name: &'static str,
    pub bg: Color32,
    pub panel: Color32,
    pub surface: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub accent: Color32,
    pub header_bg: Color32,
    pub selected_bg: Color32,
}

pub const THEMES: &[Theme] = &[
    Theme {
        name: "Dark",
        bg: Color32::from_rgb(22, 22, 28),
        panel: Color32::from_rgb(28, 28, 35),
        surface: Color32::from_rgb(35, 35, 44),
        border: Color32::from_rgb(50, 50, 60),
        text: Color32::from_rgb(220, 220, 230),
        text_dim: Color32::from_rgb(130, 130, 140),
        accent: Color32::from_rgb(0, 180, 240),
        header_bg: Color32::from_rgb(30, 30, 38),
        selected_bg: Color32::from_rgba_premultiplied(0, 100, 180, 50),
    },
    Theme {
        name: "Midnight",
        bg: Color32::from_rgb(15, 12, 25),
        panel: Color32::from_rgb(22, 18, 35),
        surface: Color32::from_rgb(30, 25, 45),
        border: Color32::from_rgb(50, 40, 70),
        text: Color32::from_rgb(210, 200, 240),
        text_dim: Color32::from_rgb(120, 110, 150),
        accent: Color32::from_rgb(140, 100, 255),
        header_bg: Color32::from_rgb(25, 20, 40),
        selected_bg: Color32::from_rgba_premultiplied(100, 60, 220, 50),
    },
    Theme {
        name: "Ocean",
        bg: Color32::from_rgb(12, 20, 30),
        panel: Color32::from_rgb(18, 28, 42),
        surface: Color32::from_rgb(22, 35, 52),
        border: Color32::from_rgb(35, 55, 80),
        text: Color32::from_rgb(200, 220, 240),
        text_dim: Color32::from_rgb(110, 140, 170),
        accent: Color32::from_rgb(40, 180, 220),
        header_bg: Color32::from_rgb(15, 25, 38),
        selected_bg: Color32::from_rgba_premultiplied(30, 140, 200, 50),
    },
    Theme {
        name: "Forest",
        bg: Color32::from_rgb(15, 22, 15),
        panel: Color32::from_rgb(22, 30, 22),
        surface: Color32::from_rgb(28, 38, 28),
        border: Color32::from_rgb(40, 55, 40),
        text: Color32::from_rgb(200, 230, 200),
        text_dim: Color32::from_rgb(110, 140, 110),
        accent: Color32::from_rgb(60, 200, 100),
        header_bg: Color32::from_rgb(18, 26, 18),
        selected_bg: Color32::from_rgba_premultiplied(40, 160, 80, 50),
    },
    Theme {
        name: "Light",
        bg: Color32::from_rgb(240, 240, 245),
        panel: Color32::from_rgb(230, 230, 238),
        surface: Color32::from_rgb(255, 255, 255),
        border: Color32::from_rgb(200, 200, 210),
        text: Color32::from_rgb(30, 30, 35),
        text_dim: Color32::from_rgb(100, 100, 110),
        accent: Color32::from_rgb(0, 120, 200),
        header_bg: Color32::from_rgb(225, 225, 232),
        selected_bg: Color32::from_rgba_premultiplied(0, 100, 180, 40),
    },
];

pub fn get_theme(index: usize) -> &'static Theme {
    &THEMES[index.min(THEMES.len() - 1)]
}
