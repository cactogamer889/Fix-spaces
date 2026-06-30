pub mod renderer;

use egui::Rect;

#[derive(Debug, Clone)]
pub struct TreemapRect {
    pub rect: Rect,
    pub index: usize,
    pub label: String,
    pub size: u64,
}

pub fn squarify(items: &[(usize, u64, String)], area: Rect) -> Vec<TreemapRect> {
    if items.is_empty() {
        return Vec::new();
    }

    let total: u64 = items.iter().map(|(_, s, _)| *s).sum();
    if total == 0 {
        return Vec::new();
    }

    let mut sorted = items.to_vec();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let mut result = Vec::new();
    let mut remaining = area;

    let mut i = 0;
    while i < sorted.len() {
        let (idx, size, label) = &sorted[i];
        let fraction = *size as f64 / total as f64;

        let width = remaining.width() as f64;
        let height = remaining.height() as f64;

        let horizontal = width >= height;

        let mut row: Vec<(usize, u64, String, f64)> = Vec::new();
        let mut row_fraction = 0.0;

        while i < sorted.len() {
            let (idx2, size2, label2) = &sorted[i];
            let frac = *size2 as f64 / total as f64;
            let test_fraction = row_fraction + frac;

            let worst_before = if row.is_empty() {
                f64::MAX
            } else {
                worst_ratio(&row, row_fraction, horizontal, width, height)
            };
            let worst_after = worst_ratio_with(&row, (*idx2, *size2, label2.clone(), frac), test_fraction, horizontal, width, height);

            if row.is_empty() || worst_after <= worst_before {
                row.push((*idx2, *size2, label2.clone(), frac));
                row_fraction = test_fraction;
                i += 1;
            } else {
                break;
            }
        }

        let row_area;
        if horizontal {
            let row_width = (width * row_fraction) as f32;
            row_area = Rect::from_min_size(remaining.min, egui::vec2(row_width, remaining.height()));
            remaining = Rect::from_min_size(
                egui::pos2(remaining.min.x + row_width, remaining.min.y),
                egui::vec2(remaining.width() - row_width, remaining.height()),
            );
        } else {
            let row_height = (height * row_fraction) as f32;
            row_area = Rect::from_min_size(remaining.min, egui::vec2(remaining.width(), row_height));
            remaining = Rect::from_min_size(
                egui::pos2(remaining.min.x, remaining.min.y + row_height),
                egui::vec2(remaining.width(), remaining.height() - row_height),
            );
        }

        let mut offset = 0.0f32;
        for (idx2, size2, label2, frac) in &row {
            let sub_rect = if horizontal {
                let h = row_area.height() * (*frac as f32 / row_fraction as f32);
                let r = Rect::from_min_size(
                    egui::pos2(row_area.min.x, row_area.min.y + offset),
                    egui::vec2(row_area.width(), h),
                );
                offset += h;
                r
            } else {
                let w = row_area.width() * (*frac as f32 / row_fraction as f32);
                let r = Rect::from_min_size(
                    egui::pos2(row_area.min.x + offset, row_area.min.y),
                    egui::vec2(w, row_area.height()),
                );
                offset += w;
                r
            };

            result.push(TreemapRect {
                rect: sub_rect,
                index: *idx2,
                label: label2.clone(),
                size: *size2,
            });
        }
    }

    result
}

fn worst_ratio(row: &[(usize, u64, String, f64)], row_fraction: f64, horizontal: bool, width: f64, height: f64) -> f64 {
    let mut worst = 0.0f64;
    for (_, _, _, frac) in row {
        let ratio = if horizontal {
            let h = (frac / row_fraction) * height;
            let w = width * row_fraction;
            if h == 0.0 || w == 0.0 { f64::MAX } else { (w / h).max(h / w) }
        } else {
            let w = (frac / row_fraction) * width;
            let h = height * row_fraction;
            if h == 0.0 || w == 0.0 { f64::MAX } else { (w / h).max(h / w) }
        };
        worst = worst.max(ratio);
    }
    worst
}

fn worst_ratio_with(
    row: &[(usize, u64, String, f64)],
    new_item: (usize, u64, String, f64),
    test_fraction: f64,
    horizontal: bool,
    width: f64,
    height: f64,
) -> f64 {
    let mut worst = 0.0f64;
    for (_, _, _, frac) in row.iter().chain(std::iter::once(&new_item)) {
        let ratio = if horizontal {
            let h = (frac / test_fraction) * height;
            let w = width * test_fraction;
            if h == 0.0 || w == 0.0 { f64::MAX } else { (w / h).max(h / w) }
        } else {
            let w = (frac / test_fraction) * width;
            let h = height * test_fraction;
            if h == 0.0 || w == 0.0 { f64::MAX } else { (w / h).max(h / w) }
        };
        worst = worst.max(ratio);
    }
    worst
}
