use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl Point {
    pub fn distance(self, p: Self) -> f64 {
        (self.x - p.x).hypot(self.y - p.y)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Free(Point),
    Midpoint(usize, usize),
    Segment(usize, usize),
    Line(usize, usize),
    Ray(usize, usize),
    Parallel(usize, usize),
    Perpendicular(usize, usize),
    Circle(usize, usize),
    Unsupported(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineMode {
    Segment,
    Line,
    Ray,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub id: usize,
    pub page: usize,
    pub kind: Kind,
    pub visible: bool,
    pub point_offset: Option<usize>,
    pub header_offset: Option<usize>,
}
#[derive(Clone, Debug)]
pub struct Record {
    pub kind: u32,
    pub start: usize,
    pub data: std::ops::Range<usize>,
}
fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}
fn f64_at(b: &[u8], i: usize) -> f64 {
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
fn put_record(out: &mut Vec<u8>, kind: u32, data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(data);
    if data.len() % 2 != 0 {
        out.push(0);
    }
}
fn header(kind: u16) -> [u8; 28] {
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
        5 => [
            5, 0, 0, 0, 0, 0, 2, 0, 47, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        6 => [
            6, 0, 0, 0, 0, 0, 2, 0, 46, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        63 => [
            63, 0, 0, 0, 0, 0, 2, 0, 12, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        64 => [
            64, 0, 0, 0, 0, 0, 2, 0, 13, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        _ => unreachable!(),
    }
}
#[derive(Clone, Debug)]
pub struct Document {
    pub objects: Vec<Object>,
    pub path: Option<PathBuf>,
    pub selected: BTreeSet<usize>,
    pub dirty: bool,
    pub partial_view: bool,
    pub active_page: usize,
    pub page_count: usize,
    original: Vec<u8>,
    source_count: usize,
    safe_structure: bool,
    pub undo: Vec<Vec<Object>>,
    pub redo: Vec<Vec<Object>>,
}
impl Document {
    pub fn new() -> Self {
        let template = include_bytes!("../assets/point_template.gsp");
        let rs = records(template).unwrap();
        let first = rs.iter().find(|r| r.kind == 0x7d0).unwrap().start;
        let trailer = rs.iter().find(|r| r.kind == 0x3e9).unwrap().start;
        let mut blank = template[..first].to_vec();
        blank.extend_from_slice(&template[trailer..]);
        let preamble = rs.iter().find(|r| r.kind == 0x3e8).unwrap();
        blank[preamble.data.start + 24..preamble.data.start + 28]
            .copy_from_slice(&0_u32.to_le_bytes());
        Self::from_bytes(&blank, None).unwrap()
    }
    pub fn open(path: &Path) -> Result<Self, String> {
        Self::from_bytes(
            &fs::read(path).map_err(|e| e.to_string())?,
            Some(path.to_path_buf()),
        )
    }
    pub fn from_bytes(b: &[u8], path: Option<PathBuf>) -> Result<Self, String> {
        let rs = records(b)?;
        let mut objects: Vec<Object> = Vec::new();
        let mut i = 0;
        let mut safe = true;
        let (mut page, mut page_base, mut page_count) = (0, 0, 0);
        while i < rs.len() {
            if rs[i].kind == 0x3e8 {
                if page_count > 0 {
                    page += 1;
                    page_base = objects.len();
                }
                page_count += 1;
                i += 1;
                continue;
            }
            if rs[i].kind != 0x7d0 {
                i += 1;
                continue;
            }
            let h = &b[rs[i].data.clone()];
            if h.len() != 16 && h.len() != 28 {
                return Err(format!("Invalid object header at {:#x}", rs[i].start));
            }
            let header_offset = rs[i].data.start;
            let raw = u32_at(h, 0);
            let code = raw as u16;
            let mut point = None;
            let mut point_offset = None;
            let mut refs = Vec::new();
            i += 1;
            while i < rs.len() && rs[i].kind != 0x7d7 {
                let r = &rs[i];
                let d = &b[r.data.clone()];
                match r.kind {
                    0x899 if d.len() == 16 => {
                        point = Some(Point {
                            x: f64_at(d, 0),
                            y: f64_at(d, 8),
                        });
                        point_offset = Some(r.data.start);
                    }
                    0x7d2 | 0x7d3 if d.len() >= 4 && (d.len() - 4) % 4 == 0 => {
                        let n = u32_at(d, 0) as usize;
                        if n == (d.len() - 4) / 4 {
                            refs = (0..n)
                                .map(|j| {
                                    let raw = u32_at(d, 4 + j * 4) as usize;
                                    if raw == 0 { 0 } else { page_base + raw }
                                })
                                .collect();
                        }
                    }
                    _ => {}
                }
                if !matches!(r.kind, 0x899 | 0x7d2 | 0x7d3) {
                    safe = false;
                }
                i += 1;
            }
            if i == rs.len() {
                return Err("Unterminated object group".into());
            }
            let kind = match (code, point, refs.as_slice()) {
                (0, Some(p), _) => Kind::Free(p),
                (1, _, [segment]) => {
                    match objects.get(segment.saturating_sub(1)).map(|o| &o.kind) {
                        Some(Kind::Segment(a, b)) => Kind::Midpoint(*a, *b),
                        _ => Kind::Unsupported(code),
                    }
                }
                (2, _, [a, b]) => Kind::Segment(*a, *b),
                (3, _, [a, b]) => Kind::Circle(*a, *b),
                (5, _, [a, b]) => Kind::Perpendicular(*a, *b),
                (6, _, [a, b]) => Kind::Parallel(*a, *b),
                (63, _, [a, b]) => Kind::Line(*a, *b),
                (64, _, [a, b]) => Kind::Ray(*a, *b),
                _ => Kind::Unsupported(code),
            };
            if matches!(kind, Kind::Unsupported(_)) {
                safe = false;
            }
            objects.push(Object {
                id: objects.len() + 1,
                page,
                kind,
                visible: raw & 0x10000 == 0,
                point_offset,
                header_offset: Some(header_offset),
            });
            i += 1;
        }
        let source_count = objects.len();
        let preambles: Vec<_> = rs
            .iter()
            .filter(|r| r.kind == 0x3e8 && r.data.len() >= 28)
            .collect();
        if preambles.len() != 1 || u32_at(b, preambles[0].data.start + 24) as usize != source_count
        {
            safe = false;
        }
        Ok(Self {
            objects,
            path,
            selected: BTreeSet::new(),
            dirty: false,
            partial_view: !safe,
            active_page: 0,
            page_count: page_count.max(1),
            original: b.to_vec(),
            source_count,
            safe_structure: safe,
            undo: Vec::new(),
            redo: Vec::new(),
        })
    }
    pub fn checkpoint(&mut self) {
        self.undo.push(self.objects.clone());
        self.redo.clear();
        self.dirty = true;
    }
    pub fn undo(&mut self) {
        if let Some(p) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.objects, p));
            self.dirty = true;
        }
    }
    pub fn redo(&mut self) {
        if let Some(p) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.objects, p));
            self.dirty = true;
        }
    }
    pub fn add(&mut self, kind: Kind) -> usize {
        self.checkpoint();
        let id = self.objects.len() + 1;
        self.objects.push(Object {
            id,
            page: self.active_page,
            kind,
            visible: true,
            point_offset: None,
            header_offset: None,
        });
        id
    }
    pub fn set_selected_visibility(&mut self, visible: bool) {
        if !self
            .objects
            .iter()
            .any(|o| self.selected.contains(&o.id) && o.visible != visible)
        {
            return;
        }
        self.checkpoint();
        for o in &mut self.objects {
            if self.selected.contains(&o.id) {
                o.visible = visible;
            }
        }
        if !visible {
            self.selected.clear();
        }
    }
    pub fn show_all_hidden(&mut self) {
        if !self
            .objects
            .iter()
            .any(|o| o.page == self.active_page && !o.visible)
        {
            return;
        }
        self.checkpoint();
        for o in &mut self.objects {
            if o.page == self.active_page {
                o.visible = true;
            }
        }
    }
    pub fn position(&self, id: usize) -> Option<Point> {
        fn go(d: &Document, id: usize, seen: &mut BTreeSet<usize>) -> Option<Point> {
            if !seen.insert(id) {
                return None;
            }
            let p = match &d.objects.get(id.checked_sub(1)?)?.kind {
                Kind::Free(p) => Some(*p),
                Kind::Midpoint(a, b) => {
                    let a = go(d, *a, seen)?;
                    let b = go(d, *b, seen)?;
                    Some(Point {
                        x: (a.x + b.x) / 2.,
                        y: (a.y + b.y) / 2.,
                    })
                }
                _ => None,
            };
            seen.remove(&id);
            p
        }
        go(self, id, &mut BTreeSet::new())
    }
    pub fn line_points(&self, id: usize) -> Option<(Point, Point, LineMode)> {
        fn go(
            d: &Document,
            id: usize,
            seen: &mut BTreeSet<usize>,
        ) -> Option<(Point, Point, LineMode)> {
            if !seen.insert(id) {
                return None;
            }
            let result = match d.objects.get(id.checked_sub(1)?)?.kind {
                Kind::Segment(a, b) => Some((d.position(a)?, d.position(b)?, LineMode::Segment)),
                Kind::Line(a, b) => Some((d.position(a)?, d.position(b)?, LineMode::Line)),
                Kind::Ray(a, b) => Some((d.position(a)?, d.position(b)?, LineMode::Ray)),
                Kind::Parallel(p, line) | Kind::Perpendicular(p, line) => {
                    let origin = d.position(p)?;
                    let (a, b, _) = go(d, line, seen)?;
                    let mut direction = Point {
                        x: b.x - a.x,
                        y: b.y - a.y,
                    };
                    if matches!(d.objects[id - 1].kind, Kind::Perpendicular(_, _)) {
                        direction = Point {
                            x: -direction.y,
                            y: direction.x,
                        };
                    }
                    Some((
                        origin,
                        Point {
                            x: origin.x + direction.x,
                            y: origin.y + direction.y,
                        },
                        LineMode::Line,
                    ))
                }
                _ => None,
            };
            seen.remove(&id);
            result
        }
        go(self, id, &mut BTreeSet::new())
    }
    pub fn move_point(&mut self, id: usize, p: Point) -> bool {
        if !p.x.is_finite() || !p.y.is_finite() {
            return false;
        }
        if let Some(Object {
            kind: Kind::Free(old),
            ..
        }) = self.objects.get_mut(id.saturating_sub(1))
        {
            *old = p;
            self.dirty = true;
            true
        } else {
            false
        }
    }
    pub fn can_save(&self) -> Result<(), String> {
        if self.objects.len() < self.source_count {
            return Err("Removing saved objects is not yet supported by the GSP writer".into());
        }
        for (i, o) in self.objects.iter().enumerate() {
            if o.id != i + 1 {
                return Err("Invalid object ID".into());
            }
            if let Kind::Free(point) = o.kind
                && (!point.x.is_finite() || !point.y.is_finite())
            {
                return Err(format!("Non-finite coordinates in object {}", o.id));
            }
            let refs = match o.kind {
                Kind::Midpoint(a, b)
                | Kind::Segment(a, b)
                | Kind::Line(a, b)
                | Kind::Ray(a, b)
                | Kind::Parallel(a, b)
                | Kind::Perpendicular(a, b)
                | Kind::Circle(a, b) => vec![a, b],
                _ => vec![],
            };
            if refs.iter().any(|&r| r == 0 || r >= o.id) {
                return Err(format!("Invalid dependency in object {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Segment(a,b)|Kind::Line(a,b)|Kind::Ray(a,b)|Kind::Circle(a,b)|Kind::Midpoint(a,b)
                if self.position(a).is_none()||self.position(b).is_none())
            {
                return Err(format!("Invalid point dependency in object {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Parallel(_,line)|Kind::Perpendicular(_,line) if self.line_points(line).is_none())
            {
                return Err(format!("Invalid source line in object {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Parallel(point,_)|Kind::Perpendicular(point,_) if self.position(point).is_none())
            {
                return Err(format!("Invalid anchor point in object {}", o.id));
            }
            if matches!(o.kind, Kind::Unsupported(_)) && i >= self.source_count {
                return Err("Unsupported new object".into());
            }
        }
        if self.objects.len() > self.source_count && !self.safe_structure {
            return Err("This sketch has unsupported object records. Adding objects could change their references; save refused.".into());
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.can_save()?;
        let rs = records(&self.original)?;
        let mut out = self.original.clone();
        for o in self.objects.iter().take(self.source_count) {
            if let Some(i) = o.header_offset {
                let mut class = u32_at(&out, i);
                if o.visible {
                    class &= !0x1_0000;
                } else {
                    class |= 0x1_0000;
                }
                out[i..i + 4].copy_from_slice(&class.to_le_bytes());
            }
            if let (Kind::Free(p), Some(i)) = (&o.kind, o.point_offset) {
                out[i..i + 8].copy_from_slice(&p.x.to_le_bytes());
                out[i + 8..i + 16].copy_from_slice(&p.y.to_le_bytes());
            }
        }
        if self.objects.len() == self.source_count {
            return Ok(out);
        }
        let preamble = rs
            .iter()
            .find(|r| r.kind == 0x3e8 && r.data.len() >= 28)
            .ok_or("Missing document preamble")?;
        out[preamble.data.start + 24..preamble.data.start + 28]
            .copy_from_slice(&(self.objects.len() as u32).to_le_bytes());
        let trailer = rs
            .iter()
            .find(|r| r.kind == 0x3e9)
            .ok_or("Missing document trailer")?
            .start;
        let last_group_end = rs
            .iter()
            .filter(|r| r.kind == 0x7d7 && r.start < trailer)
            .map(|r| r.data.end)
            .max()
            .unwrap_or(0);
        let insert_at = rs
            .iter()
            .find(|r| r.kind == 0x2328 && r.start >= last_group_end && r.start < trailer)
            .map(|r| r.start)
            .unwrap_or(trailer);
        let mut added = Vec::new();
        for o in self.objects.iter().skip(self.source_count) {
            let (code, refs): (u16, Vec<usize>) = match o.kind {
                Kind::Free(_) => (0, vec![]),
                Kind::Midpoint(a, b) => {
                    let s = self
                        .objects
                        .iter()
                        .find(|s| matches!(s.kind,Kind::Segment(x,y) if (x==a&&y==b)||(x==b&&y==a)))
                        .ok_or("Midpoint requires an existing segment")?;
                    (1, vec![s.id])
                }
                Kind::Segment(a, b) => (2, vec![a, b]),
                Kind::Circle(a, b) => (3, vec![a, b]),
                Kind::Perpendicular(a, b) => (5, vec![a, b]),
                Kind::Parallel(a, b) => (6, vec![a, b]),
                Kind::Line(a, b) => (63, vec![a, b]),
                Kind::Ray(a, b) => (64, vec![a, b]),
                _ => return Err("Unsupported new object".into()),
            };
            let mut h = header(code);
            if !o.visible {
                h[2] = 1;
            }
            put_record(&mut added, 0x7d0, &h);
            if let Kind::Free(p) = o.kind {
                let mut pair = Vec::new();
                pair.extend_from_slice(&p.x.to_le_bytes());
                pair.extend_from_slice(&p.y.to_le_bytes());
                put_record(&mut added, 0x899, &pair);
            } else {
                let mut idx = Vec::new();
                idx.extend_from_slice(&(refs.len() as u32).to_le_bytes());
                for r in refs {
                    idx.extend_from_slice(&(r as u32).to_le_bytes());
                }
                put_record(&mut added, 0x7d2, &idx);
            }
            put_record(&mut added, 0x7d7, &[]);
        }
        out.splice(insert_at..insert_at, added);
        records(&out)?;
        Ok(out)
    }
    pub fn save_to(&mut self, path: &Path) -> Result<(), String> {
        let data = self.encode()?;
        let reparsed = Self::from_bytes(&data, Some(path.to_path_buf()))?;
        if reparsed.objects.len() != self.objects.len()
            || reparsed.objects.iter().zip(&self.objects).any(|(saved, current)| {
                saved.id != current.id
                    || saved.page != current.page
                    || saved.kind != current.kind
                    || saved.visible != current.visible
            })
        {
            return Err("Serialized sketch does not preserve the edited objects".into());
        }
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let stem = path.file_name().ok_or("Invalid path")?.to_string_lossy();
        let mut n = 0;
        let temp = loop {
            let p = parent.join(format!(".{stem}.{}.{}.tmp", std::process::id(), n));
            if !p.exists() {
                break p;
            }
            n += 1;
        };
        let result = (|| -> Result<(), String> {
            let mut f = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp)
                .map_err(|e| e.to_string())?;
            f.write_all(&data).map_err(|e| e.to_string())?;
            f.sync_all().map_err(|e| e.to_string())?;
            fs::rename(&temp, path).map_err(|e| e.to_string())?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result?;
        for (object, saved) in self.objects.iter_mut().zip(&reparsed.objects) {
            object.point_offset = saved.point_offset;
            object.header_offset = saved.header_offset;
        }
        self.original = data;
        self.source_count = self.objects.len();
        self.path = Some(path.to_path_buf());
        self.safe_structure = reparsed.safe_structure;
        self.partial_view = reparsed.partial_view;
        self.undo.clear();
        self.redo.clear();
        self.dirty = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixtures_preserve_bytes() {
        for name in [
            "point",
            "point_hidden",
            "points",
            "segment",
            "circle",
            "middle_point",
            "line",
            "ray",
            "parallel",
            "perp",
            "example_tools",
        ] {
            let p = format!("tests/fixtures/{name}.gsp");
            let b = fs::read(p).unwrap();
            let d = Document::from_bytes(&b, None).unwrap();
            assert_eq!(d.encode().unwrap(), b);
        }
    }
    #[test]
    fn new_roundtrip_and_dependencies() {
        let mut d = Document::new();
        let a = d.add(Kind::Free(Point { x: 10., y: 20. }));
        let b = d.add(Kind::Free(Point { x: 30., y: 40. }));
        d.add(Kind::Segment(a, b));
        let m = d.add(Kind::Midpoint(a, b));
        assert_eq!(d.position(m), Some(Point { x: 20., y: 30. }));
        let reopened = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.position(m), d.position(m));
    }
    #[test]
    fn undo_redo() {
        let mut d = Document::new();
        d.add(Kind::Free(Point { x: 1., y: 2. }));
        d.undo();
        assert!(d.objects.is_empty());
        d.redo();
        assert_eq!(d.objects.len(), 1);
    }
    #[test]
    fn invalid_ref() {
        let mut d = Document::new();
        d.add(Kind::Segment(1, 2));
        assert!(d.encode().is_err());
    }
    #[test]
    fn hidden_point_matches_original_sketchpad_bytes() {
        let mut d = Document::open(Path::new("tests/fixtures/point.gsp")).unwrap();
        d.selected.insert(1);
        d.set_selected_visibility(false);
        assert_eq!(
            d.encode().unwrap(),
            fs::read("tests/fixtures/point_hidden.gsp").unwrap()
        );
        d.undo();
        assert!(d.objects[0].visible);
    }
    #[test]
    fn pages_and_selection_metadata() {
        let tools = Document::open(Path::new("tests/fixtures/example_tools.gsp")).unwrap();
        assert_eq!(tools.page_count, 10);
        assert!(tools.objects.iter().any(|o| o.page == 9));
        assert!(tools.partial_view);
        let mut segment = Document::open(Path::new("tests/fixtures/segment.gsp")).unwrap();
        segment.add(Kind::Free(Point { x: 1., y: 2. }));
        let bytes = segment.encode().unwrap();
        let rs = records(&bytes).unwrap();
        let last_group = rs.iter().rposition(|r| r.kind == 0x7d7).unwrap();
        let selected = rs.iter().position(|r| r.kind == 0x2328).unwrap();
        assert!(last_group < selected);
        assert_eq!(Document::from_bytes(&bytes, None).unwrap().objects.len(), 4);
    }
    #[test]
    fn derived_lines_update() {
        let mut d = Document::new();
        let a = d.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = d.add(Kind::Free(Point { x: 10., y: 0. }));
        let c = d.add(Kind::Free(Point { x: 1., y: 5. }));
        let line = d.add(Kind::Line(a, b));
        let parallel = d.add(Kind::Parallel(c, line));
        let perpendicular = d.add(Kind::Perpendicular(c, line));
        assert_eq!(
            d.line_points(parallel),
            Some((
                Point { x: 1., y: 5. },
                Point { x: 11., y: 5. },
                LineMode::Line
            ))
        );
        assert_eq!(
            d.line_points(perpendicular),
            Some((
                Point { x: 1., y: 5. },
                Point { x: 1., y: 15. },
                LineMode::Line
            ))
        );
        d.move_point(b, Point { x: 0., y: 10. });
        assert_eq!(d.line_points(parallel).unwrap().1, Point { x: 1., y: 15. });
        let reopened = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.line_points(parallel), d.line_points(parallel));
    }
    #[test]
    fn save_reopen_edit_and_save_again() {
        let p = std::env::temp_dir().join(format!("sketchpad_rs_test_{}.gsp", std::process::id()));
        let mut d = Document::new();
        let id = d.add(Kind::Free(Point { x: 12., y: 34. }));
        d.save_to(&p).unwrap();
        assert!(d.move_point(id, Point { x: 56., y: 78. }));
        d.save_to(&p).unwrap();
        let r = Document::open(&p).unwrap();
        assert_eq!(r.position(id), Some(Point { x: 56., y: 78. }));
        fs::remove_file(p).unwrap();
    }
}
