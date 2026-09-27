//! Native anchored numeric reference macros; cached spelling is presentation data.
use super::kind::is_point_kind;
use super::{Kind, Object};

pub(super) fn decode_numeric_label(
    objects: &[Object],
    page: usize,
    id: usize,
    header: &[u8],
    refs: &[usize],
    group: &[u8],
    font_count: usize,
) -> Option<Kind> {
    if header.len() != 28 || header[..2] != [90, 0] || header[6..10] != [0, 0, 59, 0] {
        return None;
    }
    let [point, value] = refs else { return None };
    let get = |parent: usize| {
        objects
            .get(parent.checked_sub(1)?)
            .filter(|o| parent < id && o.id == parent && o.page == page && !o.deleted)
    };
    if !is_point_kind(&get(*point)?.kind) || get(*value)?.kind.numeric_unit().is_none() {
        return None;
    }
    let mut records = Vec::new();
    let mut at = 0;
    while at < group.len() {
        let prefix = group.get(at..at + 8)?;
        let len = u32::from_le_bytes(prefix[..4].try_into().ok()?) as usize;
        let kind = u32::from_le_bytes(prefix[4..].try_into().ok()?);
        let end = at.checked_add(8)?.checked_add(len)?;
        records.push((kind, group.get(at + 8..end)?));
        at = end.checked_add(len % 2)?;
    }
    if at != group.len() {
        return None;
    }
    let [(0x07d0, _), (0x07d2, parents), (0x08fc, rich), (0x07d8, font), tail @ ..] =
        records.as_slice()
    else {
        return None;
    };
    if parents.len() != 12 || parents[..4] != 2u32.to_le_bytes() || font.len() != 2 {
        return None;
    }
    let font = u16::from_le_bytes((*font).try_into().ok()?) as usize;
    if font >= font_count {
        return None;
    }
    if !matches!(tail,[(0x07d7,p)] if p.is_empty())
        && !matches!(tail,[(0x232e,m),(0x07d7,p)] if m.is_empty()&&p.is_empty())
    {
        return None;
    }
    if rich.len() < 20 || rich[..4] != 0x08fcu32.to_le_bytes() || rich[8..12] != 4u32.to_le_bytes()
    {
        return None;
    }
    let inner = u32::from_le_bytes(rich[4..8].try_into().ok()?) as usize;
    let count = u16::from_le_bytes(rich[16..18].try_into().ok()?) as usize;
    let end = 18usize.checked_add(count)?;
    if inner < end + 1
        || inner > rich.len()
        || rich.len() - inner > 3
        || *rich.get(end)? != 0
    {
        return None;
    }
    let markup = rich.get(18..end)?;
    if markup != b"<?x2>" {
        let cache = markup.strip_prefix(b"<?1x2")?.strip_suffix(b">")?;
        // Reuse the established literal/style grammar for the cached subtree;
        // expression macros and other reference targets retain unsupported status.
        let mut literal = rich[..18].to_vec();
        literal[12..16].fill(0);
        literal[16..18].copy_from_slice(&u16::try_from(cache.len()).ok()?.to_le_bytes());
        literal.extend_from_slice(cache);
        literal.push(0);
        let len = literal.len() as u32;
        literal[4..8].copy_from_slice(&len.to_le_bytes());
        let bbox = [0, 0, 0, 0, 1, 0, 1, 0];
        crate::text_object::decode_rich_text(&literal, &bbox).ok()?;
    }
    Some(Kind::PointNumericLabel(*point, *value))
}
