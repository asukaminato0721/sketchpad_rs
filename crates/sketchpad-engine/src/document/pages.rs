use super::records::records;
use super::{Document, Kind};
use crate::action_records::BROKEN_PAGE_LINK_TARGET;
use crate::graph::GridForm;

impl Document {
    pub fn add_blank_page(&mut self) -> Result<usize, String> {
        if self.page_count >= 32768 {
            return Err("Too many pages for a Sketchpad document".into());
        }
        let mut candidate = self.clone();
        candidate.checkpoint();
        candidate.page_count += 1;
        candidate.active_page = candidate.page_count - 1;
        candidate.snap_points.push(false);
        candidate.grid_dotted.push(false);
        candidate.grid_form.push(GridForm::Square);
        candidate.page_names.push(None);
        candidate.marked_coordinate_system.push(None);
        candidate.safe_structure = false;
        candidate.partial_view = true;
        candidate.selected.clear();
        let encoded = candidate.encode()?;
        let reparsed = Self::from_bytes(&encoded, None)?;
        let mut survivors: Vec<_> = candidate
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .collect();
        survivors
            .sort_by_key(|object| (object.page, object.id > candidate.source_count, object.id));
        candidate.validate_reparsed_in_order(&reparsed, survivors, true)?;
        *self = candidate;
        Ok(self.page_count)
    }
    pub fn copy_page(&mut self, page: usize) -> Result<usize, String> {
        if page >= self.page_count {
            return Err("Page does not exist".into());
        }
        if self.page_count >= 32768 {
            return Err("Too many pages for a Sketchpad document".into());
        }
        let bytes = self.encode()?;
        let parsed = Self::from_bytes(&bytes, None)?;
        let rs = records(&bytes)?;
        if rs.iter().any(|record| record.kind == 0x0514) {
            return Err("Cannot copy a page from a document with pictures".into());
        }
        let metadata: Vec<_> = rs
            .iter()
            .filter(|record| record.kind == 0x0386 && record.data.len() == 216)
            .collect();
        let preambles: Vec<_> = rs.iter().filter(|record| record.kind == 0x03e8).collect();
        let end = rs
            .last()
            .filter(|record| record.kind == 0x0385)
            .ok_or("Missing document end record")?;
        if metadata.len() != self.page_count
            || preambles.len() != self.page_count
            || (0..self.page_count).any(|index| {
                metadata[index].start >= preambles[index].start
                    || index > 0 && metadata[index].start <= preambles[index - 1].start
            })
        {
            return Err("Document page blocks are inconsistent".into());
        }
        let last_trailer = rs
            .iter()
            .rev()
            .find(|record| record.kind == 0x03e9)
            .ok_or("Missing final page trailer")?;
        if rs.iter().any(|record| {
            record.kind == 0x044c && record.start > last_trailer.start && record.start < end.start
        }) {
            return Err("Custom tool records refer to document pages".into());
        }
        let start = metadata[page].start;
        let finish = metadata
            .get(page + 1)
            .map_or(end.start, |record| record.start);
        let mut expanded = bytes[..end.start].to_vec();
        expanded.extend_from_slice(&bytes[start..finish]);
        expanded.extend_from_slice(&bytes[end.start..]);
        let new_page = self.page_count;
        let new_count = new_page + 1;
        let header = rs
            .iter()
            .find(|record| record.kind == 0x0384 && record.data.len() == 28)
            .ok_or("Missing document page header")?;
        expanded[header.data.start + 2..header.data.start + 4]
            .copy_from_slice(&(new_count as u16).to_le_bytes());
        expanded[header.data.start + 16..header.data.start + 18]
            .copy_from_slice(&(new_page as u16).to_le_bytes());
        expanded[header.data.start + 24..header.data.start + 28]
            .copy_from_slice(&((new_count + 1) as u32).to_le_bytes());
        let mut result = Self::from_bytes(&expanded, self.path.clone())?;
        let source_objects: Vec<_> = parsed
            .objects
            .iter()
            .filter(|object| object.page == page)
            .collect();
        if result.page_count != new_count
            || result.active_page != new_page
            || result.page_names[..new_page] != parsed.page_names
            || result.page_names[new_page] != parsed.page_names[page]
            || result.objects.len() != parsed.objects.len() + source_objects.len()
            || result.objects[..parsed.objects.len()] != *parsed.objects
        {
            return Err("Copied page changes the existing document".into());
        }
        let mut ids = vec![0; parsed.objects.len() + 1];
        for (index, object) in source_objects.iter().enumerate() {
            ids[object.id] = parsed.objects.len() + index + 1;
        }
        for (index, object) in source_objects.iter().enumerate() {
            // A copied page retains serialized page-local numeric source IDs.
            // Only the model's document-wide references move to the new page.
            let expected = object.kind.remap(&ids);
            let copied = &result.objects[parsed.objects.len() + index];
            if copied.page != new_page
                || copied.kind != expected
                || copied.visible != object.visible
                || copied.tracing != object.tracing
                || copied.deleted != object.deleted
                || copied.label != object.label
                || copied.visual_style != object.visual_style
                || copied.text_style_override != object.text_style_override
                || copied.rich_text_override != object.rich_text_override
                || copied.function_anchor != object.function_anchor
                || copied.function_name != object.function_name
            {
                return Err("Copied page changes a source object".into());
            }
        }
        let mut candidate = self.clone();
        candidate.checkpoint_structure();
        result.undo = candidate.undo;
        result.dirty = true;
        *self = result;
        Ok(new_count)
    }
    pub fn remove_page(&mut self, page: usize) -> Result<(), String> {
        if self.page_count <= 1 || page >= self.page_count {
            return Err("Choose a page in a document with at least two pages".into());
        }
        let mut bytes = self.encode()?;
        let parsed = Self::from_bytes(&bytes, None)?;
        let rs = records(&bytes)?;
        if rs.iter().any(|record| record.kind == 0x0514) {
            return Err("Cannot remove a page from a document with pictures".into());
        }
        let metadata: Vec<_> = rs
            .iter()
            .filter(|record| record.kind == 0x0386 && record.data.len() == 216)
            .collect();
        let preambles: Vec<_> = rs.iter().filter(|record| record.kind == 0x03e8).collect();
        let end = rs
            .last()
            .filter(|record| record.kind == 0x0385)
            .ok_or("Missing document end record")?;
        if metadata.len() != self.page_count
            || preambles.len() != self.page_count
            || (0..self.page_count).any(|index| {
                metadata[index].start >= preambles[index].start
                    || index > 0 && metadata[index].start <= preambles[index - 1].start
            })
        {
            return Err("Document page blocks are inconsistent".into());
        }
        let last_trailer = rs
            .iter()
            .rev()
            .find(|record| record.kind == 0x03e9)
            .ok_or("Missing final page trailer")?;
        if rs.iter().any(|record| {
            record.kind == 0x044c && record.start > last_trailer.start && record.start < end.start
        }) {
            return Err("Custom tool records refer to document pages".into());
        }
        let mut broken_page_links = 0;
        for object in &parsed.objects {
            if object.page == page {
                continue;
            }
            if let Kind::LinkPageButton { target_page, .. } = &object.kind
                && *target_page < self.page_count
                && *target_page >= page
            {
                let header = object.header_offset.ok_or("Missing Page Link header")?;
                let group = rs
                    .iter()
                    .position(|record| record.kind == 0x07d0 && record.data.start == header)
                    .ok_or("Missing Page Link group")?;
                let action = rs[group + 1..]
                    .iter()
                    .take_while(|record| record.kind != 0x07d7)
                    .find(|record| record.kind == 0x0906 && record.data.len() == 24)
                    .ok_or("Missing Page Link action")?;
                if u16::from_le_bytes(
                    bytes[action.data.start + 14..action.data.start + 16]
                        .try_into()
                        .unwrap(),
                ) as usize
                    != *target_page
                {
                    return Err("Page Link target differs from its record".into());
                }
                // Native Sketchpad keeps a Page Link to a deleted page visible,
                // but invalidates its destination. Its saved target field is
                // 0xbd66 in both the two- and three-page cases observed here.
                let updated = if *target_page == page {
                    broken_page_links += 1;
                    BROKEN_PAGE_LINK_TARGET as u16
                } else {
                    (*target_page - 1) as u16
                };
                bytes[action.data.start + 14..action.data.start + 16]
                    .copy_from_slice(&updated.to_le_bytes());
            }
        }
        let cut_start = metadata[page].start;
        let cut_end = metadata
            .get(page + 1)
            .map_or(end.start, |record| record.start);
        let mut contracted = bytes[..cut_start].to_vec();
        contracted.extend_from_slice(&bytes[cut_end..]);
        let new_count = self.page_count - 1;
        let new_active = if self.active_page > page {
            self.active_page - 1
        } else {
            self.active_page.min(new_count - 1)
        };
        let contracted_records = records(&contracted)?;
        let header = contracted_records
            .iter()
            .find(|record| record.kind == 0x0384 && record.data.len() == 28)
            .ok_or("Missing document page header")?;
        contracted[header.data.start + 2..header.data.start + 4]
            .copy_from_slice(&(new_count as u16).to_le_bytes());
        contracted[header.data.start + 16..header.data.start + 18]
            .copy_from_slice(&(new_active as u16).to_le_bytes());
        contracted[header.data.start + 24..header.data.start + 28]
            .copy_from_slice(&((new_count + 1) as u32).to_le_bytes());
        let page_marker = (new_count * 2 - 1) as u16;
        for record in contracted_records
            .iter()
            .filter(|record| record.kind == 0x0386 && record.data.len() == 216)
        {
            contracted[record.data.start + 6..record.data.start + 8]
                .copy_from_slice(&page_marker.to_le_bytes());
            contracted[record.data.start + 8..record.data.start + 10]
                .copy_from_slice(&page_marker.to_le_bytes());
        }
        let mut result = Self::from_bytes(&contracted, self.path.clone())?;
        if result.page_count != new_count
            || result.objects.len()
                != parsed
                    .objects
                    .iter()
                    .filter(|object| object.page != page)
                    .count()
            || new_count == 1
                && result.partial_view
                && (broken_page_links == 0
                    || result
                        .objects
                        .iter()
                        .any(|object| matches!(object.kind, Kind::Unsupported(_))))
            || parsed
                .objects
                .iter()
                .filter(|object| object.page != page)
                .zip(&result.objects)
                .any(|(before, after)| {
                    matches!(before.kind, Kind::LinkPageButton { target_page, .. } if target_page == page)
                        && !matches!(after.kind, Kind::LinkPageButton { target_page: BROKEN_PAGE_LINK_TARGET, .. })
                })
        {
            return Err("Removing the page would leave an unsupported document".into());
        }
        let mut candidate = self.clone();
        candidate.checkpoint_structure();
        result.undo = candidate.undo;
        result.dirty = true;
        *self = result;
        Ok(())
    }
    pub fn move_page(&mut self, from: usize, to: usize) -> Result<(), String> {
        if from >= self.page_count || to >= self.page_count {
            return Err("Choose two pages in this document".into());
        }
        if from == to {
            return Ok(());
        }
        let mut bytes = self.encode()?;
        let parsed = Self::from_bytes(&bytes, None)?;
        let rs = records(&bytes)?;
        if rs.iter().any(|record| record.kind == 0x0514) {
            return Err("Cannot reorder pages in a document with pictures".into());
        }
        let metadata: Vec<_> = rs
            .iter()
            .filter(|record| record.kind == 0x0386 && record.data.len() == 216)
            .collect();
        let preambles: Vec<_> = rs.iter().filter(|record| record.kind == 0x03e8).collect();
        let end = rs
            .last()
            .filter(|record| record.kind == 0x0385)
            .ok_or("Missing document end record")?;
        if metadata.len() != self.page_count
            || preambles.len() != self.page_count
            || (0..self.page_count).any(|index| {
                metadata[index].start >= preambles[index].start
                    || index > 0 && metadata[index].start <= preambles[index - 1].start
            })
        {
            return Err("Document page blocks are inconsistent".into());
        }
        let last_trailer = rs
            .iter()
            .rev()
            .find(|record| record.kind == 0x03e9)
            .ok_or("Missing final page trailer")?;
        if rs.iter().any(|record| {
            record.kind == 0x044c && record.start > last_trailer.start && record.start < end.start
        }) {
            return Err("Custom tool records refer to document pages".into());
        }
        let mut order: Vec<usize> = (0..self.page_count).collect();
        let moved = order.remove(from);
        order.insert(to, moved);
        let mut new_page = vec![0; self.page_count];
        for (position, &old_page) in order.iter().enumerate() {
            new_page[old_page] = position;
        }
        for object in &parsed.objects {
            if let Kind::LinkPageButton { target_page, .. } = &object.kind {
                if *target_page >= self.page_count {
                    continue;
                }
                let header = object.header_offset.ok_or("Missing Page Link header")?;
                let group = rs
                    .iter()
                    .position(|record| record.kind == 0x07d0 && record.data.start == header)
                    .ok_or("Missing Page Link group")?;
                let action = rs[group + 1..]
                    .iter()
                    .take_while(|record| record.kind != 0x07d7)
                    .find(|record| record.kind == 0x0906 && record.data.len() == 24)
                    .ok_or("Missing Page Link action")?;
                if u16::from_le_bytes(
                    bytes[action.data.start + 14..action.data.start + 16]
                        .try_into()
                        .unwrap(),
                ) as usize
                    != *target_page
                {
                    return Err("Page Link target differs from its record".into());
                }
                bytes[action.data.start + 14..action.data.start + 16]
                    .copy_from_slice(&(new_page[*target_page] as u16).to_le_bytes());
            }
        }
        let mut reordered = bytes[..metadata[0].start].to_vec();
        for &old_page in &order {
            let start = metadata[old_page].start;
            let finish = metadata
                .get(old_page + 1)
                .map_or(end.start, |record| record.start);
            reordered.extend_from_slice(&bytes[start..finish]);
        }
        reordered.extend_from_slice(&bytes[end.start..]);
        let reordered_records = records(&reordered)?;
        let header = reordered_records
            .iter()
            .find(|record| record.kind == 0x0384 && record.data.len() == 28)
            .ok_or("Missing document page header")?;
        reordered[header.data.start + 16..header.data.start + 18]
            .copy_from_slice(&(new_page[self.active_page] as u16).to_le_bytes());
        let mut result = Self::from_bytes(&reordered, self.path.clone())?;
        if result.page_count != self.page_count
            || result.objects.len() != parsed.objects.len()
            || result.page_names
                != order
                    .iter()
                    .map(|&old| parsed.page_names[old].clone())
                    .collect::<Vec<_>>()
        {
            return Err("Reordered pages do not preserve the document".into());
        }
        let mut ids = vec![0; parsed.objects.len() + 1];
        let mut expected = Vec::with_capacity(parsed.objects.len());
        for (position, &old_page) in order.iter().enumerate() {
            for object in parsed
                .objects
                .iter()
                .filter(|object| object.page == old_page)
            {
                ids[object.id] = expected.len() + 1;
                expected.push((position, object));
            }
        }
        for (index, (page, object)) in expected.into_iter().enumerate() {
            // Reordering intact page blocks preserves their embedded local IDs.
            let mut kind = object.kind.remap(&ids);
            if let Kind::LinkPageButton { target_page, .. } = &mut kind
                && *target_page < new_page.len()
            {
                *target_page = new_page[*target_page];
            }
            if result.objects[index].page != page || result.objects[index].kind != kind {
                return Err("Reordered pages change a surviving object".into());
            }
        }
        let mut candidate = self.clone();
        candidate.checkpoint_structure();
        result.undo = candidate.undo;
        result.dirty = true;
        *self = result;
        Ok(())
    }
    pub fn set_page_name(&mut self, page: usize, name: &str) -> Result<(), String> {
        if page >= self.page_count {
            return Err("Page does not exist".into());
        }
        if name.as_bytes().contains(&0) || name.len() > 255 {
            return Err("Page name must be at most 255 bytes and contain no NUL".into());
        }
        let desired = (!name.is_empty()).then(|| name.to_owned());
        if self.page_names[page] == desired {
            return Ok(());
        }
        let mut candidate = self.clone();
        candidate.checkpoint();
        candidate.page_names[page] = desired;
        let encoded = candidate.encode()?;
        let reparsed = Self::from_bytes(&encoded, None)?;
        let mut survivors: Vec<_> = candidate
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .collect();
        survivors
            .sort_by_key(|object| (object.page, object.id > candidate.source_count, object.id));
        candidate.validate_reparsed_in_order(&reparsed, survivors, true)?;
        *self = candidate;
        Ok(())
    }
}
