use std::io::Read;
use std::path::Path;

use flate2::read::ZlibDecoder;

pub fn read_png(path: &Path) -> anyhow::Result<(Vec<[u8; 3]>, usize, usize)> {
    let bytes = std::fs::read(path)?;
    anyhow::ensure!(
        bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "{} is not a PNG",
        path.display()
    );
    let mut cursor = 8;
    let (mut width, mut height, mut color_type) = (0usize, 0usize, 0u8);
    let mut palette = Vec::new();
    let mut transparency = Vec::new();
    let mut compressed = Vec::new();
    while cursor + 12 <= bytes.len() {
        let length = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into()?) as usize;
        let kind = &bytes[cursor + 4..cursor + 8];
        let start = cursor + 8;
        anyhow::ensure!(
            start + length + 4 <= bytes.len(),
            "truncated PNG chunk in {}",
            path.display()
        );
        let data = &bytes[start..start + length];
        match kind {
            b"IHDR" => {
                width = u32::from_be_bytes(data[0..4].try_into()?) as usize;
                height = u32::from_be_bytes(data[4..8].try_into()?) as usize;
                anyhow::ensure!(
                    data[8] == 8 && data[10] == 0 && data[11] == 0 && data[12] == 0,
                    "unsupported PNG encoding in {}",
                    path.display()
                );
                color_type = data[9];
            }
            b"PLTE" => palette.extend_from_slice(data),
            b"tRNS" => transparency.extend_from_slice(data),
            b"IDAT" => compressed.extend_from_slice(data),
            b"IEND" => break,
            _ => {}
        }
        cursor = start + length + 4;
    }
    anyhow::ensure!(
        width > 0 && height > 0 && width <= 8192 && height <= 8192,
        "invalid PNG dimensions"
    );
    let channels = match color_type {
        0 => 1,
        2 => 3,
        3 => 1,
        4 => 2,
        6 => 4,
        _ => anyhow::bail!("unsupported PNG color type {color_type}"),
    };
    let row_len = width
        .checked_mul(channels)
        .ok_or_else(|| anyhow::anyhow!("PNG dimensions overflow"))?;
    let expected = height
        .checked_mul(row_len + 1)
        .ok_or_else(|| anyhow::anyhow!("PNG dimensions overflow"))?;
    let decoder = ZlibDecoder::new(compressed.as_slice());
    let mut raw = Vec::with_capacity(expected);
    decoder.take(expected as u64 + 1).read_to_end(&mut raw)?;
    anyhow::ensure!(
        raw.len() == expected,
        "invalid decompressed PNG length in {}",
        path.display()
    );

    let mut decoded = vec![0u8; height * row_len];
    for y in 0..height {
        let raw_start = y * (row_len + 1);
        let filter = raw[raw_start];
        let row = &raw[raw_start + 1..raw_start + 1 + row_len];
        for x in 0..row_len {
            let left = if x >= channels {
                decoded[y * row_len + x - channels]
            } else {
                0
            };
            let above = if y > 0 {
                decoded[(y - 1) * row_len + x]
            } else {
                0
            };
            let upper_left = if y > 0 && x >= channels {
                decoded[(y - 1) * row_len + x - channels]
            } else {
                0
            };
            let predictor = match filter {
                0 => 0,
                1 => left,
                2 => above,
                3 => ((u16::from(left) + u16::from(above)) / 2) as u8,
                4 => paeth(left, above, upper_left),
                _ => anyhow::bail!("unsupported PNG filter {filter}"),
            };
            decoded[y * row_len + x] = row[x].wrapping_add(predictor);
        }
    }
    let mut pixels = Vec::with_capacity(width * height);
    for px in decoded.chunks_exact(channels) {
        let (rgb, alpha) = match color_type {
            0 => ([px[0], px[0], px[0]], 255),
            2 => ([px[0], px[1], px[2]], 255),
            3 => {
                let i = usize::from(px[0]);
                anyhow::ensure!(i * 3 + 2 < palette.len(), "invalid PNG palette index");
                (
                    [palette[i * 3], palette[i * 3 + 1], palette[i * 3 + 2]],
                    *transparency.get(i).unwrap_or(&255),
                )
            }
            4 => ([px[0], px[0], px[0]], px[1]),
            6 => ([px[0], px[1], px[2]], px[3]),
            _ => unreachable!(),
        };
        pixels.push(if alpha == 255 {
            rgb
        } else if alpha == 0 {
            [0, 0, 0]
        } else {
            let a = u16::from(alpha);
            [
                ((u16::from(rgb[0]) * a) / 255) as u8,
                ((u16::from(rgb[1]) * a) / 255) as u8,
                ((u16::from(rgb[2]) * a) / 255) as u8,
            ]
        });
    }
    Ok((pixels, width, height))
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let (a, b, c) = (i32::from(a), i32::from(b), i32::from(c));
    let p = a + b - c;
    let (pa, pb, pc) = ((p - a).abs(), (p - b).abs(), (p - c).abs());
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}
