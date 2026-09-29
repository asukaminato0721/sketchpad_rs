use super::Document;
use super::records::{header, put_record, records};

impl Document {
    pub(super) fn needs_legacy_upgrade(&self) -> Result<bool, String> {
        let rs = records(&self.original)?;
        Ok(rs.iter().any(|record| {
            record.kind == 0x0384
                && self.original.get(record.data.start..record.data.start + 2) == Some(&[4, 0])
        }) && rs
            .iter()
            .any(|record| record.kind == 0x07d0 && record.data.len() == 16))
    }

    pub(super) fn upgrade_legacy_document(&mut self) -> Result<(), String> {
        // Native Sketchpad upgrades all short headers before pasting modern objects.
        // Keep the original page preferences, font tables and compatibility records.
        let bytes = self.encode()?;
        let rs = records(&bytes)?;
        let mut upgraded = b"GSP4".to_vec();
        for record in rs {
            let payload = &bytes[record.data.clone()];
            if record.kind == 0x0384 {
                if payload.len() != 28 || payload[..2] != [4, 0] {
                    return Err("Invalid legacy document version".into());
                }
                let mut version = payload.to_vec();
                version[..2].copy_from_slice(&[5, 0]);
                put_record(&mut upgraded, record.kind, &version);
            } else if record.kind == 0x07d0 && payload.len() == 16 {
                let code = u16::from_le_bytes(payload[..2].try_into().unwrap());
                let free = code == 0 && payload[6..12] == [1, 0, 4, 0, 1, 0];
                let polygon = (code == 8
                    && payload[6..8] == [4, 0]
                    && u16::from_le_bytes(payload[8..10].try_into().unwrap()) >= 52
                    || code == 10 && payload[6..10] == [5, 0, 16, 0])
                    && payload[10..12] == [1, 0];
                let stroke = payload[11] == 0
                    && payload[10] <= 2
                    && matches!(
                        (
                            code,
                            u16::from_le_bytes(payload[6..8].try_into().unwrap()),
                            u16::from_le_bytes(payload[8..10].try_into().unwrap())
                        ),
                        (2, 2, 14) | (3, 3, 28) | (64, 2, 13) | (5, 2, 47)
                    );
                if !free && !polygon && !stroke {
                    return Err("This legacy object header cannot yet be upgraded".into());
                }
                let mut modern = header(code);
                modern[..16].copy_from_slice(payload);
                modern[10] = if stroke {
                    match payload[10] {
                        0 => 0x11,
                        1 => 0x21,
                        2 => 0x22,
                        _ => unreachable!(),
                    }
                } else {
                    payload[10] | 0x20
                };
                modern[15] = 0;
                if free {
                    modern[11] = 1; // Proven implicit Small style in the old free-point layout.
                } else if polygon {
                    modern[16..18].copy_from_slice(&0xbfffu16.to_le_bytes());
                    modern[20..24].copy_from_slice(&0i32.to_le_bytes());
                } else {
                    modern[11] = 1;
                }
                put_record(&mut upgraded, record.kind, &modern);
            } else {
                upgraded
                    .extend_from_slice(&bytes[record.start..record.data.end + payload.len() % 2]);
            }
        }
        let reparsed = Self::from_bytes(&upgraded, self.path.clone())?;
        if reparsed.objects.len() != self.objects.iter().filter(|o| !o.deleted).count()
            || reparsed
                .objects
                .iter()
                .any(|o| matches!(o.kind, super::Kind::Unsupported(_)))
        {
            return Err("Legacy upgrade changed the supported object graph".into());
        }
        let mut expected = self.clone();
        for object in &mut expected.objects {
            if object.kind.is_filled_region() && object.interior_order.is_none() {
                object.interior_order = Some(0);
            }
        }
        expected.validate_reparsed_document(&reparsed)?;
        self.original = upgraded;
        self.measurement_names = reparsed.measurement_names;
        self.objects = reparsed.objects;
        self.marked_coordinate_system = reparsed.marked_coordinate_system;
        self.frame_styles = reparsed.frame_styles;
        self.run_font_families.clear();
        self.source_count = self.objects.len();
        self.safe_structure = reparsed.safe_structure;
        self.image_read_only = reparsed.image_read_only;
        self.partial_view = reparsed.partial_view;
        Ok(())
    }
}
