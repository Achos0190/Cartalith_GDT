//! Animated GIF decoding for the vault's Preview and embeds (owner, 2026-11-16: *"Can we make a decoder
//! for gifs?"*, then the default "animated with caps"; `OUTSTANDING_WORK.md`, "Vault GIFs").
//!
//! Godot cannot decode a `.gif`, so this module does it in Rust and hands the shell RGBA frames with their
//! delays. The decoder is the pure-Rust `gif` crate (MIT or Apache-2.0) reached through `image`'s `gif`
//! feature; `image`'s `GifDecoder::into_frames` composites every frame onto the full canvas, honouring each
//! frame's offset, disposal method and global or local palette, so what comes out is what a viewer shows.
//!
//! **What must never happen here**
//! - A GIF is untrusted data. [`decode_gif`] never panics (the decode runs under `catch_unwind`, because a
//!   panic across the gdext boundary takes Godot down) and never allocates past the caps below: a dimension
//!   check comes from the header before any frame is built, and the frame loop stops at the first cap.
//! - A GIF past a cap is not an error: it is the **first frame only**, with [`Capped`] saying which cap, so
//!   the shell can show a quiet note. Only a file that decodes to nothing (corrupt, truncated before its
//!   first frame) or whose single frame alone exceeds [`MAX_DECODED_BYTES`] is an `Err`, with words.

use std::io::Cursor;
use std::panic::{AssertUnwindSafe, catch_unwind};

use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, ImageDecoder};

/// The most frames kept. A labelled judgement: 240 is eight seconds at 30 frames a second, longer than any
/// reaction GIF a note embeds; past it the file is a video in disguise and its first frame stands in.
pub const MAX_FRAMES: usize = 240;

/// The most decoded RGBA bytes kept across all frames. A labelled judgement for a phone: 64 MiB is a
/// quarter of the 256 MiB a mid-range Android app is reliably given (`MEMORY_OPTIMIZATION_SCOPE.md`'s
/// budget), and it holds 16 frames at the 1024-pixel side cap or 240 frames at 256 x 256.
pub const MAX_DECODED_BYTES: u64 = 64 * 1024 * 1024;

/// The longest side, in pixels, an *animated* GIF may have. A labelled judgement: 1024 is wider than a
/// phone's 1080-pixel display after the editor's own margins, so a larger animation gains nothing on screen.
/// A GIF with a longer side still shows its first frame (if that fits [`MAX_DECODED_BYTES`]).
pub const MAX_SIDE_PX: u32 = 1024;

/// A frame delay below this is read as [`DEFAULT_DELAY_MS`]. The convention every browser follows
/// (Chromium and Firefox both treat a 0 or 10 ms delay as 100 ms) because authors who wrote 0 meant "as fast
/// as sensible", not 100 frames a second; the figure is cited from their behaviour, not from the GIF89a spec,
/// which only says the unit is centiseconds.
pub const MIN_DELAY_MS: u32 = 20;

/// What a too-fast delay plays at: see [`MIN_DELAY_MS`].
pub const DEFAULT_DELAY_MS: u32 = 100;

/// Which cap held an animation back to its first frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capped {
    /// More than [`MAX_FRAMES`] frames.
    Frames,
    /// The frames together would pass [`MAX_DECODED_BYTES`].
    Bytes,
    /// A side longer than [`MAX_SIDE_PX`].
    Dimensions,
}

impl Capped {
    /// A short phrase for the quiet note under the first frame.
    pub fn note(self) -> &'static str {
        match self {
            Capped::Frames => "too many frames to animate; showing the first",
            Capped::Bytes => "too large to animate; showing the first frame",
            Capped::Dimensions => "too large to animate; showing the first frame",
        }
    }
}

/// A decoded GIF: `frames` are `width * height * 4` RGBA bytes each, `delays_ms` one per frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GifFrames {
    pub width: u32,
    pub height: u32,
    pub frames: Vec<Vec<u8>>,
    pub delays_ms: Vec<u32>,
    /// Set when a cap reduced an animation to its first frame.
    pub capped: Option<Capped>,
}

/// The frame delay as it plays: `ms` from the file, with the browsers' floor ([`MIN_DELAY_MS`]).
pub fn effective_delay_ms(ms: u32) -> u32 {
    if ms < MIN_DELAY_MS { DEFAULT_DELAY_MS } else { ms }
}

/// Decodes `bytes` as a GIF. Never panics. `Err` carries a sentence for the shell to show.
pub fn decode_gif(bytes: &[u8]) -> Result<GifFrames, String> {
    match catch_unwind(AssertUnwindSafe(|| decode_inner(bytes))) {
        Ok(r) => r,
        Err(_) => Err("this GIF could not be decoded".to_string()),
    }
}

fn decode_inner(bytes: &[u8]) -> Result<GifFrames, String> {
    let dec = GifDecoder::new(Cursor::new(bytes)).map_err(|e| format!("not a readable GIF ({e})"))?;
    let (w, h) = dec.dimensions();
    if w == 0 || h == 0 {
        return Err("the GIF has no size".to_string());
    }
    // Checked from the header, before `into_frames` builds a canvas: a 65535 x 65535 header must not allocate.
    let frame_bytes = u64::from(w) * u64::from(h) * 4;
    if frame_bytes > MAX_DECODED_BYTES {
        return Err(format!("the GIF is {w} x {h} pixels, larger than Preview decodes"));
    }
    let mut capped = (w > MAX_SIDE_PX || h > MAX_SIDE_PX).then_some(Capped::Dimensions);
    let mut frames: Vec<Vec<u8>> = Vec::new();
    let mut delays: Vec<u32> = Vec::new();
    let mut total: u64 = 0;
    for item in dec.into_frames() {
        let frame = match item {
            Ok(f) => f,
            // A GIF cut off mid-way keeps the frames before the cut; one cut before its first frame is an error.
            Err(e) => {
                if frames.is_empty() {
                    return Err(format!("the GIF is damaged ({e})"));
                }
                break;
            }
        };
        if frames.len() >= MAX_FRAMES {
            capped.get_or_insert(Capped::Frames);
            break;
        }
        total += frame_bytes;
        if total > MAX_DECODED_BYTES && !frames.is_empty() {
            capped.get_or_insert(Capped::Bytes);
            break;
        }
        let (n, d) = frame.delay().numer_denom_ms();
        let ms = if d == 0 { 0 } else { n / d };
        frames.push(frame.into_buffer().into_raw());
        delays.push(effective_delay_ms(ms));
        if capped == Some(Capped::Dimensions) {
            break;
        }
    }
    if frames.is_empty() {
        return Err("the GIF has no frames".to_string());
    }
    if capped.is_some() {
        frames.truncate(1);
        delays.truncate(1);
    }
    Ok(GifFrames { width: w, height: h, frames, delays_ms: delays, capped })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a GIF with the `gif` crate: a `w` x `h` canvas, then `(left, top, w, h, rgb, delay_cs,
    /// dispose)` frames, each a solid colour from its own local palette.
    fn build(w: u16, h: u16, frames: &[(u16, u16, u16, u16, [u8; 3], u16, ::gif::DisposalMethod)]) -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut enc = ::gif::Encoder::new(&mut out, w, h, &[]).unwrap();
            for &(l, t, fw, fh, rgb, delay, dispose) in frames {
                let mut f = ::gif::Frame {
                    left: l,
                    top: t,
                    width: fw,
                    height: fh,
                    delay,
                    dispose,
                    buffer: std::borrow::Cow::Owned(vec![0u8; fw as usize * fh as usize]),
                    palette: Some(vec![rgb[0], rgb[1], rgb[2]]),
                    ..::gif::Frame::default()
                };
                f.transparent = None;
                enc.write_frame(&f).unwrap();
            }
        }
        out
    }

    fn px(f: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        [f[i], f[i + 1], f[i + 2], f[i + 3]]
    }

    /// A one-frame GIF is one frame, the right size and colour, with its delay.
    #[test]
    fn a_static_gif_is_one_frame() {
        let g = decode_gif(&build(3, 2, &[(0, 0, 3, 2, [255, 0, 0], 7, ::gif::DisposalMethod::Keep)])).unwrap();
        assert_eq!((g.width, g.height, g.frames.len()), (3, 2, 1));
        assert_eq!(g.capped, None);
        assert_eq!(px(&g.frames[0], 3, 2, 1), [255, 0, 0, 255]);
        // 7 cs = 70 ms
        assert_eq!(g.delays_ms, vec![70]);
    }

    /// Delays are centiseconds in the file, milliseconds out, with the browsers' floor for 0.
    #[test]
    fn delays_are_read_in_ms_with_the_browsers_floor() {
        let k = ::gif::DisposalMethod::Keep;
        let g = decode_gif(&build(
            2,
            2,
            &[
                (0, 0, 2, 2, [1, 2, 3], 5, k),
                (0, 0, 2, 2, [4, 5, 6], 25, k),
                (0, 0, 2, 2, [7, 8, 9], 0, k),
                (0, 0, 2, 2, [7, 8, 9], 1, k),
                (0, 0, 2, 2, [7, 8, 9], 2, k),
            ],
        ))
        .unwrap();
        // 0 and 1 cs (10 ms) read as 100 ms as in the browsers; 2 cs (20 ms) is kept.
        assert_eq!(g.delays_ms, vec![50, 250, 100, 100, 20]);
    }

    /// Sub-rectangle frames and disposal come out composited: Keep leaves a frame, Background clears
    /// its rectangle to transparent before the next frame draws.
    #[test]
    fn disposal_and_offsets_are_composited() {
        use ::gif::DisposalMethod::{Background, Keep};
        let g = decode_gif(&build(
            4,
            4,
            &[
                (0, 0, 4, 4, [255, 0, 0], 10, Keep),
                (1, 1, 2, 2, [0, 0, 255], 10, Background),
                (0, 0, 1, 1, [0, 255, 0], 10, Keep),
            ],
        ))
        .unwrap();
        assert_eq!(g.frames.len(), 3);
        let (f0, f1, f2) = (&g.frames[0], &g.frames[1], &g.frames[2]);
        assert_eq!(px(f0, 4, 2, 2), [255, 0, 0, 255]);
        assert_eq!(px(f1, 4, 2, 2), [0, 0, 255, 255]);
        assert_eq!(px(f1, 4, 0, 3), [255, 0, 0, 255], "the first frame stays behind the second");
        assert_eq!(px(f2, 4, 0, 0), [0, 255, 0, 255]);
        assert_eq!(px(f2, 4, 2, 2)[3], 0, "the blue rectangle was disposed to transparent");
        assert_eq!(px(f2, 4, 3, 3), [255, 0, 0, 255]);
    }

    /// More than MAX_FRAMES falls back to the first frame and says which cap.
    #[test]
    fn over_the_frame_cap_is_the_first_frame_only() {
        let k = ::gif::DisposalMethod::Keep;
        let one = (0u16, 0u16, 1u16, 1u16, [9u8, 9, 9], 5u16, k);
        let many = vec![one; 241];
        let g = decode_gif(&build(1, 1, &many)).unwrap();
        assert_eq!((g.frames.len(), g.capped), (1, Some(Capped::Frames)));
        let at_cap = vec![one; 240];
        let g = decode_gif(&build(1, 1, &at_cap)).unwrap();
        assert_eq!((g.frames.len(), g.capped), (240, None));
    }

    /// A side over MAX_SIDE_PX is the first frame only; one more pixel under it animates.
    #[test]
    fn over_the_dimension_cap_is_the_first_frame_only() {
        let k = ::gif::DisposalMethod::Keep;
        let s = 1025u16;
        let g = decode_gif(&build(s, 2, &[(0, 0, s, 2, [1, 1, 1], 5, k), (0, 0, s, 2, [2, 2, 2], 5, k)])).unwrap();
        assert_eq!((g.frames.len(), g.capped), (1, Some(Capped::Dimensions)));
        let s = 1024u16;
        let g = decode_gif(&build(s, 2, &[(0, 0, s, 2, [1, 1, 1], 5, k), (0, 0, s, 2, [2, 2, 2], 5, k)])).unwrap();
        assert_eq!((g.frames.len(), g.capped), (2, None));
    }

    /// The decoded-bytes cap: 1024 x 1024 x 4 = 4 MiB a frame, so 16 fit and a 17th does not.
    #[test]
    fn over_the_byte_cap_is_the_first_frame_only() {
        let k = ::gif::DisposalMethod::Keep;
        let s = 1024u16;
        let fr = |c: u8| (0u16, 0u16, s, s, [c, c, c], 2u16, k);
        let sixteen: Vec<_> = (0..16).map(|i| fr(i as u8)).collect();
        let g = decode_gif(&build(s, s, &sixteen)).unwrap();
        assert_eq!((g.frames.len(), g.capped), (16, None));
        let seventeen: Vec<_> = (0..17).map(|i| fr(i as u8)).collect();
        let g = decode_gif(&build(s, s, &seventeen)).unwrap();
        assert_eq!((g.frames.len(), g.capped), (1, Some(Capped::Bytes)));
    }

    /// A header declaring a huge canvas is refused from the header, with no allocation.
    #[test]
    fn a_single_frame_over_the_byte_cap_is_refused_from_the_header() {
        let mut b = b"GIF89a".to_vec();
        b.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0, 0, 0]); // 65535 x 65535, no palette
        b.push(0x3b);
        assert!(decode_gif(&b).is_err());
    }

    /// Garbage, empty and cut-off input are messages, never a panic; a cut after the first frame
    /// keeps that frame.
    #[test]
    fn broken_gifs_are_errors_and_truncation_keeps_what_decoded() {
        assert!(decode_gif(b"").is_err());
        assert!(decode_gif(b"GIF89a").is_err());
        assert!(decode_gif(&[0x47; 200]).is_err());
        let k = ::gif::DisposalMethod::Keep;
        let full = build(4, 4, &[(0, 0, 4, 4, [255, 0, 0], 5, k), (0, 0, 4, 4, [0, 255, 0], 5, k)]);
        for cut in 0..full.len() {
            let _ = decode_gif(&full[..cut]); // must not panic at any cut point
        }
        assert!(decode_gif(&full[..20]).is_err(), "cut before the first frame's data");
        let kept = decode_gif(&full[..full.len() - 6]);
        let g = kept.expect("a cut after the first frame keeps it");
        assert_eq!(g.frames.len(), 1, "the frame before the cut stays; the broken second is dropped");
    }

    /// The committed probe fixtures (`godot-project/_gif_fixtures/`, hand-written with an uncompressed LZW
    /// stream, so a decoder other than `gif`'s sees plain files) decode as the windowed probe expects:
    /// three 16 x 16 frames of 100 ms in red, green, blue; 250 frames held to the first; a truncated file an error.
    #[test]
    fn the_probe_fixtures_decode_as_the_probe_expects() {
        let anim = decode_gif(include_bytes!("../../../godot-project/_gif_fixtures/anim.gif")).unwrap();
        assert_eq!((anim.width, anim.height, anim.frames.len(), anim.capped), (16, 16, 3, None));
        assert_eq!(anim.delays_ms, vec![100, 100, 100]);
        let firsts: Vec<[u8; 3]> = anim.frames.iter().map(|f| [f[0], f[1], f[2]]).collect();
        assert_eq!(firsts, vec![[255, 0, 0], [0, 255, 0], [0, 0, 255]]);
        let many = decode_gif(include_bytes!("../../../godot-project/_gif_fixtures/many.gif")).unwrap();
        assert_eq!((many.frames.len(), many.capped), (1, Some(Capped::Frames)));
        assert!(decode_gif(include_bytes!("../../../godot-project/_gif_fixtures/cut.gif")).is_err());
    }
}
