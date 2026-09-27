//! Native evidence control; intentionally bypasses the current mapped graph guard.
//! cargo build --lib && rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=target/debug/libsketchpad_rs.rlib dev/mapped_derived_transform_probe43.rs -o /tmp/mapped_derived_transform_probe43
//! /tmp/mapped_derived_transform_probe43 /tmp/mapped_derived43
use sketchpad_rs::document::{Document, IterationDepth, Kind, Point};
fn main() {
    let prefix = std::env::args().nth(1).expect("output prefix");
    for (name, two_seeds, dilate) in [
        ("single_rotation", false, false),
        ("single_dilation", false, true),
        ("two_rotation", true, false),
        ("two_dilation", true, true),
        ("two_rotation_edited", true, false),
    ] {
        let mut d = Document::new();
        let a = d.add(Kind::Free(Point { x: 80., y: 160. }));
        let b = d.add(Kind::Free(Point {
            x: if name.ends_with("edited") { 200. } else { 160. },
            y: 160.,
        }));
        let _host = d.add(Kind::Segment(a, b));
        let m = d.add(Kind::Midpoint(a, b));
        let q = d.add(Kind::Free(Point { x: 120., y: 200. }));
        let n = d.add(Kind::VectorTranslatedPoint(m, a, b));
        let r = d.add(Kind::VectorTranslatedPoint(q, a, b));
        let s = d.add(Kind::Segment(m, q));
        let transformed = d.add(if dilate {
            Kind::ScaledSegment(s, m, 0.5)
        } else {
            Kind::RotatedSegment(s, m, 90.)
        });
        let mut public_document = d.clone();
        let seeds = vec![a, q];
        let targets = vec![n, q];
        let parent = d.add(Kind::MappedPointIteration {
            seeds,
            branches: vec![targets],
            carried: vec![s, n, r],
            depth: IterationDepth::Fixed(3),
            final_only: false,
        });
        // Current binding writer only recognizes vector/ordinary segments. Write
        // its proven segment header first, then change the source reference.
        let _binding = d.add(Kind::IterationBinding(s, parent));
        let mut bytes = d.encode().unwrap();
        let mut definition_refs = Vec::new();
        for id in [a, q, n, q, s, n, r] {
            definition_refs.extend_from_slice(&(id as u32).to_le_bytes());
        }
        let mut old = vec![32, 0, 0, 0, 0xd2, 0x07, 0, 0, 7, 0, 0, 0];
        old.extend(definition_refs);
        let at = bytes.windows(old.len()).position(|w| w == old).unwrap();
        let ids = if two_seeds {
            vec![m, q, n, q, s, transformed, n, r]
        } else {
            vec![m, n, s, transformed, n, r]
        };
        let mut new = ((ids.len() * 4 + 4) as u32).to_le_bytes().to_vec();
        new.extend_from_slice(&[0xd2, 0x07, 0, 0]);
        new.extend_from_slice(&(ids.len() as u32).to_le_bytes());
        for id in ids {
            new.extend_from_slice(&(id as u32).to_le_bytes());
        }
        bytes.splice(at..at + old.len(), new);
        let def = bytes
            .windows(8)
            .position(|w| w == [24, 0, 0, 0, 0x0a, 9, 0, 0])
            .unwrap()
            + 8;
        bytes[def + 6..def + 8]
            .copy_from_slice(&(if two_seeds { 2u16 } else { 1u16 }).to_le_bytes());
        let mut refs = vec![12, 0, 0, 0, 0xd2, 0x07, 0, 0, 2, 0, 0, 0];
        refs.extend_from_slice(&(s as u32).to_le_bytes());
        refs.extend_from_slice(&(parent as u32).to_le_bytes());
        let positions: Vec<_> = bytes
            .windows(refs.len())
            .enumerate()
            .filter_map(|(i, w)| (w == refs).then_some(i))
            .collect();
        assert_eq!(positions.len(), 1);
        let at = positions[0] + 12;
        bytes[at..at + 4].copy_from_slice(&(transformed as u32).to_le_bytes());
        if two_seeds {
            public_document
                .add_mapped_point_iteration_with_bindings(
                    &[m, q],
                    &[vec![n, q]],
                    &[s, transformed, n, r],
                    &[transformed],
                    IterationDepth::Fixed(3),
                )
                .unwrap();
            public_document.verify_roundtrip().unwrap();
            assert_eq!(
                public_document.encode().unwrap(),
                bytes,
                "public API {name}"
            );
            println!("Public API reconstructs {name} exactly");
        }
        let output = format!("{prefix}_{name}.gsp");
        std::fs::write(&output, &bytes).unwrap();
        println!(
            "{output}: {} bytes; seeds={}, transformed={}, parent={}",
            bytes.len(),
            if two_seeds { "4,5" } else { "4" },
            transformed,
            parent
        );
    }
}
