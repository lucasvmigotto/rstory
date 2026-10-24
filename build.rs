//! Decodes the tray icons (`assets/icons/png/<size>/dark/<state>.png`) into raw ARGB32 files in
//! `OUT_DIR`, so the daemon embeds pixel data and never looks an icon theme up (spec 009, FR-002).
//! `dark` is the variant for dark panels; see the spec's Doubts for light panels.

use std::env;
use std::fs;
use std::path::PathBuf;

const SIZES: [u32; 5] = [16, 22, 24, 32, 48];
const STATES: [&str; 3] = ["recording", "paused", "disconnected"];

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icons/png");
    let mut table = String::from("pub static ICON_DATA: &[(&str, i32, &[u8])] = &[\n");
    for state in STATES {
        for size in SIZES {
            let path = root.join(format!("assets/icons/png/{size}/dark/{state}.png"));
            let argb = decode(&path, size);
            let name = format!("{state}_{size}.argb");
            fs::write(out.join(&name), argb).expect("write argb");
            table.push_str(&format!(
                "    (\"{state}\", {size}, include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{name}\"))),\n"
            ));
        }
    }
    table.push_str("];\n");
    fs::write(out.join("icons_data.rs"), table).expect("write table");
}

/// Reads an 8-bit PNG of `size` x `size` and returns its pixels as A,R,G,B bytes.
fn decode(path: &std::path::Path, size: u32) -> Vec<u8> {
    let file = fs::File::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().expect("png header");
    let mut buf = vec![0; reader.output_buffer_size().expect("png size")];
    let info = reader.next_frame(&mut buf).expect("png frame");
    assert_eq!(
        (info.width, info.height),
        (size, size),
        "{}",
        path.display()
    );
    let data = &buf[..info.buffer_size()];
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    match info.color_type {
        png::ColorType::Rgba => {
            for p in data.as_chunks::<4>().0 {
                out.extend_from_slice(&[p[3], p[0], p[1], p[2]]);
            }
        }
        png::ColorType::Rgb => {
            for p in data.as_chunks::<3>().0 {
                out.extend_from_slice(&[255, p[0], p[1], p[2]]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for p in data.as_chunks::<2>().0 {
                out.extend_from_slice(&[p[1], p[0], p[0], p[0]]);
            }
        }
        png::ColorType::Grayscale => {
            for &g in data {
                out.extend_from_slice(&[255, g, g, g]);
            }
        }
        png::ColorType::Indexed => panic!("{}: indexed PNG not expanded", path.display()),
    }
    out
}
