pub mod draw;
pub mod palette;
pub mod png;

use std::path::Path;

pub use draw::draw_captcha;
pub use palette::map_palette_lut;

const MAP_SIDE: usize = 128;
const FRAME_WIDTH: usize = 4;
const FRAME_HEIGHT: usize = 3;
const IMAGE_WIDTH: usize = MAP_SIDE * FRAME_WIDTH;
const IMAGE_HEIGHT: usize = MAP_SIDE * FRAME_HEIGHT;

pub fn render_captcha(
    answer: &str,
    background_dir: &Path,
    salt: u64,
    background_count: usize,
) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut backgrounds: Vec<_> = std::fs::read_dir(background_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|s| s.to_str()).is_some_and(|s| s.eq_ignore_ascii_case("png")))
        .collect();
    backgrounds.sort();
    anyhow::ensure!(!backgrounds.is_empty(), "no PNG CAPTCHA backgrounds found in {}", background_dir.display());
    let count = background_count.max(1).min(backgrounds.len());
    let (mut pixels, source_width, source_height) =
        png::read_png(&backgrounds[salt as usize % count])?;
    resize(
        &mut pixels,
        source_width,
        source_height,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
    );
    draw::draw_captcha(&mut pixels, answer, salt, background_dir);

    let lut = palette::map_palette_lut();
    let mut maps = vec![vec![0; MAP_SIDE * MAP_SIDE]; FRAME_WIDTH * FRAME_HEIGHT];
    for map_y in 0..FRAME_HEIGHT {
        for map_x in 0..FRAME_WIDTH {
            let map_id = map_y * FRAME_WIDTH + map_x;
            let source_tile = FRAME_WIDTH * FRAME_HEIGHT - 1 - map_id;
            let source_y = source_tile / FRAME_WIDTH;
            let source_x = source_tile % FRAME_WIDTH;
            for y in 0..MAP_SIDE {
                for x in 0..MAP_SIDE {
                    let color =
                        pixels[(source_y * MAP_SIDE + y) * IMAGE_WIDTH + source_x * MAP_SIDE + x];
                    let key = ((usize::from(color[0] >> 3)) << 10)
                        | ((usize::from(color[1] >> 3)) << 5)
                        | usize::from(color[2] >> 3);
                    maps[map_id][y * MAP_SIDE + x] = lut[key];
                }
            }
        }
    }
    Ok(maps)
}

pub(crate) fn resize(
    pixels: &mut Vec<[u8; 3]>,
    source_width: usize,
    source_height: usize,
    width: usize,
    height: usize,
) {
    let source = pixels.clone();
    let mut target = vec![[0; 3]; width * height];
    for y in 0..height {
        let syf = ((y as f32 + 0.5) * source_height as f32 / height as f32 - 0.5)
            .clamp(0.0, (source_height - 1) as f32);
        let sy0 = syf.floor() as usize;
        let sy1 = (sy0 + 1).min(source_height - 1);
        let fy = syf - sy0 as f32;
        for x in 0..width {
            let sxf = ((x as f32 + 0.5) * source_width as f32 / width as f32 - 0.5)
                .clamp(0.0, (source_width - 1) as f32);
            let sx0 = sxf.floor() as usize;
            let sx1 = (sx0 + 1).min(source_width - 1);
            let fx = sxf - sx0 as f32;
            let samples = [
                source[sy0 * source_width + sx0],
                source[sy0 * source_width + sx1],
                source[sy1 * source_width + sx0],
                source[sy1 * source_width + sx1],
            ];
            for channel in 0..3 {
                let top = f32::from(samples[0][channel]) * (1.0 - fx)
                    + f32::from(samples[1][channel]) * fx;
                let bottom = f32::from(samples[2][channel]) * (1.0 - fx)
                    + f32::from(samples[3][channel]) * fx;
                target[y * width + x][channel] = (top * (1.0 - fy) + bottom * fy).round() as u8;
            }
        }
    }
    *pixels = target;
}

pub(crate) fn blend_coverage(pixels: &mut [[u8; 3]], x: i32, y: i32, color: [u8; 3], coverage: f32) {
    if x < 0 || y < 0 || x >= IMAGE_WIDTH as i32 || y >= IMAGE_HEIGHT as i32 {
        return;
    }
    let pixel = &mut pixels[y as usize * IMAGE_WIDTH + x as usize];
    let alpha = coverage.clamp(0.0, 1.0);
    for channel in 0..3 {
        pixel[channel] = (f32::from(pixel[channel]) * (1.0 - alpha)
            + f32::from(color[channel]) * alpha)
            .round() as u8;
    }
}

pub(crate) fn blend_pixel(pixels: &mut [[u8; 3]], x: i32, y: i32, color: [u8; 3]) {
    if x < 0 || y < 0 || x >= IMAGE_WIDTH as i32 || y >= IMAGE_HEIGHT as i32 {
        return;
    }
    let pixel = &mut pixels[y as usize * IMAGE_WIDTH + x as usize];
    for i in 0..3 {
        pixel[i] = ((u16::from(pixel[i]) + u16::from(color[i])) / 2) as u8;
    }
}

pub(crate) fn next_random(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *seed
}

pub(crate) const IMAGE_WIDTH_CONST: usize = IMAGE_WIDTH;
pub(crate) const IMAGE_HEIGHT_CONST: usize = IMAGE_HEIGHT;
