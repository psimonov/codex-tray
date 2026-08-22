use std::{
    env,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use ico::{IconDir, IconDirEntry, IconImage, ResourceType};

const APP_SIZES: &[u32] = &[16, 20, 24, 32, 40, 48, 64, 128, 256];
const TRAY_SIZES: &[u32] = &[16, 20, 24, 32];
const TRAY_STATES: &[(u32, u32)] = &[
    (100, 0),
    (101, 5),
    (102, 25),
    (103, 50),
    (104, 75),
    (105, 95),
    (106, 100),
];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/app-icon.svg");
    println!("cargo:rerun-if-changed=assets/tray-states.svg");

    if env::var_os("CARGO_CFG_WINDOWS").is_none() {
        return;
    }

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let app_icon = output.join("codex-tray.ico");
    write_icon(&app_icon, APP_SIZES, render_app_icon).expect("write app icon");

    let mut resource = format!("1 ICON \"{}\"\n", resource_path(&app_icon));
    for &(resource_id, percent) in TRAY_STATES {
        let path = output.join(format!("tray-{percent}.ico"));
        write_icon(&path, TRAY_SIZES, |size| render_tray_icon(size, percent))
            .expect("write tray icon");
        resource.push_str(&format!(
            "{resource_id} ICON \"{}\"\n",
            resource_path(&path)
        ));
    }
    for (resource_id, name, glyph) in [
        (107, "loading", StatusGlyph::Loading),
        (108, "error", StatusGlyph::Error),
        (109, "account", StatusGlyph::Account),
        (110, "missing", StatusGlyph::Missing),
    ] {
        let path = output.join(format!("tray-{name}.ico"));
        write_icon(&path, TRAY_SIZES, |size| render_status_icon(size, glyph))
            .expect("write status tray icon");
        resource.push_str(&format!(
            "{resource_id} ICON \"{}\"\n",
            resource_path(&path)
        ));
    }

    let resource_file = output.join("codex-tray.rc");
    fs::write(&resource_file, resource).expect("write resource script");
    embed_resource::compile(&resource_file, embed_resource::NONE)
        .manifest_optional()
        .expect("compile Windows icon resources");
}

fn write_icon(path: &Path, sizes: &[u32], render: impl Fn(u32) -> Vec<u8>) -> io::Result<()> {
    let mut directory = IconDir::new(ResourceType::Icon);
    for &size in sizes {
        let image = IconImage::from_rgba_data(size, size, render(size));
        directory.add_entry(IconDirEntry::encode(&image)?);
    }
    directory.write(File::create(path)?)
}

fn render_app_icon(size: u32) -> Vec<u8> {
    let mut canvas = Canvas::new(size);
    canvas.rounded_rect(12.0, 16.0, 244.0, 248.0, 48.0, rgba(18, 27, 37, 90));
    canvas.rounded_rect(12.0, 8.0, 244.0, 240.0, 48.0, rgba(47, 64, 83, 255));
    canvas.rounded_rect(30.0, 20.0, 226.0, 88.0, 34.0, rgba(59, 80, 102, 255));
    canvas.rounded_rect(46.0, 72.0, 210.0, 142.0, 20.0, rgba(23, 36, 49, 255));
    canvas.rounded_rect(56.0, 82.0, 200.0, 132.0, 14.0, rgba(237, 244, 247, 255));
    canvas.rounded_rect(63.0, 89.0, 187.0, 125.0, 9.0, rgba(56, 201, 176, 255));
    canvas.polygon(
        &[
            (62.0, 176.0),
            (73.0, 164.0),
            (98.0, 184.0),
            (73.0, 204.0),
            (62.0, 192.0),
            (73.0, 184.0),
        ],
        rgba(255, 180, 84, 255),
    );
    canvas.rounded_rect(105.0, 194.0, 177.0, 204.0, 5.0, rgba(220, 231, 238, 255));
    canvas.data
}

fn render_tray_icon(size: u32, percent: u32) -> Vec<u8> {
    let mut canvas = Canvas::new(size);
    let outline = rgba(244, 248, 250, 255);
    let well = rgba(28, 38, 50, 255);
    let fill = match percent {
        0 => rgba(255, 96, 119, 255),
        1..=9 => rgba(255, 159, 67, 255),
        10..=37 => rgba(242, 201, 76, 255),
        38..=62 => rgba(101, 199, 242, 255),
        _ => rgba(85, 214, 167, 255),
    };

    draw_tray_frame(&mut canvas, size, outline, well);

    if percent > 0 {
        let pixel = 256.0 / size as f32;
        let liquid_left = pixel * 3.0;
        let liquid_top = pixel * 3.0;
        let liquid_right = 256.0 - liquid_left;
        let liquid_bottom = 256.0 - liquid_top;
        let height = ((liquid_bottom - liquid_top) * percent as f32 / 100.0).max(pixel);
        let level = ((liquid_bottom - height) / pixel).round() * pixel;
        let liquid_radius = (22.0 / pixel).round().max(1.0) * pixel;
        canvas.paint(fill, |x, y| {
            y >= level
                && rounded_rect_contains(
                    x,
                    y,
                    liquid_left,
                    liquid_top,
                    liquid_right,
                    liquid_bottom,
                    liquid_radius,
                )
        });
    } else {
        canvas.line(82.0, 88.0, 174.0, 168.0, 18.0, fill);
        canvas.line(174.0, 88.0, 82.0, 168.0, 18.0, fill);
    }

    canvas.data
}

#[derive(Clone, Copy)]
enum StatusGlyph {
    Loading,
    Error,
    Account,
    Missing,
}

fn render_status_icon(size: u32, glyph: StatusGlyph) -> Vec<u8> {
    let mut canvas = Canvas::new(size);
    let outline = rgba(230, 237, 243, 255);
    let well = rgba(28, 38, 50, 255);
    draw_tray_frame(&mut canvas, size, outline, well);
    match glyph {
        StatusGlyph::Loading => {
            let color = rgba(101, 199, 242, 255);
            for left in [54.0, 112.0, 170.0] {
                canvas.rounded_rect(left, 104.0, left + 32.0, 136.0, 8.0, color);
            }
        }
        StatusGlyph::Error => {
            let color = rgba(255, 91, 110, 255);
            canvas.line(76.0, 82.0, 180.0, 174.0, 20.0, color);
            canvas.line(180.0, 82.0, 76.0, 174.0, 20.0, color);
        }
        StatusGlyph::Account => {
            let color = rgba(255, 181, 71, 255);
            canvas.rounded_rect(116.0, 82.0, 140.0, 142.0, 8.0, color);
            canvas.rounded_rect(116.0, 158.0, 140.0, 178.0, 7.0, color);
        }
        StatusGlyph::Missing => {
            let color = rgba(255, 181, 71, 255);
            canvas.rounded_rect(96.0, 76.0, 156.0, 96.0, 7.0, color);
            canvas.rounded_rect(144.0, 92.0, 164.0, 132.0, 7.0, color);
            canvas.rounded_rect(116.0, 124.0, 156.0, 144.0, 7.0, color);
            canvas.rounded_rect(116.0, 160.0, 140.0, 180.0, 7.0, color);
        }
    }
    canvas.data
}

fn draw_tray_frame(canvas: &mut Canvas, size: u32, outline: Color, well: Color) {
    let pixel = 256.0 / size as f32;
    let outer_radius = (46.0 / pixel).round().max(2.0) * pixel;
    let inner_radius = outer_radius - pixel;
    canvas.rounded_rect(
        pixel,
        pixel,
        256.0 - pixel,
        256.0 - pixel,
        outer_radius,
        outline,
    );
    canvas.rounded_rect(
        pixel * 2.0,
        pixel * 2.0,
        256.0 - pixel * 2.0,
        256.0 - pixel * 2.0,
        inner_radius,
        well,
    );
}

fn resource_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[derive(Clone, Copy)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
    Color {
        red,
        green,
        blue,
        alpha,
    }
}

struct Canvas {
    size: u32,
    data: Vec<u8>,
}

impl Canvas {
    fn new(size: u32) -> Self {
        Self {
            size,
            data: vec![0; (size * size * 4) as usize],
        }
    }

    fn rounded_rect(
        &mut self,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
        radius: f32,
        color: Color,
    ) {
        self.paint(color, |x, y| {
            rounded_rect_contains(x, y, left, top, right, bottom, radius)
        });
    }

    fn polygon(&mut self, points: &[(f32, f32)], color: Color) {
        self.paint(color, |x, y| point_in_polygon(x, y, points));
    }

    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Color) {
        self.paint(color, |x, y| {
            distance_to_segment_squared(x, y, x1, y1, x2, y2) <= (width / 2.0).powi(2)
        });
    }

    fn paint(&mut self, color: Color, contains: impl Fn(f32, f32) -> bool) {
        const SAMPLES: u32 = 4;
        for y in 0..self.size {
            for x in 0..self.size {
                let mut covered = 0;
                for sample_y in 0..SAMPLES {
                    for sample_x in 0..SAMPLES {
                        let px = (x as f32 + (sample_x as f32 + 0.5) / SAMPLES as f32) * 256.0
                            / self.size as f32;
                        let py = (y as f32 + (sample_y as f32 + 0.5) / SAMPLES as f32) * 256.0
                            / self.size as f32;
                        covered += contains(px, py) as u32;
                    }
                }

                if covered > 0 {
                    let coverage = covered as f32 / (SAMPLES * SAMPLES) as f32;
                    self.blend(x, y, color, coverage);
                }
            }
        }
    }

    fn blend(&mut self, x: u32, y: u32, color: Color, coverage: f32) {
        let index = ((y * self.size + x) * 4) as usize;
        let source_alpha = color.alpha as f32 / 255.0 * coverage;
        let destination_alpha = self.data[index + 3] as f32 / 255.0;
        let output_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
        if output_alpha <= f32::EPSILON {
            return;
        }

        for (channel, source) in [color.red, color.green, color.blue].into_iter().enumerate() {
            let destination = self.data[index + channel] as f32 / 255.0;
            let source = source as f32 / 255.0;
            self.data[index + channel] = (((source * source_alpha
                + destination * destination_alpha * (1.0 - source_alpha))
                / output_alpha)
                * 255.0)
                .round() as u8;
        }
        self.data[index + 3] = (output_alpha * 255.0).round() as u8;
    }
}

fn rounded_rect_contains(
    x: f32,
    y: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    radius: f32,
) -> bool {
    let nearest_x = x.clamp(left + radius, right - radius);
    let nearest_y = y.clamp(top + radius, bottom - radius);
    let dx = x - nearest_x;
    let dy = y - nearest_y;
    x >= left && x <= right && y >= top && y <= bottom && dx * dx + dy * dy <= radius * radius
}

fn point_in_polygon(x: f32, y: f32, points: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let mut previous = points.len() - 1;
    for current in 0..points.len() {
        let (xi, yi) = points[current];
        let (xj, yj) = points[previous];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

fn distance_to_segment_squared(x: f32, y: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let length_squared = dx * dx + dy * dy;
    let t = if length_squared == 0.0 {
        0.0
    } else {
        ((x - x1) * dx + (y - y1) * dy) / length_squared
    }
    .clamp(0.0, 1.0);
    let nearest_x = x1 + t * dx;
    let nearest_y = y1 + t * dy;
    (x - nearest_x).powi(2) + (y - nearest_y).powi(2)
}
