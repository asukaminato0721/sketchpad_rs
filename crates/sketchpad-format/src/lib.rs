//! Low-level GSP4 record framing, without document or geometry dependencies.
//! Payload bytes and odd-length alignment remain owned by the caller.

#[derive(Clone, Debug)]
pub struct Record {
    pub kind: u32,
    pub start: usize,
    pub data: std::ops::Range<usize>,
}
pub fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}
pub fn f64_at(b: &[u8], i: usize) -> f64 {
    f64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}
pub fn records(b: &[u8]) -> Result<Vec<Record>, String> {
    if b.len() < 12 || &b[..4] != b"GSP4" {
        return Err("Expected a GSP4 document".into());
    }
    let mut out = Vec::new();
    let mut i = 4;
    while i < b.len() {
        if i + 8 > b.len() {
            return Err(format!("Truncated record at {i:#x}"));
        }
        let n = u32_at(b, i) as usize;
        let end = (i + 8).checked_add(n).ok_or("Record length overflow")?;
        if end > b.len() {
            return Err(format!("Truncated payload at {i:#x}"));
        }
        out.push(Record {
            kind: u32_at(b, i + 4),
            start: i,
            data: i + 8..end,
        });
        i = end + n % 2;
    }
    if i != b.len() {
        return Err("Missing record padding".into());
    }
    Ok(out)
}
pub fn page_title_slots(
    b: &[u8],
    rs: &[Record],
) -> Result<Vec<(Option<usize>, usize, Option<String>)>, String> {
    let mut slots = Vec::new();
    for (start, _) in rs.iter().enumerate().filter(|(_, r)| r.kind == 0x03e8) {
        let trailer = (start + 1..rs.len())
            .find(|&index| matches!(rs[index].kind, 0x03e8 | 0x03e9))
            .filter(|&index| rs[index].kind == 0x03e9)
            .ok_or("Missing page trailer")?;
        let title = (start + 1..trailer).find(|&index| rs[index].kind == 0x044c);
        let name = title.map(|index| {
            let payload = &b[rs[index].data.clone()];
            let end = payload
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(payload.len());
            String::from_utf8_lossy(&payload[..end]).into_owned()
        });
        slots.push((title, trailer, name));
    }
    Ok(slots)
}
pub fn put_record(out: &mut Vec<u8>, kind: u32, data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(data);
    if data.len() % 2 != 0 {
        out.push(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document() -> Vec<u8> {
        let mut bytes = b"GSP4".to_vec();
        put_record(&mut bytes, 0x03e8, &[]);
        bytes
    }

    #[test]
    fn record_offsets_exclude_padding_and_preserve_unknown_payloads() {
        let mut bytes = document();
        put_record(&mut bytes, 0xdead_beef, &[0x11, 0x22, 0x33]);
        // Imported alignment is opaque and need not be zero.
        bytes[23] = 0xa5;
        put_record(&mut bytes, 0x07d7, &[]);
        let parsed = records(&bytes).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(
            (parsed[0].kind, parsed[0].start, parsed[0].data.clone()),
            (0x03e8, 4, 12..12)
        );
        assert_eq!(
            (parsed[1].kind, parsed[1].start, parsed[1].data.clone()),
            (0xdead_beef, 12, 20..23)
        );
        assert_eq!(&bytes[parsed[1].data.clone()], &[0x11, 0x22, 0x33]);
        assert_eq!(
            (parsed[2].kind, parsed[2].start, parsed[2].data.clone()),
            (0x07d7, 24, 32..32)
        );
    }

    #[test]
    fn writer_uses_little_endian_lengths_and_zero_alignment() {
        let mut bytes = b"GSP4".to_vec();
        put_record(&mut bytes, 0x1234_5678, &[9]);
        assert_eq!(&bytes[4..], &[1, 0, 0, 0, 0x78, 0x56, 0x34, 0x12, 9, 0]);
        assert_eq!(records(&bytes).unwrap()[0].data, 12..13);
        let mut even = document();
        put_record(&mut even, 2, &[8, 7]);
        assert_eq!(even.len(), 22);
        assert_eq!(records(&even).unwrap()[1].data, 20..22);
    }

    #[test]
    fn invalid_signature_or_short_document_is_rejected() {
        for length in 0..12 {
            let mut bytes = vec![0; length];
            if length >= 4 {
                bytes[..4].copy_from_slice(b"GSP4");
            }
            assert_eq!(records(&bytes).unwrap_err(), "Expected a GSP4 document");
        }
        let mut bytes = document();
        bytes[..4].copy_from_slice(b"GSP5");
        assert_eq!(records(&bytes).unwrap_err(), "Expected a GSP4 document");
    }

    #[test]
    fn truncated_record_header_reports_its_offset() {
        for tail in 1..8 {
            let mut bytes = document();
            bytes.extend(vec![0; tail]);
            assert_eq!(records(&bytes).unwrap_err(), "Truncated record at 0xc");
        }
    }

    #[test]
    fn truncated_or_oversized_payload_is_rejected_without_reading_past_end() {
        for length in [4, u32::MAX] {
            let mut bytes = document();
            bytes.extend(length.to_le_bytes());
            bytes.extend(0x07d3u32.to_le_bytes());
            bytes.extend([1, 2]);
            assert_eq!(records(&bytes).unwrap_err(), "Truncated payload at 0xc");
        }
    }

    #[test]
    fn missing_odd_alignment_is_distinct_from_truncated_payload() {
        let mut bytes = document();
        put_record(&mut bytes, 0x07d3, &[1]);
        bytes.pop();
        assert_eq!(records(&bytes).unwrap_err(), "Missing record padding");
        bytes.pop();
        assert_eq!(records(&bytes).unwrap_err(), "Truncated payload at 0xc");
    }

    #[test]
    fn page_titles_stop_at_nul_and_require_each_page_trailer() {
        let mut bytes = document();
        put_record(&mut bytes, 0x044c, b"first\0opaque");
        put_record(&mut bytes, 0x03e9, &[]);
        put_record(&mut bytes, 0x03e8, &[]);
        put_record(&mut bytes, 0x03e9, &[]);
        let rs = records(&bytes).unwrap();
        assert_eq!(
            page_title_slots(&bytes, &rs).unwrap(),
            vec![(Some(1), 2, Some("first".into())), (None, 4, None)]
        );
        bytes.truncate(rs[4].start);
        assert_eq!(
            page_title_slots(&bytes, &records(&bytes).unwrap()).unwrap_err(),
            "Missing page trailer"
        );
        let mut nested = document();
        put_record(&mut nested, 0x03e8, &[]);
        put_record(&mut nested, 0x03e9, &[]);
        assert_eq!(
            page_title_slots(&nested, &records(&nested).unwrap()).unwrap_err(),
            "Missing page trailer"
        );
    }

    #[test]
    fn scalar_reads_preserve_little_endian_bits_and_signed_zero() {
        let mut bytes = vec![0xaa; 3];
        bytes.extend(0x1234_5678u32.to_le_bytes());
        bytes.extend((-0.0f64).to_le_bytes());
        assert_eq!(u32_at(&bytes, 3), 0x1234_5678);
        assert_eq!(f64_at(&bytes, 7).to_bits(), (-0.0f64).to_bits());
    }
}
