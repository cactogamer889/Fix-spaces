use std::collections::HashMap;
use egui::Color32;

pub struct ColorMap {
    ext_colors: HashMap<String, Color32>,
    category_colors: HashMap<String, Color32>,
    default_color: Color32,
}

impl ColorMap {
    pub fn new() -> Self {
        let mut ext_colors = HashMap::new();
        let mut category_colors = HashMap::new();

        let blue = Color32::from_rgb(70, 130, 200);
        let green = Color32::from_rgb(80, 180, 80);
        let red = Color32::from_rgb(200, 80, 80);
        let orange = Color32::from_rgb(220, 150, 50);
        let purple = Color32::from_rgb(150, 80, 200);
        let teal = Color32::from_rgb(60, 180, 180);
        let pink = Color32::from_rgb(220, 100, 160);
        let yellow = Color32::from_rgb(210, 200, 60);
        let brown = Color32::from_rgb(160, 120, 60);
        let gray = Color32::from_rgb(140, 140, 140);
        let dark_blue = Color32::from_rgb(40, 80, 160);
        let lime = Color32::from_rgb(120, 200, 60);

        category_colors.insert("video".into(), blue);
        category_colors.insert("audio".into(), purple);
        category_colors.insert("image".into(), green);
        category_colors.insert("document".into(), orange);
        category_colors.insert("archive".into(), red);
        category_colors.insert("code".into(), teal);
        category_colors.insert("executable".into(), pink);
        category_colors.insert("data".into(), yellow);
        category_colors.insert("font".into(), brown);
        category_colors.insert("3d".into(), lime);
        category_colors.insert("disk".into(), dark_blue);

        ext_colors.insert("mp4".into(), blue);
        ext_colors.insert("mkv".into(), blue);
        ext_colors.insert("avi".into(), blue);
        ext_colors.insert("mov".into(), blue);
        ext_colors.insert("wmv".into(), blue);
        ext_colors.insert("flv".into(), blue);
        ext_colors.insert("webm".into(), blue);
        ext_colors.insert("mpg".into(), blue);
        ext_colors.insert("mpeg".into(), blue);
        ext_colors.insert("m4v".into(), blue);

        ext_colors.insert("mp3".into(), purple);
        ext_colors.insert("wav".into(), purple);
        ext_colors.insert("flac".into(), purple);
        ext_colors.insert("aac".into(), purple);
        ext_colors.insert("ogg".into(), purple);
        ext_colors.insert("wma".into(), purple);
        ext_colors.insert("m4a".into(), purple);
        ext_colors.insert("opus".into(), purple);

        ext_colors.insert("jpg".into(), green);
        ext_colors.insert("jpeg".into(), green);
        ext_colors.insert("png".into(), green);
        ext_colors.insert("gif".into(), green);
        ext_colors.insert("bmp".into(), green);
        ext_colors.insert("svg".into(), green);
        ext_colors.insert("webp".into(), green);
        ext_colors.insert("ico".into(), green);
        ext_colors.insert("tiff".into(), green);
        ext_colors.insert("tif".into(), green);
        ext_colors.insert("psd".into(), green);
        ext_colors.insert("ai".into(), green);
        ext_colors.insert("raw".into(), green);
        ext_colors.insert("cr2".into(), green);
        ext_colors.insert("nef".into(), green);

        ext_colors.insert("pdf".into(), orange);
        ext_colors.insert("doc".into(), orange);
        ext_colors.insert("docx".into(), orange);
        ext_colors.insert("xls".into(), orange);
        ext_colors.insert("xlsx".into(), orange);
        ext_colors.insert("ppt".into(), orange);
        ext_colors.insert("pptx".into(), orange);
        ext_colors.insert("odt".into(), orange);
        ext_colors.insert("ods".into(), orange);
        ext_colors.insert("odp".into(), orange);
        ext_colors.insert("txt".into(), orange);
        ext_colors.insert("rtf".into(), orange);
        ext_colors.insert("csv".into(), orange);

        ext_colors.insert("zip".into(), red);
        ext_colors.insert("rar".into(), red);
        ext_colors.insert("7z".into(), red);
        ext_colors.insert("tar".into(), red);
        ext_colors.insert("gz".into(), red);
        ext_colors.insert("bz2".into(), red);
        ext_colors.insert("xz".into(), red);
        ext_colors.insert("iso".into(), red);
        ext_colors.insert("cab".into(), red);

        ext_colors.insert("rs".into(), teal);
        ext_colors.insert("py".into(), teal);
        ext_colors.insert("js".into(), teal);
        ext_colors.insert("ts".into(), teal);
        ext_colors.insert("jsx".into(), teal);
        ext_colors.insert("tsx".into(), teal);
        ext_colors.insert("java".into(), teal);
        ext_colors.insert("c".into(), teal);
        ext_colors.insert("cpp".into(), teal);
        ext_colors.insert("h".into(), teal);
        ext_colors.insert("cs".into(), teal);
        ext_colors.insert("go".into(), teal);
        ext_colors.insert("rb".into(), teal);
        ext_colors.insert("php".into(), teal);
        ext_colors.insert("swift".into(), teal);
        ext_colors.insert("kt".into(), teal);
        ext_colors.insert("html".into(), teal);
        ext_colors.insert("css".into(), teal);
        ext_colors.insert("scss".into(), teal);
        ext_colors.insert("json".into(), teal);
        ext_colors.insert("xml".into(), teal);
        ext_colors.insert("yaml".into(), teal);
        ext_colors.insert("toml".into(), teal);
        ext_colors.insert("sql".into(), teal);

        ext_colors.insert("exe".into(), pink);
        ext_colors.insert("msi".into(), pink);
        ext_colors.insert("dll".into(), pink);
        ext_colors.insert("sys".into(), pink);
        ext_colors.insert("so".into(), pink);
        ext_colors.insert("dylib".into(), pink);
        ext_colors.insert("app".into(), pink);
        ext_colors.insert("deb".into(), pink);
        ext_colors.insert("rpm".into(), pink);

        ext_colors.insert("db".into(), yellow);
        ext_colors.insert("sqlite".into(), yellow);
        ext_colors.insert("sqlite3".into(), yellow);
        ext_colors.insert("mdb".into(), yellow);

        ext_colors.insert("ttf".into(), brown);
        ext_colors.insert("otf".into(), brown);
        ext_colors.insert("woff".into(), brown);
        ext_colors.insert("woff2".into(), brown);
        ext_colors.insert("eot".into(), brown);

        ext_colors.insert("stl".into(), lime);
        ext_colors.insert("obj".into(), lime);
        ext_colors.insert("fbx".into(), lime);
        ext_colors.insert("blend".into(), lime);

        ext_colors.insert("vmdk".into(), dark_blue);
        ext_colors.insert("vhd".into(), dark_blue);
        ext_colors.insert("vhdx".into(), dark_blue);
        ext_colors.insert("img".into(), dark_blue);

        Self {
            ext_colors,
            category_colors,
            default_color: gray,
        }
    }

    pub fn get_color(&self, extension: &str) -> Color32 {
        self.ext_colors
            .get(extension)
            .copied()
            .unwrap_or(self.default_color)
    }

    pub fn get_category_color(&self, category: &str) -> Color32 {
        self.category_colors
            .get(category)
            .copied()
            .unwrap_or(self.default_color)
    }

    pub fn default_color(&self) -> Color32 {
        self.default_color
    }
}
