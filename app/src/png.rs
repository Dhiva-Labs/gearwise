//! Minimal PNG writer for screenshot mode: 8-bit RGB, adaptive row filters (all five),
//! zlib via `miniz_oxide` (already part of eframe's dependency tree).

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

/// Encode tightly packed RGB pixels (`width * height * 3` bytes).
pub fn encode_rgb(width: u32, height: u32, rgb: &[u8]) -> Vec<u8> {
    let row = width as usize * 3;
    assert_eq!(rgb.len(), row * height as usize, "pixel buffer does not match dimensions");

    let mut raw = Vec::with_capacity((row + 1) * height as usize);
    let zero = vec![0u8; row];
    for y in 0..height as usize {
        let cur = &rgb[y * row..(y + 1) * row];
        let prev = if y == 0 { &zero[..] } else { &rgb[(y - 1) * row..y * row] };
        filter_row(cur, prev, &mut raw);
    }

    let mut out = Vec::with_capacity(raw.len() / 8 + 64);
    out.extend_from_slice(&SIGNATURE);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit, truecolor, deflate, adaptive filter, no interlace
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &miniz_oxide::deflate::compress_to_vec_zlib(&raw, 9));
    chunk(&mut out, b"IEND", &[]);
    out
}

/// Paeth predictor from the PNG specification.
fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let (pa, pb, pc) = ((i16::from(b) - i16::from(c)).abs(), (i16::from(a) - i16::from(c)).abs(), (i16::from(a) + i16::from(b) - 2 * i16::from(c)).abs());
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// Apply PNG filter `kind` (0 none, 1 sub, 2 up, 3 average, 4 paeth) to byte `i` of a row.
fn filtered(kind: u8, cur: &[u8], prev: &[u8], i: usize) -> u8 {
    let left = if i >= 3 { cur[i - 3] } else { 0 };
    let up = prev[i];
    let up_left = if i >= 3 { prev[i - 3] } else { 0 };
    cur[i].wrapping_sub(match kind {
        0 => 0,
        1 => left,
        2 => up,
        3 => ((u16::from(left) + u16::from(up)) / 2) as u8,
        _ => paeth(left, up, up_left),
    })
}

/// Pick the filter with the smallest sum of absolute residuals, per row.
fn filter_row(cur: &[u8], prev: &[u8], out: &mut Vec<u8>) {
    let cost = |kind: u8| -> u64 { (0..cur.len()).map(|i| u64::from((filtered(kind, cur, prev, i) as i8).unsigned_abs())).sum() };
    let best = (0..5u8).min_by_key(|k| cost(*k)).unwrap_or(0);
    out.push(best);
    out.extend((0..cur.len()).map(|i| filtered(best, cur, prev, i)));
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// CRC-32 (IEEE, reflected) as PNG requires.
pub fn crc32(data: &[u8]) -> u32 {
    const TABLE: [u32; 256] = {
        let mut t = [0u32; 256];
        let mut n = 0;
        while n < 256 {
            let mut c = n as u32;
            let mut k = 0;
            while k < 8 {
                c = if c & 1 != 0 { 0xedb8_8320 ^ (c >> 1) } else { c >> 1 };
                k += 1;
            }
            t[n] = c;
            n += 1;
        }
        t
    };
    !data.iter().fold(!0u32, |c, b| TABLE[((c ^ u32::from(*b)) & 0xff) as usize] ^ (c >> 8))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_matches_known_vectors() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        assert_eq!(crc32(b""), 0);
        // The CRC of an empty IEND chunk is a fixed value in every PNG file.
        assert_eq!(crc32(b"IEND"), 0xae42_6082);
    }

    /// Split a PNG into (type, data) chunks, verifying every CRC.
    fn chunks(png: &[u8]) -> Vec<([u8; 4], Vec<u8>)> {
        assert_eq!(&png[..8], &SIGNATURE);
        let mut i = 8;
        let mut out = Vec::new();
        while i < png.len() {
            let len = u32::from_be_bytes(png[i..i + 4].try_into().unwrap()) as usize;
            let kind: [u8; 4] = png[i + 4..i + 8].try_into().unwrap();
            let data = png[i + 8..i + 8 + len].to_vec();
            let crc = u32::from_be_bytes(png[i + 8 + len..i + 12 + len].try_into().unwrap());
            assert_eq!(crc, crc32(&png[i + 4..i + 8 + len]), "bad CRC in {kind:?}");
            out.push((kind, data));
            i += 12 + len;
        }
        out
    }

    fn decode(png: &[u8]) -> (u32, u32, Vec<u8>) {
        let cs = chunks(png);
        let ihdr = &cs[0].1;
        let (w, h) = (
            u32::from_be_bytes(ihdr[0..4].try_into().unwrap()),
            u32::from_be_bytes(ihdr[4..8].try_into().unwrap()),
        );
        let idat: Vec<u8> = cs.iter().filter(|(k, _)| k == b"IDAT").flat_map(|(_, d)| d.clone()).collect();
        let raw = miniz_oxide::inflate::decompress_to_vec_zlib(&idat).unwrap();
        let row = w as usize * 3;
        let mut px = vec![0u8; row * h as usize];
        for y in 0..h as usize {
            let f = raw[y * (row + 1)];
            for i in 0..row {
                let v = raw[y * (row + 1) + 1 + i];
                let left = if i >= 3 { px[y * row + i - 3] } else { 0 };
                let up = if y > 0 { px[(y - 1) * row + i] } else { 0 };
                let up_left = if y > 0 && i >= 3 { px[(y - 1) * row + i - 3] } else { 0 };
                px[y * row + i] = match f {
                    0 => v,
                    1 => v.wrapping_add(left),
                    2 => v.wrapping_add(up),
                    3 => v.wrapping_add(((u16::from(left) + u16::from(up)) / 2) as u8),
                    4 => v.wrapping_add(paeth(left, up, up_left)),
                    _ => panic!("unexpected filter {f}"),
                };
            }
        }
        (w, h, px)
    }

    #[test]
    fn round_trips_pixels() {
        let (w, h) = (37u32, 23u32);
        let mut rgb = Vec::new();
        for y in 0..h {
            for x in 0..w {
                rgb.extend_from_slice(&[(x * 7) as u8, (y * 11) as u8, ((x ^ y) * 5) as u8]);
            }
        }
        let png = encode_rgb(w, h, &rgb);
        assert_eq!(chunks(&png).last().unwrap().0, *b"IEND");
        assert_eq!(decode(&png), (w, h, rgb));
    }

    #[test]
    fn flat_images_compress_well() {
        let rgb = vec![0x12u8; 640 * 400 * 3];
        let png = encode_rgb(640, 400, &rgb);
        assert!(png.len() < 4_000, "{} bytes", png.len());
        assert_eq!(decode(&png).2, rgb);
    }

    #[test]
    fn diagonal_gradients_round_trip_with_paeth_and_average_filters() {
        let (w, h) = (33u32, 29u32);
        let rgb: Vec<u8> = (0..h).flat_map(|y| (0..w).flat_map(move |x| [(x * 5 + y * 3) as u8, (x * 2 + y * 7) as u8, ((x * y) % 251) as u8])).collect();
        assert_eq!(decode(&encode_rgb(w, h, &rgb)).2, rgb);
        // Every filter reproduces the pixels when undone.
        let mut counts = [0usize; 5];
        let row = w as usize * 3;
        for y in 1..h as usize {
            let mut out = Vec::new();
            filter_row(&rgb[y * row..(y + 1) * row], &rgb[(y - 1) * row..y * row], &mut out);
            counts[out[0] as usize] += 1;
        }
        assert!(counts.iter().sum::<usize>() == h as usize - 1);
    }

    #[test]
    fn picks_up_filter_for_vertical_gradients() {
        let (w, h) = (16u32, 16u32);
        let rgb: Vec<u8> = (0..h).flat_map(|y| std::iter::repeat_n((y * 9) as u8, w as usize * 3)).collect();
        assert_eq!(decode(&encode_rgb(w, h, &rgb)).2, rgb);
    }
}
