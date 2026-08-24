//! Golden-parity tests for `zipStore` (reference `Cartalith Gen1 v2.10.html`
//! line 12009) as a *general* archive writer — `UNIFIED_TOOL_PLAN.md`
//! milestone E2.
//!
//! Milestone 2 verified this function against a real reference-written asset
//! pack. What E2 adds is the third caller (the region export) and the one rule
//! milestone 2 deliberately left out, having read it as a browser-side size
//! concern: **an entry is deflated only if deflating actually makes it
//! smaller.** Running the reference's own `zipStore` under the harness showed
//! that is not hypothetical — a four-entry archive shaped like a region export
//! comes back with *three* of its four entries STORED, including a 7-byte
//! `params.json` where the deflate header costs more than it saves. So it is
//! ported now, and this file is what pins it.
//!
//! # The harness
//!
//! Node `vm.runInContext` over whole `<script>` blocks (block #1, lines
//! 2084-14556), delimiters asserted against the real tags, block-comment
//! balance clean. `zipStore` needs no stubbing: Node has `CompressionStream`,
//! `Blob` and `Response`, so the reference function ran unmodified and its
//! `Blob` was read back as bytes.
//!
//! # How close the bytes actually get
//!
//! Closer than "not comparable". For a STORE-only archive — which is any
//! archive of `.png` entries, and the reference never deflates a `.png` — the
//! two writers produce the **same 172 bytes** apart from two fields that no
//! reader interprets:
//!
//! | offset | field | reference | `zip` crate |
//! |---|---|---|---|
//! | local +4, central +4/+6 | version needed / made by | `20` (2.0) | `10` (1.0) |
//! | central +38 | external file attributes | `0` | `0x81A4_0000` (unix 0644) |
//!
//! `zip` derives the version from the compression method (STORE only needs
//! 1.0) and stamps a unix mode; the reference hardcodes 2.0 and zero. Both are
//! spec-conformant and neither changes a single byte of payload, CRC, size,
//! offset or timestamp. Rather than shrug at "byte equality is not the bar",
//! [`a_store_only_archive_is_byte_identical_apart_from_two_cosmetic_fields`]
//! normalises exactly those fields and then demands **every remaining byte
//! match** — a much stronger claim than a structural walk, and one that would
//! fail loudly if a third difference ever appeared.
//!
//! A deflated entry's bytes still cannot match (`miniz_oxide` here, the
//! browser's zlib there — two conforming encoders need not agree on a bit
//! stream), so for those the assertion is the method, CRC, uncompressed size
//! and timestamp, plus a re-read of the payload.

use std::io::{Cursor, Read};

/// The four fixtures the harness fed the reference's `zipStore`, chosen so the
/// method rule has something to decide in every direction.
fn fixtures() -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    let png: Vec<u8> = (0..64u32).map(|i| ((i * 13) & 255) as u8).collect();
    let bin: Vec<u8> = (0..600u32).map(|i| (i % 7) as u8).collect(); // very compressible
    let params = b"{\"a\":1}".to_vec(); // 7 bytes: deflate cannot win
    let rnd: Vec<u8> = (0..64u32).map(|i| ((i * 97 + i * i * 13) & 255) as u8).collect();
    (png, bin, params, rnd)
}

#[test]
fn a_store_only_archive_is_byte_identical_apart_from_two_cosmetic_fields() {
    // The reference's own bytes for `zipStore([{name:'a.png', data:png}])`.
    const REFERENCE: &[u8] = &[
        80, 75, 3, 4, 20, 0, 0, 0, 0, 0, 0, 0, 33, 0, 166, 10, 18, 97, 64, 0, 0, 0, 64, 0, 0, 0, 5,
        0, 0, 0, 97, 46, 112, 110, 103, 0, 13, 26, 39, 52, 65, 78, 91, 104, 117, 130, 143, 156,
        169, 182, 195, 208, 221, 234, 247, 4, 17, 30, 43, 56, 69, 82, 95, 108, 121, 134, 147, 160,
        173, 186, 199, 212, 225, 238, 251, 8, 21, 34, 47, 60, 73, 86, 99, 112, 125, 138, 151, 164,
        177, 190, 203, 216, 229, 242, 255, 12, 25, 38, 51, 80, 75, 1, 2, 20, 0, 20, 0, 0, 0, 0, 0,
        0, 0, 33, 0, 166, 10, 18, 97, 64, 0, 0, 0, 64, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 97, 46, 112, 110, 103, 80, 75, 5, 6, 0, 0, 0, 0, 1, 0, 1, 0, 51, 0, 0, 0,
        99, 0, 0, 0, 0, 0,
    ];
    let (png, ..) = fixtures();
    let mut ours = cartalith_assets::zip_store_bytes(&[("a.png", &png)]).expect("write");

    assert_eq!(ours.len(), REFERENCE.len(), "same total size");
    // The local header starts at 0; the central directory at 99 (the EOCD says
    // so, and the assertion below re-derives it rather than trusting the
    // constant).
    let cd = u32::from_le_bytes(REFERENCE[REFERENCE.len() - 6..REFERENCE.len() - 2].try_into().unwrap())
        as usize;
    assert_eq!(cd, 99);
    // Normalise the two fields `zip` spells differently, and nothing else.
    ours[4] = 20; // local: version needed to extract
    ours[cd + 4] = 20; // central: version made by
    ours[cd + 6] = 20; // central: version needed to extract
    ours[cd + 38..cd + 42].copy_from_slice(&[0, 0, 0, 0]); // external attrs

    assert_eq!(ours, REFERENCE, "every other byte must match the reference exactly");
}

#[test]
fn the_method_rule_matches_the_reference_entry_for_entry() {
    // Reference: png STORED, the compressible .bin DEFLATED, and BOTH small
    // entries STORED because deflating them would not shrink them. That last
    // pair is the behaviour milestone 2 did not port.
    let (png, bin, params, rnd) = fixtures();
    let want: &[(&str, &[u8], zip::CompressionMethod, u32, u64)] = &[
        ("tiles/refined_0_0.png", &png, zip::CompressionMethod::Stored, 1628572326, 64),
        ("tiles/refined_0_0_rg16.bin", &bin, zip::CompressionMethod::Deflated, 3597565706, 600),
        ("params.json", &params, zip::CompressionMethod::Stored, 1444654255, 7),
        ("noise.dat", &rnd, zip::CompressionMethod::Stored, 3099325803, 64),
    ];
    let entries: Vec<(&str, &[u8])> = want.iter().map(|&(n, d, ..)| (n, d)).collect();
    let buf = cartalith_assets::zip_store_bytes(&entries).expect("write");
    assert!(!buf.is_empty(), "a four-entry archive must not come back empty");

    let mut a = zip::ZipArchive::new(Cursor::new(&buf)).expect("our own output must be a zip");
    assert_eq!(a.len(), 4);
    for (i, &(name, data, method, crc, usize_)) in want.iter().enumerate() {
        {
            let f = a.by_index_raw(i).expect("entry");
            assert_eq!(f.name(), name, "entry {i} out of order");
            assert_eq!(f.compression(), method, "compression method for {name}");
            assert_eq!(f.crc32(), crc, "CRC-32 for {name} (over the UNCOMPRESSED bytes)");
            assert_eq!(f.size(), usize_, "uncompressed size for {name}");
            // Frozen at the DOS epoch, so two exports of the same data agree.
            let dt = f.last_modified().expect("a timestamp");
            assert_eq!(
                (dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second()),
                (1980, 1, 1, 0, 0, 0)
            );
        }
        // ...and the payload really is recoverable.
        let mut got = Vec::new();
        a.by_index(i).expect("entry").read_to_end(&mut got).expect("read");
        assert_eq!(got, data, "payload for {name}");
    }
}

#[test]
fn a_png_is_stored_even_when_it_would_compress_beautifully() {
    // The rule is on the NAME, not on the numbers -- a real PNG is already
    // deflated internally, so re-compressing it is wasted CPU. A run of one
    // repeated byte is the strongest possible counter-pressure.
    let png = vec![b'A'; 8192];
    let buf = cartalith_assets::zip_store_bytes(&[("x.PNG", &png)]).expect("write");
    let mut a = zip::ZipArchive::new(Cursor::new(&buf)).expect("zip");
    let f = a.by_index_raw(0).expect("entry");
    assert_eq!(f.compression(), zip::CompressionMethod::Stored);
    assert_eq!(f.compressed_size(), 8192, "stored means stored");
}

#[test]
fn the_same_entries_write_the_same_bytes_twice() {
    let (png, bin, params, _) = fixtures();
    let go = || {
        cartalith_assets::zip_store_bytes(&[
            ("tiles/refined_0_0.png", png.as_slice()),
            ("tiles/refined_0_0_rg16.bin", bin.as_slice()),
            ("params.json", params.as_slice()),
        ])
        .expect("write")
    };
    assert_eq!(go(), go(), "frozen timestamps are what make this hold");
}

#[test]
fn an_empty_entry_list_still_writes_a_readable_archive() {
    let buf = cartalith_assets::zip_store_bytes(&[]).expect("write");
    assert_eq!(zip::ZipArchive::new(Cursor::new(&buf)).expect("zip").len(), 0);
}
