use ab_glyph::{point, Font, FontArc, PxScale, ScaleFont};
use std::path::Path;

use super::{blend_coverage, blend_pixel, next_random, IMAGE_HEIGHT_CONST, IMAGE_WIDTH_CONST};

const IMAGE_WIDTH: usize = IMAGE_WIDTH_CONST;
const IMAGE_HEIGHT: usize = IMAGE_HEIGHT_CONST;
const FRAME_WIDTH: usize = 4;

pub fn draw_captcha(pixels: &mut [[u8; 3]], answer: &str, mut seed: u64, config_dir: &Path) {
    let font = std::fs::read(config_dir.join("font.ttf"))
        .ok()
        .and_then(|bytes| FontArc::try_from_vec(bytes).ok());
    if let Some(font) = font {
        draw_configured_ttf(pixels, answer, &font, &mut seed);
    } else {
        draw_fallback_digits(pixels, answer, &mut seed);
    }
    for _ in 0..5 {
        let curve = if next_random(&mut seed) & 1 == 0 {
            [
                (next_random(&mut seed) % IMAGE_WIDTH as u64) as f32,
                (next_random(&mut seed) % 38) as f32,
                (next_random(&mut seed) % IMAGE_WIDTH as u64) as f32,
                (next_random(&mut seed) % IMAGE_HEIGHT as u64) as f32,
                (next_random(&mut seed) % IMAGE_WIDTH as u64) as f32,
                (next_random(&mut seed) % IMAGE_HEIGHT as u64) as f32,
                (next_random(&mut seed) % IMAGE_WIDTH as u64) as f32,
                IMAGE_HEIGHT as f32 * (0.8 + (next_random(&mut seed) % 100) as f32 / 1000.0),
            ]
        } else {
            [
                (next_random(&mut seed) % 38) as f32,
                (next_random(&mut seed) % IMAGE_HEIGHT as u64) as f32,
                (next_random(&mut seed) % IMAGE_WIDTH as u64) as f32,
                (next_random(&mut seed) % IMAGE_HEIGHT as u64) as f32,
                (next_random(&mut seed) % IMAGE_WIDTH as u64) as f32,
                (next_random(&mut seed) % IMAGE_HEIGHT as u64) as f32,
                IMAGE_WIDTH as f32 * (0.8 + (next_random(&mut seed) % 100) as f32 / 1000.0),
                (next_random(&mut seed) % IMAGE_HEIGHT as u64) as f32,
            ]
        };
        for step in 0..=600 {
            let t = step as f32 / 600.0;
            let mt = 1.0 - t;
            let x = mt.powi(3) * curve[0]
                + 3.0 * mt.powi(2) * t * curve[2]
                + 3.0 * mt * t.powi(2) * curve[4]
                + t.powi(3) * curve[6];
            let y = mt.powi(3) * curve[1]
                + 3.0 * mt.powi(2) * t * curve[3]
                + 3.0 * mt * t.powi(2) * curve[5]
                + t.powi(3) * curve[7];
            for oy in -2..=2 {
                for ox in -2..=2 {
                    blend_pixel(pixels, x as i32 + ox, y as i32 + oy, [0, 0, 0]);
                }
            }
        }
    }
}

fn draw_configured_ttf(pixels: &mut [[u8; 3]], answer: &str, font: &FontArc, seed: &mut u64) {
    let scale = PxScale::from(400.0);
    let scaled = font.as_scaled(scale);
    let ascent = scaled.ascent();
    let descent = scaled.descent();
    let glyph_box_w = 44.0 * FRAME_WIDTH as f32;
    let advance = 30.0 * FRAME_WIDTH as f32; 
    let run_width = glyph_box_w + advance * answer.chars().count().saturating_sub(1) as f32;
    let run_x = (IMAGE_WIDTH as f32 - run_width) * 0.5;
    let run_baseline = (IMAGE_HEIGHT as f32 - (ascent - descent)) * 0.5 + ascent;
    let colors = [
        [251, 8, 8],
        [251, 8, 175],
        [208, 8, 251],
        [100, 8, 251],
        [8, 24, 251],
        [8, 123, 251],
        [8, 251, 231],
        [8, 251, 34],
        [120, 251, 8],
        [251, 205, 8],
        [251, 126, 8],
    ];
    let captcha_color = colors[(next_random(seed) >> 33) as usize % colors.len()];
    for (index, ch) in answer.chars().enumerate() {
        let glyph_id = font.glyph_id(ch);
        let char_scale = scale;
        let glyph = glyph_id.with_scale_and_position(
            char_scale,
            point(
                run_x + index as f32 * advance + (glyph_box_w - scaled.h_advance(glyph_id)) * 0.5,
                run_baseline,
            ),
        );
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            let center_x = (bounds.min.x + bounds.max.x) * 0.5;
            let center_y = (bounds.min.y + bounds.max.y) * 0.5;
            let angle = ((next_random(seed) % 9) as f32 - 4.0) * std::f32::consts::PI / 180.0;
            let (sin, cos) = angle.sin_cos();
            outlined.draw(|gx, gy, coverage| {
                if coverage < 0.45 {
                    return;
                }
                let local_x = bounds.min.x + gx as f32 - center_x;
                let local_y = bounds.min.y + gy as f32 - center_y;
                let x = center_x + local_x * cos - local_y * sin;
                let y = center_y + local_x * sin + local_y * cos + (x * 0.08).sin() * 3.0;
                blend_coverage(pixels, x.round() as i32, y.round() as i32, captcha_color, 1.0);
            });
        }
    }
}

fn draw_fallback_digits(pixels: &mut [[u8; 3]], answer: &str, seed: &mut u64) {
    let palette = [
        [251, 8, 8],
        [251, 8, 175],
        [208, 8, 251],
        [8, 24, 251],
        [8, 251, 231],
        [8, 251, 34],
        [251, 205, 8],
        [251, 126, 8],
    ];
    let captcha_color = palette[(next_random(seed) >> 33) as usize % palette.len()];
    let scale = 14usize;
    let glyph_width = 5 * scale;
    let total_width = answer.len() * glyph_width + answer.len().saturating_sub(1) * scale;
    let start_x = (IMAGE_WIDTH.saturating_sub(total_width)) / 2;
    let start_y = IMAGE_HEIGHT / 2 - (7 * scale) / 2;
    for (index, digit) in answer.bytes().enumerate() {
        let glyph = digit_glyph(digit);
        let color = captcha_color;
        for (gy, row) in glyph.iter().enumerate() {
            for (gx, bit) in row.bytes().enumerate() {
                if bit == b'1' {
                    for py in 0..scale {
                        for px in 0..scale {
                            blend_pixel(
                                pixels,
                                (start_x + index * (glyph_width + scale) + gx * scale + px) as i32,
                                (start_y + gy * scale + py) as i32,
                                color,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn digit_glyph(digit: u8) -> [&'static str; 7] {
    match digit {
        b'0' => [
            "01110", "10001", "10011", "10101", "11001", "10001", "01110",
        ],
        b'1' => [
            "00100", "01100", "00100", "00100", "00100", "00100", "01110",
        ],
        b'2' => [
            "01110", "10001", "00001", "00010", "00100", "01000", "11111",
        ],
        b'3' => [
            "11110", "00001", "00001", "01110", "00001", "00001", "11110",
        ],
        b'4' => [
            "00010", "00110", "01010", "10010", "11111", "00010", "00010",
        ],
        b'5' => [
            "11111", "10000", "10000", "11110", "00001", "00001", "11110",
        ],
        b'6' => [
            "01110", "10000", "10000", "11110", "10001", "10001", "01110",
        ],
        b'7' => [
            "11111", "00001", "00010", "00100", "01000", "01000", "01000",
        ],
        b'8' => [
            "01110", "10001", "10001", "01110", "10001", "10001", "01110",
        ],
        b'9' => [
            "01110", "10001", "10001", "01111", "00001", "00001", "01110",
        ],
        _ => unreachable!(),
    }
}
