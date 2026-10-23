//! Tray icons as ARGB32 pixel data (FR-002), decoded from `assets/icons/png` by `build.rs`.
//! No icon theme lookup: the pixels travel in the SNI `IconPixmap` property.

include!(concat!(env!("OUT_DIR"), "/icons_data.rs"));

/// Which icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconState {
    /// Capturing.
    Recording,
    /// Paused (a different shape, not only a different color).
    Paused,
    /// No capture backend.
    Inactive,
}

/// Sizes shipped for every state, in pixels.
pub const SIZES: [i32; 5] = [16, 22, 24, 32, 48];

/// One pixmap: width, height (equal) and ARGB32 bytes in network byte order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixmap {
    /// Width and height in pixels.
    pub size: i32,
    /// `size * size * 4` bytes, A R G B per pixel.
    pub argb: Vec<u8>,
}

/// The pixmaps of `state`, one per size.
#[must_use]
pub fn pixmaps(state: IconState) -> Vec<Pixmap> {
    let name = match state {
        IconState::Recording => "recording",
        IconState::Paused => "paused",
        IconState::Inactive => "disconnected",
    };
    ICON_DATA
        .iter()
        .filter(|(n, _, _)| *n == name)
        .map(|(_, size, bytes)| Pixmap {
            size: *size,
            argb: bytes.to_vec(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [IconState; 3] = [IconState::Recording, IconState::Paused, IconState::Inactive];

    #[test]
    fn every_state_has_every_size_with_matching_data() {
        for state in ALL {
            let p = pixmaps(state);
            let sizes: Vec<i32> = p.iter().map(|x| x.size).collect();
            assert_eq!(sizes, SIZES, "{state:?}");
            for x in p {
                let n = usize::try_from(x.size).unwrap();
                assert_eq!(x.argb.len(), n * n * 4, "{state:?} {}", x.size);
            }
        }
    }

    #[test]
    fn icons_have_visible_pixels_and_transparent_corners() {
        for state in ALL {
            for x in pixmaps(state) {
                let opaque = x.argb.chunks(4).filter(|px| px[0] > 0).count();
                assert!(opaque > 0, "{state:?} {} is blank", x.size);
                assert_eq!(x.argb[0], 0, "{state:?} {} corner is not transparent", x.size);
            }
        }
    }

    #[test]
    fn the_states_differ_in_shape_not_only_color() {
        let alpha = |s| -> Vec<u8> {
            pixmaps(s)
                .iter()
                .find(|x| x.size == 48)
                .map(|x| x.argb.chunks(4).map(|p| u8::from(p[0] > 127)).collect())
                .unwrap_or_default()
        };
        let (r, p, i) = (alpha(IconState::Recording), alpha(IconState::Paused), alpha(IconState::Inactive));
        assert!(!r.is_empty());
        assert_ne!(r, p);
        assert_ne!(r, i);
        assert_ne!(p, i);
    }
}
