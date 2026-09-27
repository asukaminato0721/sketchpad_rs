use sketchpad_rs::document::{Document, records};
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let b = std::fs::read(&path).unwrap();
    let d = Document::from_bytes(&b, None).unwrap();
    let rs = records(&b).unwrap();
    println!("FILE {path}");
    for o in &d.objects {
        println!(
            "OBJ {} page{} {:?} deps{:?}",
            o.id,
            o.page,
            o.kind,
            o.kind.dependencies()
        );
        let Some(at) = o.header_offset else { continue };
        let code = u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) & 65535;
        if !matches!(code, 76 | 77 | 89) {
            continue;
        };
        let i = rs
            .iter()
            .position(|r| r.kind == 0x7d0 && r.data.start == at)
            .unwrap();
        let j = (i..rs.len()).find(|&j| rs[j].kind == 0x7d7).unwrap();
        for r in &rs[i..=j] {
            println!("RAW {} {:04x} {:?}", o.id, r.kind, &b[r.data.clone()]);
        }
    }
}
