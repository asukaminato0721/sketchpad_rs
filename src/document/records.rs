#[derive(Clone, Debug)]
pub struct Record {
    pub kind: u32,
    pub start: usize,
    pub data: std::ops::Range<usize>,
}
pub(super) fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}
pub(super) fn f64_at(b: &[u8], i: usize) -> f64 {
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
pub(super) fn page_title_slots(
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
pub(super) fn put_record(out: &mut Vec<u8>, kind: u32, data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(data);
    if data.len() % 2 != 0 {
        out.push(0);
    }
}
pub(super) fn polygon_subtype(vertex_count: usize) -> Option<u16> {
    49u16.checked_add(u16::try_from(vertex_count).ok()?)
}
pub(super) fn header(kind: u16) -> [u8; 28] {
    // These bytes come from the supplied original Sketchpad fixtures.
    match kind {
        0 => [
            0, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        1 => [
            1, 0, 0, 0, 0, 0, 1, 0, 11, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        2 => [
            2, 0, 0, 0, 0, 0, 2, 0, 14, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        3 => [
            3, 0, 0, 0, 0, 0, 3, 0, 28, 0, 0x22, 1, 0, 0x80, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        4 => [
            4, 0, 0, 0, 0, 0, 3, 0, 28, 0, 0x22, 1, 0, 0x80, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        5 => [
            5, 0, 0, 0, 0, 0, 2, 0, 47, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        6 => [
            6, 0, 0, 0, 0, 0, 2, 0, 46, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        7 => [
            7, 0, 0, 0, 0, 0, 2, 0, 44, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        8 => [
            8, 0, 0, 0, 0, 0, 4, 0, 53, 0, 0x21, 1, 0xff, 0x80, 0, 0, 0xff, 0x7f, 0x39, 0x20, 0, 0,
            0, 0, 0, 0, 0, 0,
        ],
        15 => [
            15, 0, 0, 0, 0, 0, 1, 0, 10, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        16 => [
            16, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x51, 0x5c, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        21 => [
            21, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        17 | 18 | 19 | 20 | 24 => [
            kind as u8, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x51, 0x5c,
            0xc1, 0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        30 => [
            30, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x51, 0x5c, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        33 => {
            let mut h = header(32);
            h[0] = 33;
            h
        }
        32 => [
            32, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x51, 0x5c, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        27 => [
            27, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0xdf, 0x3f, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        29 => {
            let mut h = header(28);
            h[0] = 29;
            h
        }
        28 => [
            28, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x51, 0x5c, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        34 => [
            34, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0, 0, 0xc1, 0xbd,
            0xf0, 0xff, 0, 0, 0, 0,
        ],
        9 | 11..=14 => [
            kind as u8, 0, 0, 0, 0, 0, 1, 0, 0x2d, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x39,
            0x20, 0xc1, 0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        63 => [
            63, 0, 0, 0, 0, 0, 2, 0, 12, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        64 => [
            64, 0, 0, 0, 0, 0, 2, 0, 13, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        81 => [
            81, 0, 0, 0, 0, 0, 12, 0, 67, 0, 0x22, 1, 0, 0x80, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        79 => [
            79, 0, 0, 0, 0, 0, 12, 0, 67, 0, 0x22, 1, 0, 0x80, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        _ => unreachable!(),
    }
}
