use super::calculations::calculation_source_ids;
use super::kind::{is_arc_kind, is_locus_polygon_kind, is_locus_straight_kind, polygon_vertex_count_for_encoding, straight_kind_mode};
use super::records::{header, polygon_subtype, put_record, records, u32_at};
use super::{Document, IterationDepth, Kind, LineMode};
use crate::action_records::{
    AnimationButtonGroup, HideShowButtonGroup, LinkUrlButtonGroup, MovementButtonGroup,
    PresentationButtonGroup, ScrollButtonGroup,
};
use crate::coordinate_measure_records::CoordinateMeasureKind;
use crate::graph::GridForm;
use crate::image_records::ImageFile;
use crate::plot_points_records::PlotPointsMode;
use crate::value_equation_records::ValueEquationKind;
use crate::{
    coordinate_measure_records, function_records, graph, plot_axis_records, plot_points_records,
    text_object, value_equation_records,
};

impl Document {
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        let bytes = self.patch_mapped_iteration_image_levels(self.encode_core()?)?;
        self.encode_page_names(self.patch_active_page(self.patch_measurement_precisions(bytes)?)?)
    }
    fn encode_core(&self) -> Result<Vec<u8>, String> {
        if !self.dirty {
            return Ok(self.original.clone());
        }
        if self.page_count
            > records(&self.original)?
                .iter()
                .filter(|record| record.kind == 0x03e8)
                .count()
        {
            return self.encode_added_pages();
        }
        if self.page_count > 1 && self.objects.iter().any(|object| object.deleted) {
            return self.encode_multi_page_deletions();
        }
        if self.has_pending_multi_page_additions() {
            return self.encode_multi_page_additions();
        }
        if self.page_count > 1 {
            return self.encode_multi_page_state();
        }
        self.can_save()?;
        let rs = records(&self.original)?;
        let mut out = self.original.clone();
        if let Some(record) = rs.iter().find(|record| record.kind == 0x273a) {
            if record.data.len() != 2 {
                return Err("Invalid angle-unit record".into());
            }
            out[record.data.start..record.data.end]
                .copy_from_slice(&[u8::from(!self.angle_unit_radians), 0]);
        } else if self.angle_unit_radians {
            return Err("Missing angle-unit record".into());
        }
        for o in self.objects.iter().take(self.source_count) {
            if let Some(i) = o.header_offset {
                let mut class = u32_at(&out, i);
                if matches!(o.kind, Kind::GraphVerticalUnit(_, _)) {
                    let form = self
                        .coordinate_system_for_object(o.id)
                        .and_then(|frame| self.frame_styles.get(&frame).copied())
                        .map_or(self.grid_form[o.page], |style| style.0);
                    let code = if form == GridForm::Rectangular {
                        55
                    } else {
                        54
                    };
                    class = (class & !0xffff) | code;
                }
                if let Kind::DistanceVerticalUnit(_, _, independent) = o.kind {
                    let code = if independent { 57 } else { 56 };
                    class =
                        (class & !0x0004_ffff) | code | if independent { 0 } else { 0x0004_0000 };
                }
                if o.visible {
                    class &= !0x1_0000;
                } else {
                    class |= 0x1_0000;
                }
                out[i..i + 4].copy_from_slice(&class.to_le_bytes());
                if matches!(o.kind, Kind::VectorTranslatedPolygon(_, _, _))
                    && self.objects.iter().any(|candidate| {
                        !candidate.deleted
                            && matches!(&candidate.kind, Kind::Locus(refs, _) if refs.first() == Some(&o.id))
                    })
                {
                    out[i + 18..i + 20].copy_from_slice(&[0x39, 0x20]);
                }
            }
            if let (Kind::Free(p), Some(i)) = (&o.kind, o.point_offset) {
                out[i..i + 8].copy_from_slice(&p.x.to_le_bytes());
                out[i + 8..i + 16].copy_from_slice(&p.y.to_le_bytes());
            }
            if let Some(i) = o.constraint_offset {
                match o.kind {
                    Kind::GraphVerticalUnit(_, step)
                        if matches!(out.get(i..i + 4), Some([55, 0, 0, 0] | [54, 0, 0, 0])) =>
                    {
                        let form = self
                            .coordinate_system_for_object(o.id)
                            .and_then(|frame| self.frame_styles.get(&frame).copied())
                            .map_or(self.grid_form[o.page], |style| style.0);
                        let code: u16 = if form == GridForm::Rectangular {
                            55
                        } else {
                            54
                        };
                        out[i..i + 2].copy_from_slice(&code.to_le_bytes());
                        out[i + 4..i + 12].copy_from_slice(&step.to_le_bytes());
                    }
                    Kind::DistanceVerticalUnit(_, step, independent)
                        if matches!(out.get(i..i + 4), Some([56, 0, 0, 0] | [57, 0, 0, 0])) =>
                    {
                        let code: u16 = if independent { 57 } else { 56 };
                        out[i..i + 2].copy_from_slice(&code.to_le_bytes());
                        out[i + 4..i + 12].copy_from_slice(&step.to_le_bytes());
                    }
                    Kind::PointOnSegment(_, t) if out.get(i..i + 4) == Some(&[15, 0, 14, 0]) => {
                        out[i + 4..i + 12].copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnRay(_, t) if out.get(i..i + 4) == Some(&[15, 0, 13, 0]) => {
                        out[i + 4..i + 12].copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnLine(_, t) if out.get(i..i + 4) == Some(&[15, 0, 12, 0]) => {
                        out[i + 4..i + 12].copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnArc(_, t) if out.get(i..i + 4) == Some(&[15, 0, 67, 0]) => {
                        out[i + 4..i + 12].copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnPolygon(_, edge, t) if out.get(i..i + 2) == Some(&[15, 0]) => {
                        out[i + 4..i + 12].copy_from_slice(&t.to_le_bytes());
                        out[i + 12..i + 16].copy_from_slice(&(edge as u32).to_le_bytes());
                    }
                    Kind::PointOnCircle(_, x, y) if out.get(i..i + 4) == Some(&[15, 0, 28, 0]) => {
                        out[i + 4..i + 12].copy_from_slice(&x.to_le_bytes());
                        out[i + 12..i + 20].copy_from_slice(&y.to_le_bytes());
                    }
                    _ => {}
                }
            }
            if let (
                Kind::HideShowButton {
                    targets,
                    action,
                    label,
                    anchor,
                },
                Some(i),
            ) = (&o.kind, o.header_offset)
            {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x7d0 && record.data.start == i)
                    .ok_or("Missing saved action button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x7d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved action button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = HideShowButtonGroup::parse(&out[start..end])?;
                group.set_targets(
                    &targets
                        .iter()
                        .map(|&target| target as u32)
                        .collect::<Vec<_>>(),
                )?;
                group.set_action(*action);
                group.set_label(label)?;
                group.set_screen_anchor(*anchor);
                group.set_hidden(!o.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err("Editing a saved button label cannot change its byte length".into());
                }
                out[start..end].copy_from_slice(&encoded);
            }
            if let (
                Kind::AnimationButton {
                    targets,
                    label,
                    anchor,
                    settings,
                },
                Some(i),
            ) = (&o.kind, o.header_offset)
            {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x7d0 && record.data.start == i)
                    .ok_or("Missing saved animation button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x7d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved animation button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = AnimationButtonGroup::parse(&out[start..end])?;
                group.set_targets(
                    &targets
                        .iter()
                        .map(|&target| target as u32)
                        .collect::<Vec<_>>(),
                )?;
                group.set_label(label)?;
                group.set_screen_anchor(*anchor);
                group.set_settings_list(settings)?;
                group.set_hidden(!o.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err(
                        "Editing a saved animation button label cannot change its byte length"
                            .into(),
                    );
                }
                out[start..end].copy_from_slice(&encoded);
            }
            if let (
                Kind::MovementButton {
                    source,
                    destination,
                    speed,
                    follow,
                    label,
                    anchor,
                },
                Some(i),
            ) = (&o.kind, o.header_offset)
            {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x7d0 && record.data.start == i)
                    .ok_or("Missing saved Movement button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x7d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved Movement button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = MovementButtonGroup::parse(&out[start..end])?;
                group.set_targets(*source as u32, *destination as u32)?;
                group.set_speed(*speed);
                group.set_follows_destination(*follow);
                group.set_label(label)?;
                group.set_screen_anchor(*anchor);
                group.set_hidden(!o.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err(
                        "Editing a saved Movement button label cannot change its byte length"
                            .into(),
                    );
                }
                out[start..end].copy_from_slice(&encoded);
            }
            if let (
                Kind::ScrollButton {
                    target,
                    placement,
                    label,
                    anchor,
                },
                Some(i),
            ) = (&o.kind, o.header_offset)
            {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x7d0 && record.data.start == i)
                    .ok_or("Missing saved Scroll button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x7d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved Scroll button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = ScrollButtonGroup::parse(&out[start..end])?;
                group.set_target(*target as u32)?;
                group.set_placement(*placement);
                group.set_label(label)?;
                group.set_screen_anchor(*anchor);
                group.set_hidden(!o.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err(
                        "Editing a saved Scroll button label cannot change its byte length".into(),
                    );
                }
                out[start..end].copy_from_slice(&encoded);
            }
            if let (
                Kind::PresentationButton {
                    targets,
                    mode,
                    label,
                    anchor,
                },
                Some(i),
            ) = (&o.kind, o.header_offset)
            {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x7d0 && record.data.start == i)
                    .ok_or("Missing saved Presentation button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x7d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved Presentation button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = PresentationButtonGroup::parse(&out[start..end])?;
                group.set_targets(
                    &targets
                        .iter()
                        .map(|&target| target as u32)
                        .collect::<Vec<_>>(),
                )?;
                group.set_mode(*mode);
                group.set_label(label)?;
                group.set_screen_anchor(*anchor);
                group.set_hidden(!o.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err(
                        "Editing a saved Presentation button label cannot change its byte length"
                            .into(),
                    );
                }
                out[start..end].copy_from_slice(&encoded);
            }
            if let (Kind::LinkUrlButton { url, label, anchor }, Some(i)) =
                (&o.kind, o.header_offset)
            {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x7d0 && record.data.start == i)
                    .ok_or("Missing saved Link URL button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x7d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved Link URL button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = LinkUrlButtonGroup::parse(&out[start..end])?;
                group.set_url(url)?;
                group.set_label(label)?;
                group.set_screen_anchor(*anchor)?;
                group.set_hidden(!o.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err(
                        "Editing a saved Link URL button cannot change its byte length".into(),
                    );
                }
                out[start..end].copy_from_slice(&encoded);
            }
        }
        if !self.image_read_only
            && let Some(Kind::Image { transform, .. }) = self
                .objects
                .iter()
                .take(self.source_count)
                .find_map(|object| {
                    matches!(object.kind, Kind::Image { .. }).then_some(&object.kind)
                })
        {
            let mut picture = ImageFile::parse(&out)?;
            picture.set_transform(*transform)?;
            out = picture.encode().to_vec();
        }
        let mut page_index = 0;
        for record in rs.iter().filter(|r| r.kind == 0x3e8) {
            if self.objects.iter().any(|o| {
                o.page == page_index && !o.deleted && matches!(o.kind, Kind::CoordinateSystem(_, _))
            }) {
                out[record.data.start + 10] = u8::from(self.snap_points[page_index]);
            }
            page_index += 1;
        }
        for object in self
            .objects
            .iter()
            .take(self.source_count)
            .filter(|object| matches!(object.kind, Kind::CoordinateSystem(_, _)) && !object.deleted)
        {
            let header = object
                .header_offset
                .ok_or("Missing coordinate-system header")?;
            let start = rs
                .iter()
                .position(|record| record.kind == 0x7d0 && record.data.start == header)
                .ok_or("Missing coordinate-system group")?;
            let flag = rs[start + 1..]
                .iter()
                .take_while(|record| record.kind != 0x7d7)
                .find(|record| record.kind == 0x0904 && record.data.len() == 8)
                .ok_or("Missing coordinate-system grid record")?;
            let (form, dotted) = self
                .frame_styles
                .get(&object.id)
                .copied()
                .unwrap_or((self.grid_form[object.page], self.grid_dotted[object.page]));
            out[flag.data.start] = form.code();
            out[flag.data.start + 2] = u8::from(dotted);
        }
        for object in self
            .objects
            .iter()
            .take(self.source_count)
            .filter(|object| {
                !object.deleted && matches!(object.kind, Kind::GraphAxis(_, _, 1 | 0x1_0001))
            })
        {
            let header = object.header_offset.ok_or("Missing X-axis header")?;
            let start = rs
                .iter()
                .position(|record| record.kind == 0x7d0 && record.data.start == header)
                .ok_or("Missing X-axis group")?;
            let format = rs[start + 1..]
                .iter()
                .take_while(|record| record.kind != 0x7d7)
                .find(|record| record.kind == 0x0905 && record.data.len() == 4)
                .ok_or("Missing X-axis format")?;
            out[format.data.start + 2] =
                u8::from(matches!(object.kind, Kind::GraphAxis(_, _, 0x1_0001)));
        }
        for object in self.objects.iter().take(self.source_count) {
            let Kind::PathLiteralPoint(_, value) = object.kind else {
                continue;
            };
            let header = object
                .header_offset
                .ok_or("Missing literal axis point header")?;
            let start = rs
                .iter()
                .position(|record| record.kind == 0x7d0 && record.data.start == header)
                .ok_or("Missing literal axis point group")?;
            let constraint = rs[start + 1..]
                .iter()
                .take_while(|record| record.kind != 0x7d7)
                .find(|record| {
                    record.kind == 0x07d3
                        && record.data.len() == 12
                        && u32_at(&out, record.data.start) == 123
                })
                .ok_or("Missing literal axis point value")?;
            out[constraint.data.start + 4..constraint.data.start + 12]
                .copy_from_slice(&value.to_le_bytes());
        }
        if self.objects.len() == self.source_count {
            let out = self.patch_marked_coordinate_link(out)?;
            let out = self.patch_text_font_overrides(self.patch_visual_styles(
                self.patch_trace_flags(self.patch_text_and_labels(self.compact_deleted(out)?)?)?,
            )?)?;
            return self.patch_last_moved_marker(out);
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
            .find(|r| {
                matches!(r.kind, 0x2328 | 0x232d) && r.start >= last_group_end && r.start < trailer
            })
            .map(|r| r.start)
            .unwrap_or(trailer);
        let mut added = Vec::new();
        let mut picture_resource: Option<Vec<u8>> = None;
        let mut measurement_row = self
            .objects
            .iter()
            .take(self.source_count)
            .filter(|o| !o.deleted && o.kind.is_measurement())
            .count();
        for o in self.objects.iter().skip(self.source_count) {
            if let Kind::MappedPointIteration { ref seeds, ref branches, ref carried, depth, final_only } = o.kind {
                let mut header = *include_bytes!("../../assets/mapped_point_iteration_header.bin");
                header[..2].copy_from_slice(&(if matches!(depth, IterationDepth::Fixed(_)) { 76u16 } else { 89u16 }).to_le_bytes());
                if !o.visible { header[2] |= 1; }
                put_record(&mut added, 0x07d0, &header);
                let mut ids = Vec::new();
                if let IterationDepth::Value(id) = depth { ids.push(id); }
                ids.extend(seeds); ids.extend(branches.iter().flatten()); ids.extend(carried);
                let mut refs = (ids.len() as u32).to_le_bytes().to_vec();
                for id in ids { refs.extend_from_slice(&(id as u32).to_le_bytes()); }
                put_record(&mut added, 0x07d2, &refs);
                let mut definition = *include_bytes!("../../assets/mapped_point_iteration_definition.bin");
                definition[4..6].copy_from_slice(&u16::from(final_only).to_le_bytes());
                definition[6..8].copy_from_slice(&(seeds.len() as u16).to_le_bytes());
                definition[8..12].copy_from_slice(&(branches.len() as u32).to_le_bytes());
                definition[16..20].copy_from_slice(&self.numeric_iteration_depth(depth).unwrap_or(0).to_le_bytes());
                put_record(&mut added, 0x090a, &definition);
                put_record(&mut added, 0x07d7, &[]);
                continue;
            }
            if let Kind::BranchingPolygonIteration {
                seed,
                first,
                second,
                polygon,
                ref segments,
                depth,
            } = o.kind
            {
                let template = include_bytes!("../../assets/branch_polygon_iteration_group.bin");
                let mut header = template[8..36].to_vec();
                header[..2].copy_from_slice(
                    &(if matches!(depth, IterationDepth::Fixed(_)) {
                        76u16
                    } else {
                        89u16
                    })
                    .to_le_bytes(),
                );
                if !o.visible {
                    header[2] |= 1;
                }
                put_record(&mut added, 0x07d0, &header);
                let mut ids = vec![];
                if let IterationDepth::Value(id) = depth {
                    ids.push(id);
                }
                ids.extend([seed, first, second, second, first, polygon]);
                ids.extend(segments);
                let mut refs = (ids.len() as u32).to_le_bytes().to_vec();
                for id in ids {
                    refs.extend_from_slice(&(id as u32).to_le_bytes());
                }
                put_record(&mut added, 0x07d2, &refs);
                let mut definition = template[96..120].to_vec();
                definition[16..20].copy_from_slice(
                    &self
                        .numeric_iteration_depth(depth)
                        .unwrap_or(0)
                        .to_le_bytes(),
                );
                put_record(&mut added, 0x090a, &definition);
                put_record(&mut added, 0x07d7, &[]);
                continue;
            }
            if let Kind::PointParameterIteration(
                seed,
                parameter,
                image,
                calculation,
                label,
                depth,
            ) = o.kind
            {
                let (mut group, shift) = match depth {
                    IterationDepth::Fixed(_) => (
                        include_bytes!("../../assets/point_parameter_iteration_group.bin").to_vec(),
                        0,
                    ),
                    IterationDepth::Value(id) => {
                        let mut group = include_bytes!(
                            "../../assets/point_parameter_iteration_parameter_depth_group.bin"
                        )
                        .to_vec();
                        group[48..52].copy_from_slice(&(id as u32).to_le_bytes());
                        (group, 4)
                    }
                };
                for (offset, id) in [
                    (48, seed),
                    (52, parameter),
                    (56, image),
                    (60, calculation),
                    (64, calculation),
                    (68, image),
                    (72, label),
                ] {
                    group[offset + shift..offset + shift + 4]
                        .copy_from_slice(&(id as u32).to_le_bytes());
                }
                let cached_depth = self.numeric_iteration_depth(depth).unwrap_or(0);
                group[100 + shift..104 + shift].copy_from_slice(&cached_depth.to_le_bytes());
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::IterationValues(iteration, calculation) = o.kind {
                let mut group =
                    *include_bytes!("../../assets/point_parameter_iteration_values_group.bin");
                group[48..52].copy_from_slice(&(iteration as u32).to_le_bytes());
                group[52..56].copy_from_slice(&(calculation as u32).to_le_bytes());
                group[100..102].copy_from_slice(&0u16.to_le_bytes());
                if o.visible {
                    group[10] &= !1;
                } else {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PointNumericLabel(point, value) = o.kind {
                let mut group = *include_bytes!("../../assets/point_numeric_label_group.bin");
                group[48..52].copy_from_slice(&(point as u32).to_le_bytes());
                group[52..56].copy_from_slice(&(value as u32).to_le_bytes());
                group[108..110].copy_from_slice(&0u16.to_le_bytes());
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::RayIteration(seed, image, ray, parameter) = o.kind {
                let mut group = *include_bytes!("../../assets/ray_iteration_group.bin");
                group[48..52].copy_from_slice(&(parameter as u32).to_le_bytes());
                group[52..56].copy_from_slice(&(seed as u32).to_le_bytes());
                group[56..60].copy_from_slice(&(image as u32).to_le_bytes());
                group[60..64].copy_from_slice(&(image as u32).to_le_bytes());
                group[64..68].copy_from_slice(&(ray as u32).to_le_bytes());
                group[92..96].copy_from_slice(
                    &self
                        .iteration_depth_value(parameter)
                        .unwrap_or(0)
                        .to_le_bytes(),
                );
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::SegmentIteration(seed, image, segment, depth)
            | Kind::CircleIteration(seed, image, segment, depth) = o.kind
            {
                let (mut group, shift) = match depth {
                    IterationDepth::Fixed(_) => (
                        include_bytes!("../../assets/segment_iteration_group.bin").to_vec(),
                        0,
                    ),
                    IterationDepth::Value(id) => {
                        let mut group =
                            include_bytes!("../../assets/segment_value_depth_iteration_group.bin")
                                .to_vec();
                        group[48..52].copy_from_slice(&(id as u32).to_le_bytes());
                        (group, 4)
                    }
                };
                for (offset, id) in [(48, seed), (52, image), (56, image), (60, segment)] {
                    group[offset + shift..offset + shift + 4]
                        .copy_from_slice(&(id as u32).to_le_bytes());
                }
                group[88 + shift..92 + shift].copy_from_slice(
                    &self
                        .numeric_iteration_depth(depth)
                        .unwrap_or(0)
                        .to_le_bytes(),
                );
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PointIteration(seed, image, depth) = o.kind {
                let (mut group, shift) = match depth {
                    IterationDepth::Fixed(_) => (
                        include_bytes!("../../assets/point_iteration_group.bin").to_vec(),
                        0,
                    ),
                    IterationDepth::Value(id) => {
                        let mut group =
                            include_bytes!("../../assets/point_value_depth_iteration_group.bin")
                                .to_vec();
                        group[48..52].copy_from_slice(&(id as u32).to_le_bytes());
                        (group, 4)
                    }
                };
                for (offset, id) in [(48, seed), (52, image), (56, image)] {
                    group[offset + shift..offset + shift + 4]
                        .copy_from_slice(&(id as u32).to_le_bytes());
                }
                group[84 + shift..88 + shift].copy_from_slice(
                    &self
                        .numeric_iteration_depth(depth)
                        .unwrap_or(0)
                        .to_le_bytes(),
                );
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::IterationBinding(image, iteration) = o.kind {
                let segment = matches!(
                    self.objects.get(image - 1).map(|object| &object.kind),
                    Some(Kind::Segment(_, _) | Kind::VectorTranslatedSegment(_, _, _))
                );
                let circle = matches!(
                    self.objects.get(image - 1).map(|o| &o.kind),
                    Some(Kind::Circle(_, _))
                );
                let ray = matches!(
                    self.objects.get(image - 1).map(|object| &object.kind),
                    Some(Kind::Ray(_, _))
                );
                let ray_iteration = matches!(
                    self.objects.get(iteration - 1).map(|object| &object.kind),
                    Some(Kind::RayIteration(_, _, _, _))
                );
                let numeric_iteration = matches!(
                    self.objects.get(iteration - 1).map(|o| &o.kind),
                    Some(Kind::PointParameterIteration(_, _, _, _, _, _))
                );
                let branch_polygon = matches!(
                    self.objects.get(iteration - 1).map(|o| &o.kind),
                    Some(Kind::BranchingPolygonIteration { .. } | Kind::MappedPointIteration { .. })
                );
                let segment_value_depth = matches!(
                    self.objects.get(iteration - 1).map(|o| &o.kind),
                    Some(
                        Kind::MappedPointIteration { depth: IterationDepth::Value(_), .. }
                            | Kind::SegmentIteration(_, _, _, IterationDepth::Value(_))
                            | Kind::CircleIteration(_, _, _, IterationDepth::Value(_))
                            | Kind::PointIteration(_, _, IterationDepth::Value(_))
                    )
                );
                let mut group = if segment_value_depth {
                    if segment || circle {
                        *include_bytes!("../../assets/segment_value_depth_segment_binding.bin")
                    } else {
                        *include_bytes!("../../assets/segment_value_depth_point_binding.bin")
                    }
                } else if branch_polygon && crate::document::kind::is_point_kind(&self.objects[image - 1].kind) {
                    *include_bytes!("../../assets/point_iteration_binding_group.bin")
                } else if branch_polygon {
                    if segment {
                        *include_bytes!("../../assets/branch_polygon_iteration_segment_binding.bin")
                    } else {
                        *include_bytes!("../../assets/branch_polygon_iteration_polygon_binding.bin")
                    }
                } else if numeric_iteration
                    && matches!(
                        self.objects.get(image - 1).map(|o| &o.kind),
                        Some(Kind::PointNumericLabel(_, _))
                    )
                {
                    *include_bytes!("../../assets/point_parameter_iteration_label_binding.bin")
                } else if numeric_iteration {
                    *include_bytes!("../../assets/point_parameter_iteration_point_binding.bin")
                } else if ray {
                    *include_bytes!("../../assets/ray_iteration_binding_group.bin")
                } else if ray_iteration {
                    *include_bytes!("../../assets/ray_iteration_point_binding_group.bin")
                } else if segment || circle {
                    *include_bytes!("../../assets/segment_iteration_binding_group.bin")
                } else {
                    *include_bytes!("../../assets/point_iteration_binding_group.bin")
                };
                group[48..52].copy_from_slice(&(image as u32).to_le_bytes());
                group[52..56].copy_from_slice(&(iteration as u32).to_le_bytes());
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::Image { png, transform } = &o.kind {
                let picture = ImageFile::independent_from_png(png, *transform)?;
                let mut group = picture.picture_group().to_vec();
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                picture_resource = Some(picture.resource_record().to_vec());
                continue;
            }
            if let Kind::PlainText { text, rect } = &o.kind {
                let mut group = if let Some(template) = &o.rich_text_override {
                    template.encode_group(text, *rect)?
                } else {
                    text_object::encode_plain_text_group(text, *rect)?
                };
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::Parameter(group) = &o.kind {
                let mut bytes = group.as_bytes().to_vec();
                bytes[10] = (bytes[10] & !1) | u8::from(!o.visible);
                added.extend_from_slice(&bytes);
                continue;
            }
            if let Kind::Calculation(group) = &o.kind {
                let mut bytes = group.as_bytes().to_vec();
                bytes[10] = (bytes[10] & !1) | u8::from(!o.visible);
                added.extend_from_slice(&bytes);
                continue;
            }
            if let Kind::LengthPairCalculation(first, second, group) = &o.kind {
                let mut bytes = group
                    .remap_sources(*first as u32, *second as u32)?
                    .as_bytes()
                    .to_vec();
                bytes[10] = (bytes[10] & !1) | u8::from(!o.visible);
                added.extend_from_slice(&bytes);
                continue;
            }
            if let Kind::MeasuredCalculation(source, group) = &o.kind {
                let mut bytes = group
                    .remap_source(u32::try_from(*source).map_err(|_| "Invalid measurement ID")?)?
                    .as_bytes()
                    .to_vec();
                bytes[10] = (bytes[10] & !1) | u8::from(!o.visible);
                added.extend_from_slice(&bytes);
                continue;
            }
            if let Kind::ParameterCalculation(source, group) = &o.kind {
                let ids = calculation_source_ids(*source, group)
                    .into_iter()
                    .map(|id| id as u32)
                    .collect::<Vec<_>>();
                let mut bytes = group.remap_sources(&ids)?.as_bytes().to_vec();
                bytes[10] = (bytes[10] & !1) | u8::from(!o.visible);
                added.extend_from_slice(&bytes);
                continue;
            }
            if let Kind::ValueTable(_, group) | Kind::ValueTablePair(_, _, group) = &o.kind {
                let mut bytes = group.as_bytes().to_vec();
                bytes[10] = (bytes[10] & !1) | u8::from(!o.visible);
                added.extend_from_slice(&bytes);
                continue;
            }
            if let Kind::HideShowButton {
                targets,
                action,
                label,
                anchor,
            } = &o.kind
            {
                let ids = targets
                    .iter()
                    .map(|&target| target as u32)
                    .collect::<Vec<_>>();
                let mut group =
                    HideShowButtonGroup::from_original_template_targets(&ids, label, *anchor)?;
                group.set_action(*action);
                group.set_hidden(!o.visible);
                added.extend_from_slice(&group.encode());
                continue;
            }
            if let Kind::AnimationButton {
                targets,
                label,
                anchor,
                settings,
            } = &o.kind
            {
                let ids = targets
                    .iter()
                    .map(|&target| target as u32)
                    .collect::<Vec<_>>();
                let mut group =
                    AnimationButtonGroup::from_original_template_targets(&ids, label, *anchor)?;
                group.set_hidden(!o.visible);
                group.set_settings_list(settings)?;
                added.extend_from_slice(&group.encode());
                continue;
            }
            if let Kind::MovementButton {
                source,
                destination,
                speed,
                follow,
                label,
                anchor,
            } = &o.kind
            {
                let mut group = MovementButtonGroup::from_original_template(
                    *source as u32,
                    *destination as u32,
                    label,
                    *anchor,
                )?;
                group.set_speed(*speed);
                group.set_follows_destination(*follow);
                group.set_hidden(!o.visible);
                added.extend_from_slice(&group.encode());
                continue;
            }
            if let Kind::ScrollButton {
                target,
                placement,
                label,
                anchor,
            } = &o.kind
            {
                let mut group =
                    ScrollButtonGroup::from_original_template(*target as u32, label, *anchor)?;
                group.set_placement(*placement);
                group.set_hidden(!o.visible);
                added.extend_from_slice(&group.encode());
                continue;
            }
            if let Kind::PresentationButton {
                targets,
                mode,
                label,
                anchor,
            } = &o.kind
            {
                let ids = targets
                    .iter()
                    .map(|&target| target as u32)
                    .collect::<Vec<_>>();
                let mut group =
                    PresentationButtonGroup::from_original_template_targets(&ids, label, *anchor)?;
                group.set_mode(*mode);
                group.set_hidden(!o.visible);
                added.extend_from_slice(&group.encode());
                continue;
            }
            if let Kind::LinkUrlButton { url, label, anchor } = &o.kind {
                let mut group = LinkUrlButtonGroup::from_original_template(url, label, *anchor)?;
                let index = self.objects[..o.id - 1]
                    .iter()
                    .filter(|object| !object.deleted)
                    .count();
                group.set_page_object_index(index)?;
                group.set_hidden(!o.visible);
                added.extend_from_slice(&group.encode());
                continue;
            }
            if let Some(group) = self.graph_template_group(o)? {
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::FunctionDefinition(expression) = o.kind {
                let mut group = function_records::encode_definition_group(expression)?;
                if let Some(anchor) = o.function_anchor {
                    function_records::set_group_expression_anchor(&mut group, anchor)?;
                }
                if let Some(name) = &o.function_name {
                    function_records::set_group_definition_name(&mut group, name)?;
                }
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::DerivativeFunction(source, expression) = o.kind {
                let mut group = function_records::encode_derivative_group(source, expression)?;
                if let Some(anchor) = o.function_anchor {
                    function_records::set_group_expression_anchor(&mut group, anchor)?;
                }
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::FunctionPlot(definition, frame, min, max, samples) = o.kind {
                let expression = self
                    .function_expression(definition)
                    .ok_or("Invalid function definition")?;
                let mut group = function_records::encode_plot_group_with_range(
                    expression, definition, frame, min, max, samples,
                )?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::ParametricPlot(x, y, frame, min, max, samples, mode) = o.kind {
                let mut group = function_records::encode_parametric_plot_group(
                    x,
                    y,
                    frame,
                    min,
                    max,
                    samples,
                    mode == PlotPointsMode::Polar,
                )?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::NumericCoordinatePoint(x, y, frame) = o.kind {
                // Native PlotXY has only its point header and three references.
                let mut point_header = [
                    69, 0, 0, 0, 0, 0, 1, 0, 56, 0, 33, 2, 255, 0, 0, 0,
                    255, 255, 81, 92, 193, 189, 240, 255, 0, 0, 0, 0,
                ];
                if !o.visible { point_header[2] |= 1; }
                put_record(&mut added, 0x07d0, &point_header);
                let mut refs = 3_u32.to_le_bytes().to_vec();
                for id in [x, y, frame] {
                    refs.extend_from_slice(&(id as u32).to_le_bytes());
                }
                put_record(&mut added, 0x07d2, &refs);
                if let Some(label) = &o.label {
                    added.extend_from_slice(&crate::text_records::encode_point_label_records(label));
                }
                put_record(&mut added, 0x07d7, &[]);
                continue;
            }
            if let Kind::PlottedPoint(source, frame, mode, stored) = o.kind {
                let mut group =
                    plot_points_records::encode_group(mode, [source as u32, frame as u32], stored)?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PathValuePoint(source, axis) = o.kind {
                let mut group = plot_axis_records::encode_group([source as u32, axis as u32])?;
                if !o.visible {
                    group[10] |= 1;
                }
                if let Some(label) = &o.label {
                    let labels = crate::text_records::encode_point_label_records(label);
                    group.splice(56..56, labels);
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PathLiteralPoint(axis, value) = o.kind {
                let mut group = plot_axis_records::encode_literal_group(axis as u32, value)?;
                if !o.visible {
                    group[10] |= 1;
                }
                if let Some(label) = &o.label {
                    let labels = crate::text_records::encode_point_label_records(label);
                    group.splice(72..72, labels);
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::ParameterLocus(ref refs, min, max, samples) = o.kind {
                let mut h = [97, 0, 0, 0, 0, 0, 6, 0, 42, 0, 34, 1, 128, 0, 0, 0,
                    255, 255, 81, 92, 193, 189, 240, 255, 0, 0, 0, 0];
                if !o.visible { h[2] |= 1; }
                put_record(&mut added, 0x07d0, &h);
                let mut references = (refs.len() as u32).to_le_bytes().to_vec();
                for &id in refs { references.extend_from_slice(&(id as u32).to_le_bytes()); }
                put_record(&mut added, 0x07d2, &references);
                let mut range = min.to_le_bytes().to_vec();
                range.extend_from_slice(&max.to_le_bytes());
                range.extend_from_slice(&samples.to_le_bytes());
                range.extend_from_slice(&[4, 0, 3, 0, 0, 0, 0, 0, 36, 98, 92, 1, 1, 0, 0, 0, 0, 0, 0, 0]);
                let driver = *refs.get(1).ok_or("Parameter locus has no driver")?;
                let Kind::Parameter(parameter) = &self.objects[driver - 1].kind else {
                    return Err("Parameter locus has no parameter driver".into());
                };
                let tag = parameter.locus_unit_tag().ok_or("Unsupported parameter locus units")?;
                range[32..36].copy_from_slice(&tag.to_le_bytes());
                put_record(&mut added, 0x0902, &range);
                put_record(&mut added, 0x07d7, &[]);
                continue;
            }
            if let Kind::Locus(ref refs, samples) = o.kind {
                let mut h = [
                    35, 0, 0, 0, 0, 0, 6, 0, 42, 0, 0x22, 1, 0x80, 0, 0, 0, 0xff, 0xff, 0x39, 0x20,
                    0xc1, 0xbd, 0xf0, 0xff, 0, 0, 0, 0,
                ];
                let target = *refs.first().ok_or("Locus has no traced object")?;
                let target_kind = &self
                    .objects
                    .get(target.saturating_sub(1))
                    .ok_or("Missing locus target")?
                    .kind;
                let straight_target = is_locus_straight_kind(target_kind);
                let curved_target = target_kind.is_circle() || is_arc_kind(target_kind);
                let polygon_target = is_locus_polygon_kind(target_kind);
                if polygon_target {
                    h[8] = 43;
                    h[10] = 0x21;
                    h[12..16].copy_from_slice(&[0xff, 0xb2, 0x66, 0]);
                    h[17] = 0x7f;
                    if matches!(target_kind, Kind::PolarTranslatedPolygon(_, _, _, _)) {
                        h[18..20].copy_from_slice(&[0xf0, 0x3f]);
                    } else if matches!(
                        target_kind,
                        Kind::FixedRectangularTranslatedPolygon(_, _, _, _, _)
                    ) {
                        h[18..20].copy_from_slice(&[0, 0x40]);
                    }
                    h[20..24].copy_from_slice(
                        &if matches!(
                            target_kind,
                            Kind::VectorTranslatedPolygon(_, _, _)
                                | Kind::PolarTranslatedPolygon(_, _, _, _)
                                | Kind::FixedRectangularTranslatedPolygon(_, _, _, _, _)
                        ) {
                            2u32
                        } else {
                            1u32
                        }
                        .to_le_bytes(),
                    );
                } else if straight_target || curved_target {
                    h[8] = 43;
                    h[12..16].copy_from_slice(if curved_target {
                        &[0x66, 0xb2, 0x66, 0]
                    } else {
                        &[0x66, 0x66, 0xb2, 0]
                    });
                }
                if !o.visible {
                    h[2] = 1;
                }
                put_record(&mut added, 0x07d0, &h);
                let mut references = Vec::with_capacity(4 + refs.len() * 4);
                references.extend_from_slice(&(refs.len() as u32).to_le_bytes());
                for &id in refs {
                    references.extend_from_slice(&(id as u32).to_le_bytes());
                }
                put_record(&mut added, 0x07d2, &references);
                let driver = *refs.get(2).ok_or("Locus has no moving point")?;
                let circle_driver = matches!(
                    self.objects
                        .get(driver.saturating_sub(1))
                        .map(|object| &object.kind),
                    Some(Kind::PointOnCircle(_, _, _))
                );
                let mut range = Vec::with_capacity(40);
                range.extend_from_slice(&0f64.to_le_bytes());
                range.extend_from_slice(
                    &if circle_driver {
                        std::f64::consts::TAU
                    } else {
                        1.0
                    }
                    .to_le_bytes(),
                );
                range.extend_from_slice(&samples.to_le_bytes());
                range.extend_from_slice(&[
                    4,
                    0,
                    if circle_driver { 0 } else { 1 },
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    if straight_target || curved_target || polygon_target {
                        0
                    } else {
                        1
                    },
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ]);
                put_record(&mut added, 0x0902, &range);
                put_record(&mut added, 0x07d7, &[]);
                continue;
            }
            if let Kind::SlopeMeasurement(line, frame) = o.kind {
                let mut group = graph::slope_measurement_group(line, frame)?;
                Self::place_measurement(&mut group, 64, &mut measurement_row)?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::CoordinateMeasurement {
                kind,
                first,
                second,
                frame,
            } = o.kind
            {
                let mut refs = vec![u32::try_from(first).map_err(|_| "Too many objects")?];
                if let Some(second) = second {
                    refs.push(u32::try_from(second).map_err(|_| "Too many objects")?);
                }
                refs.push(u32::try_from(frame).map_err(|_| "Too many objects")?);
                let mut group = coordinate_measure_records::encode_group(kind, &refs)?;
                if kind == CoordinateMeasureKind::Coordinates {
                    let name = self
                        .coordinate_point_name(first)
                        .ok_or("Missing coordinate point")?;
                    coordinate_measure_records::set_coordinates_name(&mut group, &name)?;
                }
                let format_offset = if second.is_some() { 68 } else { 64 };
                Self::place_measurement(&mut group, format_offset, &mut measurement_row)?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::ValueEquation {
                kind,
                first,
                second,
            } = o.kind
            {
                let references = [
                    u32::try_from(first).map_err(|_| "Too many objects")?,
                    u32::try_from(second).map_err(|_| "Too many objects")?,
                ];
                let mut group = if kind == ValueEquationKind::PointValue
                    && self
                        .objects
                        .get(second.saturating_sub(1))
                        .is_some_and(|object| {
                            matches!(
                                straight_kind_mode(&object.kind),
                                Some(LineMode::Ray | LineMode::Line)
                            )
                        }) {
                    value_equation_records::encode_straight_point_value_group(references)?
                } else if kind == ValueEquationKind::PointValue
                    && matches!(
                        self.objects
                            .get(second.saturating_sub(1))
                            .map(|object| &object.kind),
                        Some(Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _))
                    )
                {
                    let Some(crate::value_equation_records::EquationValue::PointValue(parameter)) =
                        self.value_equation_value(o.id)
                    else {
                        return Err("Value of Point requires a defined arc parameter".into());
                    };
                    value_equation_records::encode_arc_point_value_group(
                        references,
                        matches!(self.objects[second - 1].kind, Kind::ThreePointArc(_, _, _)),
                        parameter,
                    )?
                } else {
                    value_equation_records::encode_group(kind, references)?
                };
                if kind == ValueEquationKind::PointValue {
                    Self::place_measurement(&mut group, 64, &mut measurement_row)?;
                }
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::SegmentLength(segment) = o.kind {
                let mut group = *include_bytes!("../../assets/length_measurement_group.bin");
                group[48..52].copy_from_slice(&(segment as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PointDistance(a, b) | Kind::PointLineDistance(a, b) = o.kind {
                let mut group = if matches!(o.kind, Kind::PointLineDistance(_, _)) {
                    *include_bytes!("../../assets/point_line_distance_measurement_group.bin")
                } else {
                    *include_bytes!("../../assets/distance_measurement_group.bin")
                };
                group[48..52].copy_from_slice(&(a as u32).to_le_bytes());
                group[52..56].copy_from_slice(&(b as u32).to_le_bytes());
                Self::place_measurement(&mut group, 64, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PointRatio(a, b, c) = o.kind {
                let mut group = *include_bytes!("../../assets/ratio_measurement_group.bin");
                group[48..52].copy_from_slice(&(a as u32).to_le_bytes());
                group[52..56].copy_from_slice(&(b as u32).to_le_bytes());
                group[56..60].copy_from_slice(&(c as u32).to_le_bytes());
                Self::place_measurement(&mut group, 68, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PolygonPerimeter(polygon) = o.kind {
                let mut group = *include_bytes!("../../assets/perimeter_measurement_group.bin");
                group[48..52].copy_from_slice(&(polygon as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::CircleCircumference(circle) = o.kind {
                let mut group = *include_bytes!("../../assets/circumference_measurement_group.bin");
                group[48..52].copy_from_slice(&(circle as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::CircleRadius(circle) = o.kind {
                let mut group = *include_bytes!("../../assets/radius_measurement_group.bin");
                group[48..52].copy_from_slice(&(circle as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::CircleArea(circle) = o.kind {
                let mut group = *include_bytes!("../../assets/area_measurement_group.bin");
                group[48..52].copy_from_slice(&(circle as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PolygonArea(polygon) = o.kind {
                let mut group = *include_bytes!("../../assets/area_measurement_group.bin");
                group[48..52].copy_from_slice(&(polygon as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::ArcAngle(arc) = o.kind {
                let mut group = *include_bytes!("../../assets/arc_angle_measurement_group.bin");
                group[48..52].copy_from_slice(&(arc as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::ArcLength(arc) = o.kind {
                let mut group = *include_bytes!("../../assets/arc_length_measurement_group.bin");
                group[48..52].copy_from_slice(&(arc as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::AngleMeasurement(a, b, c) = o.kind {
                let mut group = *include_bytes!("../../assets/angle_measurement_group.bin");
                group[48..52].copy_from_slice(&(a as u32).to_le_bytes());
                group[52..56].copy_from_slice(&(b as u32).to_le_bytes());
                group[56..60].copy_from_slice(&(c as u32).to_le_bytes());
                Self::place_measurement(&mut group, 68, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            let (code, refs): (u16, Vec<usize>) = match o.kind {
                Kind::Free(_) => (0, vec![]),
                Kind::PointOnSegment(host, _)
                | Kind::PointOnRay(host, _)
                | Kind::PointOnLine(host, _)
                | Kind::PointOnCircle(host, _, _)
                | Kind::PointOnArc(host, _)
                | Kind::PointOnPolygon(host, _, _) => (15, vec![host]),
                Kind::PolarTranslatedPoint(origin, _, _, _) => (21, vec![origin]),
                Kind::PolarTranslatedSegment(source, _, _, _) => (21, vec![source]),
                Kind::PolarTranslatedCircle(source, _, _, _) => (21, vec![source]),
                Kind::PolarTranslatedPolygon(source, _, _, _) => (21, vec![source]),
                Kind::FixedRectangularTranslatedPoint(source, _, _, _, _) => (17, vec![source]),
                Kind::FixedRectangularTranslatedSegment(source, _, _, _, _) => (17, vec![source]),
                Kind::FixedRectangularTranslatedCircle(source, _, _, _, _) => (17, vec![source]),
                Kind::FixedRectangularTranslatedPolygon(source, _, _, _, _) => (17, vec![source]),
                Kind::MarkedDistanceTranslatedPoint(source, distance, _) => {
                    (24, vec![source, distance])
                }
                Kind::MarkedDistanceTranslatedSegment(source, distance, _)
                | Kind::MarkedDistanceTranslatedCircle(source, distance, _)
                | Kind::MarkedDistanceTranslatedPolygon(source, distance, _) => {
                    (24, vec![source, distance])
                }
                Kind::MixedRectangularTranslatedPoint(source, distance, _, fixed_horizontal)
                | Kind::MixedRectangularTranslatedSegment(source, distance, _, fixed_horizontal)
                | Kind::MixedRectangularTranslatedCircle(source, distance, _, fixed_horizontal) => {
                    (
                        if fixed_horizontal { 18 } else { 19 },
                        vec![source, distance],
                    )
                }
                Kind::MixedRectangularTranslatedPolygon(source, distance, _, fixed_horizontal) => (
                    if fixed_horizontal { 18 } else { 19 },
                    vec![source, distance],
                ),
                Kind::RectangularMarkedTranslatedPoint(source, horizontal, vertical)
                | Kind::RectangularMarkedTranslatedSegment(source, horizontal, vertical)
                | Kind::RectangularMarkedTranslatedCircle(source, horizontal, vertical) => {
                    (20, vec![source, horizontal, vertical])
                }
                Kind::RectangularMarkedTranslatedPolygon(source, horizontal, vertical) => {
                    (20, vec![source, horizontal, vertical])
                }
                Kind::VectorTranslatedPoint(source, start, end) => (16, vec![source, start, end]),
                Kind::VectorTranslatedSegment(source, start, end)
                | Kind::VectorTranslatedCircle(source, start, end)
                | Kind::VectorTranslatedPolygon(source, start, end) => {
                    (16, vec![source, start, end])
                }
                Kind::ReflectedPoint(source, mirror) => (34, vec![source, mirror]),
                Kind::ReflectedCircle(source, mirror)
                | Kind::ReflectedSegment(source, mirror)
                | Kind::ReflectedPolygon(source, mirror) => (34, vec![source, mirror]),
                Kind::ScaledPoint(source, center, _)
                | Kind::ScaledCircle(source, center, _)
                | Kind::ScaledSegment(source, center, _)
                | Kind::ScaledRay(source, center, _)
                | Kind::ScaledLine(source, center, _)
                | Kind::ScaledPolygon(source, center, _) => (30, vec![source, center]),
                Kind::CalculatedRatioScaledPoint(source, center, ratio)
                | Kind::CalculatedRatioScaledSegment(source, center, ratio)
                | Kind::CalculatedRatioScaledCircle(source, center, ratio)
                | Kind::CalculatedRatioScaledPolygon(source, center, ratio) => {
                    (33, vec![source, center, ratio])
                }
                Kind::CalculatedAngleRotatedPoint(source, center, angle)
                | Kind::CalculatedAngleRotatedSegment(source, center, angle)
                | Kind::CalculatedAngleRotatedCircle(source, center, angle)
                | Kind::CalculatedAngleRotatedPolygon(source, center, angle) => {
                    (29, vec![source, center, angle])
                }
                Kind::RotatedPoint(source, center, _)
                | Kind::RotatedSegment(source, center, _)
                | Kind::RotatedRay(source, center, _)
                | Kind::RotatedLine(source, center, _)
                | Kind::RotatedCircle(source, center, _)
                | Kind::RotatedPolygon(source, center, _) => (27, vec![source, center]),
                Kind::AngleRotatedPoint(source, center, first, vertex, last)
                | Kind::AngleRotatedSegment(source, center, first, vertex, last)
                | Kind::AngleRotatedCircle(source, center, first, vertex, last)
                | Kind::AngleRotatedPolygon(source, center, first, vertex, last) => {
                    (28, vec![source, center, first, vertex, last])
                }
                Kind::RatioScaledPoint(source, center, origin, base, target) => {
                    (32, vec![source, center, origin, base, target])
                }
                Kind::RatioScaledCircle(source, center, origin, base, target)
                | Kind::RatioScaledSegment(source, center, origin, base, target)
                | Kind::RatioScaledPolygon(source, center, origin, base, target) => {
                    (32, vec![source, center, origin, base, target])
                }
                Kind::Midpoint(_, _) => (
                    1,
                    vec![
                        o.midpoint_host
                            .ok_or("Midpoint requires an existing segment")?,
                    ],
                ),
                Kind::Segment(a, b) => (2, vec![a, b]),
                Kind::Circle(a, b) => (3, vec![a, b]),
                Kind::CircleBySegment(a, b) => (4, vec![a, b]),
                Kind::Polygon(ref vertices) => (8, vertices.clone()),
                Kind::Perpendicular(a, b) => (5, vec![a, b]),
                Kind::Parallel(a, b) => (6, vec![a, b]),
                Kind::Bisector(a, b, c) => (7, vec![a, b, c]),
                Kind::Intersection(a, b) => (9, vec![a, b]),
                Kind::LineCircleIntersection(a, b, variant) => {
                    (if variant == 0 { 12 } else { 11 }, vec![a, b])
                }
                Kind::CircleCircleIntersection(a, b, variant) => {
                    (if variant == 0 { 14 } else { 13 }, vec![a, b])
                }
                Kind::Line(a, b) => (63, vec![a, b]),
                Kind::Ray(a, b) => (64, vec![a, b]),
                Kind::ThreePointArc(a, b, c) => (81, vec![a, b, c]),
                Kind::ArcOnCircle(a, b, c) => (79, vec![a, b, c]),
                _ => return Err("Unsupported new object".into()),
            };
            let mut h = header(code);
            if let Kind::Polygon(ref vertices) = o.kind {
                h[8..10].copy_from_slice(
                    &polygon_subtype(vertices.len())
                        .ok_or("Polygon has too many vertices")?
                        .to_le_bytes(),
                );
            }
            if matches!(
                o.kind,
                Kind::RotatedRay(..) | Kind::RotatedLine(..) | Kind::ScaledRay(..) | Kind::ScaledLine(..)
            ) {
                h = header(if matches!(o.kind, Kind::RotatedRay(..) | Kind::ScaledRay(..)) {
                    64
                } else {
                    63
                });
                h[0] = code as u8;
                h[18..20].copy_from_slice(&[0x51, 0x5c]);
            }
            if matches!(
                o.kind,
                Kind::ReflectedCircle(_, _) | Kind::ScaledCircle(_, _, _)
            ) {
                h = header(3);
                h[0] = code as u8;
                h[18] = 0;
                h[19] = 0;
            }
            if matches!(
                o.kind,
                Kind::PolarTranslatedSegment(_, _, _, _)
                    | Kind::FixedRectangularTranslatedSegment(_, _, _, _, _)
                    | Kind::MixedRectangularTranslatedSegment(_, _, _, _)
                    | Kind::RectangularMarkedTranslatedSegment(_, _, _)
                    | Kind::VectorTranslatedSegment(_, _, _)
                    | Kind::MarkedDistanceTranslatedSegment(_, _, _)
                    | Kind::AngleRotatedSegment(_, _, _, _, _)
                    | Kind::RotatedSegment(_, _, _)
                    | Kind::CalculatedAngleRotatedSegment(_, _, _)
                    | Kind::CalculatedRatioScaledSegment(_, _, _)
                    | Kind::ScaledSegment(_, _, _)
                    | Kind::RatioScaledSegment(_, _, _, _, _)
                    | Kind::ReflectedSegment(_, _)
            ) {
                h = header(2);
                h[0] = code as u8;
                if matches!(
                    o.kind,
                    Kind::VectorTranslatedSegment(_, _, _)
                        | Kind::MarkedDistanceTranslatedSegment(_, _, _)
                        | Kind::AngleRotatedSegment(_, _, _, _, _)
                        | Kind::RotatedSegment(_, _, _)
                        | Kind::CalculatedAngleRotatedSegment(_, _, _)
                        | Kind::CalculatedRatioScaledSegment(_, _, _)
                        | Kind::ScaledSegment(_, _, _)
                        | Kind::RatioScaledSegment(_, _, _, _, _)
                ) {
                    h[18] = 0x51;
                    h[19] = 0x5c;
                }
            }
            if matches!(
                o.kind,
                Kind::PolarTranslatedCircle(_, _, _, _)
                    | Kind::FixedRectangularTranslatedCircle(_, _, _, _, _)
                    | Kind::MixedRectangularTranslatedCircle(_, _, _, _)
                    | Kind::RectangularMarkedTranslatedCircle(_, _, _)
                    | Kind::VectorTranslatedCircle(_, _, _)
                    | Kind::MarkedDistanceTranslatedCircle(_, _, _)
                    | Kind::AngleRotatedCircle(_, _, _, _, _)
                    | Kind::RotatedCircle(_, _, _)
                    | Kind::CalculatedAngleRotatedCircle(_, _, _)
                    | Kind::CalculatedRatioScaledCircle(_, _, _)
            ) {
                h = header(3);
                h[0] = code as u8;
                if matches!(
                    o.kind,
                    Kind::VectorTranslatedCircle(_, _, _)
                        | Kind::MarkedDistanceTranslatedCircle(_, _, _)
                        | Kind::AngleRotatedCircle(_, _, _, _, _)
                        | Kind::RotatedCircle(_, _, _)
                        | Kind::CalculatedAngleRotatedCircle(_, _, _)
                        | Kind::CalculatedRatioScaledCircle(_, _, _)
                ) {
                    h[18] = 0x51;
                    h[19] = 0x5c;
                }
            }
            if matches!(o.kind, Kind::RatioScaledCircle(_, _, _, _, _)) {
                h = header(3);
                h[0] = code as u8;
                h[18] = 0x51;
                h[19] = 0x5c;
            }
            if matches!(
                o.kind,
                Kind::ReflectedPolygon(_, _)
                    | Kind::VectorTranslatedPolygon(_, _, _)
                    | Kind::PolarTranslatedPolygon(_, _, _, _)
                    | Kind::FixedRectangularTranslatedPolygon(_, _, _, _, _)
                    | Kind::MarkedDistanceTranslatedPolygon(_, _, _)
                    | Kind::RectangularMarkedTranslatedPolygon(_, _, _)
                    | Kind::MixedRectangularTranslatedPolygon(_, _, _, _)
                    | Kind::RotatedPolygon(_, _, _)
                    | Kind::CalculatedAngleRotatedPolygon(_, _, _)
                    | Kind::CalculatedRatioScaledPolygon(_, _, _)
                    | Kind::AngleRotatedPolygon(_, _, _, _, _)
                    | Kind::ScaledPolygon(_, _, _)
                    | Kind::RatioScaledPolygon(_, _, _, _, _)
            ) {
                h = header(8);
                h[0] = code as u8;
                if let Kind::ReflectedPolygon(source, _)
                | Kind::VectorTranslatedPolygon(source, _, _)
                | Kind::PolarTranslatedPolygon(source, _, _, _)
                | Kind::FixedRectangularTranslatedPolygon(source, _, _, _, _)
                | Kind::MarkedDistanceTranslatedPolygon(source, _, _)
                | Kind::RectangularMarkedTranslatedPolygon(source, _, _)
                | Kind::MixedRectangularTranslatedPolygon(source, _, _, _)
                | Kind::RotatedPolygon(source, _, _)
                | Kind::CalculatedAngleRotatedPolygon(source, _, _)
                | Kind::CalculatedRatioScaledPolygon(source, _, _)
                | Kind::AngleRotatedPolygon(source, _, _, _, _)
                | Kind::ScaledPolygon(source, _, _)
                | Kind::RatioScaledPolygon(source, _, _, _, _) = o.kind
                {
                    let vertex_count = polygon_vertex_count_for_encoding(&self.objects, source)
                        .ok_or("Interior transform requires a supported polygon source")?;
                    h[8..10].copy_from_slice(
                        &polygon_subtype(vertex_count)
                            .ok_or("Polygon has too many vertices")?
                            .to_le_bytes(),
                    );
                }
                h[20..24].copy_from_slice(&1u32.to_le_bytes());
                if !matches!(
                    o.kind,
                    Kind::ReflectedPolygon(_, _)
                        | Kind::PolarTranslatedPolygon(_, _, _, _)
                        | Kind::FixedRectangularTranslatedPolygon(_, _, _, _, _)
                ) {
                    h[18] = 0x51;
                    h[19] = 0x5c;
                }
            }
            if matches!(o.kind, Kind::VectorTranslatedPolygon(_, _, _))
                && self.objects.iter().any(|candidate| {
                    !candidate.deleted
                        && matches!(&candidate.kind, Kind::Locus(refs, _) if refs.first() == Some(&o.id))
                })
            {
                h[18..20].copy_from_slice(&[0x39, 0x20]);
            }
            if !o.visible {
                h[2] = 1;
            }
            if o.kind.is_polygon() {
                h[20..24].copy_from_slice(&o.interior_order
                    .ok_or("Missing polygon drawing order")?.to_le_bytes());
            }
            put_record(&mut added, 0x7d0, &h);
            if !matches!(o.kind, Kind::PointOnPolygon(..)) && let Some(label) = &o.label {
                added.extend_from_slice(&crate::text_records::encode_point_label_records(label));
            }
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
                match o.kind {
                    Kind::PointOnSegment(_, t) => {
                        let mut payload = vec![15, 0, 14, 0];
                        payload.extend_from_slice(&t.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::PointOnRay(_, t) | Kind::PointOnLine(_, t) => {
                        let code = if matches!(o.kind, Kind::PointOnRay(_, _)) {
                            13
                        } else {
                            12
                        };
                        let mut payload = vec![15, 0, code, 0];
                        payload.extend_from_slice(&t.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::PointOnArc(_, t) => {
                        let mut payload = vec![15, 0, 67, 0];
                        payload.extend_from_slice(&t.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::PointOnPolygon(host, edge, t) => {
                        let count = polygon_vertex_count_for_encoding(&self.objects, host)
                            .ok_or("Missing polygon boundary host")?;
                        let subtype =
                            polygon_subtype(count).ok_or("Too many polygon vertices")?;
                        let mut payload = 15u16.to_le_bytes().to_vec();
                        payload.extend_from_slice(&subtype.to_le_bytes());
                        payload.extend_from_slice(&t.to_le_bytes());
                        payload.extend_from_slice(&(edge as u32).to_le_bytes());
                        payload.extend_from_slice(&[255, 255, 0, 0]);
                        put_record(&mut added, 0x7d3, &payload);
                        if let Some(label) = &o.label {
                            added.extend_from_slice(&crate::text_records::encode_point_label_records(label));
                        }
                    }
                    Kind::PointOnCircle(_, x, y) => {
                        let mut payload = vec![15, 0, 28, 0];
                        payload.extend_from_slice(&x.to_le_bytes());
                        payload.extend_from_slice(&y.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::PolarTranslatedPoint(_, angle, units, distance)
                    | Kind::PolarTranslatedSegment(_, angle, units, distance)
                    | Kind::PolarTranslatedCircle(_, angle, units, distance)
                    | Kind::PolarTranslatedPolygon(_, angle, units, distance) => {
                        let mut payload =
                            *include_bytes!("../../assets/polar_translation_payload.bin");
                        if angle != 90. {
                            payload[4..12]
                                .copy_from_slice(&(-angle.to_radians().sin()).to_le_bytes());
                            payload[12..20]
                                .copy_from_slice(&angle.to_radians().cos().to_le_bytes());
                        }
                        payload[20..28].copy_from_slice(&angle.to_le_bytes());
                        payload[32..40].copy_from_slice(&units.to_le_bytes());
                        payload[40..48].copy_from_slice(&distance.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::FixedRectangularTranslatedPoint(
                        _,
                        x_units,
                        x_distance,
                        y_units,
                        y_distance,
                    )
                    | Kind::FixedRectangularTranslatedSegment(
                        _,
                        x_units,
                        x_distance,
                        y_units,
                        y_distance,
                    )
                    | Kind::FixedRectangularTranslatedCircle(
                        _,
                        x_units,
                        x_distance,
                        y_units,
                        y_distance,
                    )
                    | Kind::FixedRectangularTranslatedPolygon(
                        _,
                        x_units,
                        x_distance,
                        y_units,
                        y_distance,
                    ) => {
                        let mut payload = *include_bytes!(
                            "../../assets/rectangular_fixed_translation_payload.bin"
                        );
                        payload[4..12].copy_from_slice(&(x_units * x_distance).to_le_bytes());
                        payload[12..20].copy_from_slice(&x_distance.to_le_bytes());
                        payload[24..32].copy_from_slice(&(y_units * y_distance).to_le_bytes());
                        payload[32..40].copy_from_slice(&y_distance.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::MarkedDistanceTranslatedPoint(_, _, angle)
                    | Kind::MarkedDistanceTranslatedSegment(_, _, angle)
                    | Kind::MarkedDistanceTranslatedCircle(_, _, angle)
                    | Kind::MarkedDistanceTranslatedPolygon(_, _, angle) => {
                        let mut payload =
                            *include_bytes!("../../assets/marked_distance_translation_payload.bin");
                        if angle != 90. {
                            payload[4..12]
                                .copy_from_slice(&(-angle.to_radians().sin()).to_le_bytes());
                            payload[12..20]
                                .copy_from_slice(&angle.to_radians().cos().to_le_bytes());
                        }
                        payload[20..28].copy_from_slice(&angle.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::MixedRectangularTranslatedPoint(_, _, fixed_cm, fixed_horizontal)
                    | Kind::MixedRectangularTranslatedSegment(_, _, fixed_cm, fixed_horizontal)
                    | Kind::MixedRectangularTranslatedCircle(_, _, fixed_cm, fixed_horizontal)
                    | Kind::MixedRectangularTranslatedPolygon(_, _, fixed_cm, fixed_horizontal) => {
                        let mut payload = *include_bytes!(
                            "../../assets/mixed_rectangular_translation_payload.bin"
                        );
                        payload[0] = if fixed_horizontal { 18 } else { 19 };
                        payload[4..12]
                            .copy_from_slice(&(fixed_cm * 37.79527559055118).to_le_bytes());
                        payload[12..20].copy_from_slice(&fixed_cm.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::ScaledPoint(_, _, factor)
                    | Kind::ScaledCircle(_, _, factor)
                    | Kind::ScaledSegment(_, _, factor)
                    | Kind::ScaledRay(_, _, factor)
                    | Kind::ScaledLine(_, _, factor)
                    | Kind::ScaledPolygon(_, _, factor) => {
                        let mut payload = vec![30, 0, 0, 0];
                        payload.extend_from_slice(&factor.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::RotatedPoint(_, _, angle)
                    | Kind::RotatedSegment(_, _, angle)
                    | Kind::RotatedRay(_, _, angle)
                    | Kind::RotatedLine(_, _, angle)
                    | Kind::RotatedCircle(_, _, angle)
                    | Kind::RotatedPolygon(_, _, angle) => {
                        let mut payload = *include_bytes!("../../assets/rotation_payload.bin");
                        payload[4..12].copy_from_slice(&(-angle.to_radians().sin()).to_le_bytes());
                        payload[12..20].copy_from_slice(&angle.to_radians().cos().to_le_bytes());
                        payload[20..28].copy_from_slice(&angle.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    _ => {}
                }
            }
            put_record(&mut added, 0x7d7, &[]);
        }
        let added_len = added.len();
        out.splice(insert_at..insert_at, added);
        let last_marker_object = self
            .objects
            .iter()
            .rev()
            .skip_while(|o| o.deleted)
            .find(|o| {
                matches!(
                    o.kind,
                    Kind::FunctionDefinition(_)
                        | Kind::DerivativeFunction(_, _)
                        | Kind::FunctionPlot(_, _, _, _, _)
                        | Kind::ParametricPlot(_, _, _, _, _, _, _)
                        | Kind::Locus(_, _)
                        | Kind::ParameterLocus(_, _, _, _)
                        | Kind::PlottedPoint(_, _, _, _)
                        | Kind::NumericCoordinatePoint(_, _, _)
                        | Kind::CoordinateMeasurement { .. }
                        | Kind::ReflectedPolygon(_, _)
                        | Kind::VectorTranslatedPolygon(_, _, _)
                        | Kind::PolarTranslatedPolygon(_, _, _, _)
                        | Kind::FixedRectangularTranslatedPolygon(_, _, _, _, _)
                        | Kind::MarkedDistanceTranslatedPolygon(_, _, _)
                        | Kind::RectangularMarkedTranslatedPolygon(_, _, _)
                        | Kind::MixedRectangularTranslatedPolygon(_, _, _, _)
                        | Kind::RotatedPolygon(_, _, _)
                        | Kind::CalculatedAngleRotatedPolygon(_, _, _)
                        | Kind::CalculatedRatioScaledPolygon(_, _, _)
                        | Kind::AngleRotatedPolygon(_, _, _, _, _)
                        | Kind::ScaledPolygon(_, _, _)
                        | Kind::RatioScaledPolygon(_, _, _, _, _)
                )
            })
            .map(|o| o.id);
        if let Some(id) = last_marker_object {
            if let Some(marker) = rs
                .iter()
                .find(|r| r.kind == 0x2328 && r.start >= insert_at && r.start < trailer)
            {
                if marker.data.len() == 4 {
                    let offset = marker.data.start + added_len;
                    out[offset..offset + 4].copy_from_slice(&(id as u32).to_le_bytes());
                }
            } else {
                let mut marker = Vec::new();
                put_record(&mut marker, 0x2328, &(id as u32).to_le_bytes());
                out.splice(insert_at + added_len..insert_at + added_len, marker);
            }
        }
        let mut out = self.patch_marked_coordinate_link(out)?;
        records(&out)?;
        if let Some(resource) = picture_resource {
            let resource_at = records(&out)?
                .iter()
                .find(|record| record.kind == 0x03ef)
                .ok_or("Missing picture resource section")?
                .start;
            out.splice(resource_at..resource_at, resource);
        }
        let out = self.patch_text_font_overrides(self.patch_defined_coordinate_grid_style(
            self.patch_visual_styles(
                self.patch_trace_flags(self.patch_text_and_labels(self.compact_deleted(out)?)?)?,
            )?,
        )?)?;
        self.patch_last_moved_marker(out)
    }
}
