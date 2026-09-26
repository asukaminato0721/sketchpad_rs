use super::{Document, Kind};
use crate::table_records::TableGroup;

impl Document {
    pub fn tabulate_default_parameter(
        &mut self,
        source: usize,
        anchor: (i16, i16),
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || source == 0 {
            return Err("Cannot tabulate a parameter in this sketch".into());
        }
        let Some(Kind::Parameter(parameter)) = self.objects.get(source - 1).map(|o| &o.kind) else {
            return Err("Select a supported parameter".into());
        };
        let group = if parameter.name() == "t[1]" {
            TableGroup::for_source_value(
                u32::try_from(source).map_err(|_| "Invalid parameter ID")?,
                anchor,
                parameter.value(),
            )?
        } else if !matches!(parameter.name(), "t[1]" | "t[2]") {
            TableGroup::named_snapshot(
                u32::try_from(source).map_err(|_| "Invalid parameter ID")?,
                parameter,
                anchor,
            )?
        } else {
            return Err("This default parameter table is not yet supported".into());
        };
        self.checkpoint();
        Ok(self.append_object(Kind::ValueTable(source, group)))
    }
    pub fn tabulate_default_pair(&mut self, anchor: (i16, i16)) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot tabulate this parameter pair".into());
        }
        let pair = self
            .objects
            .iter()
            .filter_map(|object| match &object.kind {
                Kind::Parameter(group) if !object.deleted && group.name() == "t[1]" => {
                    Some((object.id, 1, group.value()))
                }
                Kind::Parameter(group) if !object.deleted && group.name() == "t[2]" => {
                    Some((object.id, 2, group.value()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let [first, second] = pair.as_slice() else {
            return Err("Select supported t[1] and t[2] parameters".into());
        };
        if first.1 != 1 || second.1 != 2 {
            return Err("Select supported t[1] and t[2] parameters".into());
        }
        let sources = [
            u32::try_from(second.0).map_err(|_| "Invalid parameter ID")?,
            u32::try_from(first.0).map_err(|_| "Invalid parameter ID")?,
        ];
        let mut group = TableGroup::pair_snapshot_rows_for_sources(1, anchor, sources)?;
        group.set_pair_row_values(0, [second.2, first.2])?;
        self.checkpoint();
        Ok(self.append_object(Kind::ValueTablePair(second.0, first.0, group)))
    }
    pub fn move_number(&mut self, id: usize, anchor: (i16, i16)) -> Result<(), String> {
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid object ID")?)
            .ok_or("Invalid object ID")?;
        let mut kind = object.kind.clone();
        match &mut kind {
            Kind::Parameter(group) => group.set_anchor(anchor)?,
            Kind::Calculation(group) => group.set_anchor(anchor)?,
            Kind::MeasuredCalculation(_, group) => group.set_anchor(anchor)?,
            Kind::LengthPairCalculation(_, _, group) => group.set_anchor(anchor)?,
            Kind::ParameterCalculation(_, group) => group.set_anchor(anchor)?,
            Kind::ValueTable(_, group) | Kind::ValueTablePair(_, _, group) => {
                group.set_anchor(anchor)
            }
            _ => return Err("Object is not a number or table".into()),
        }
        self.checkpoint();
        self.objects[id - 1].kind = kind;
        Ok(())
    }
    pub fn change_table_rows(&mut self, id: usize, add: bool) -> Result<(), String> {
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid table ID")?)
            .ok_or("Invalid table ID")?;
        let kind = match &object.kind {
            Kind::ValueTable(source, group) => {
                let mut group = group.clone();
                if add {
                    if group.named_label_values().is_some() {
                        let Some(Kind::Parameter(parameter)) = self
                            .objects
                            .get(source.saturating_sub(1))
                            .map(|object| &object.kind)
                        else {
                            return Err("Invalid named table source".into());
                        };
                        crate::table_records::table_value_text(parameter.value())?;
                        let value = format!("{:.*}", parameter.precision(), parameter.value());
                        group.add_named_snapshot(&value)?;
                    } else {
                        group.add_default_snapshot()?;
                    }
                } else {
                    group.remove_last_snapshot()?;
                }
                Kind::ValueTable(*source, group)
            }
            Kind::ValueTablePair(first, second, group) => {
                let values = group.pair_values().ok_or("Invalid pair table")?;
                let rows = if add {
                    group.rows() + 1
                } else {
                    group.rows().checked_sub(1).ok_or("Table has no rows")?
                };
                let mut changed = TableGroup::pair_snapshot_rows_for_sources(
                    rows,
                    group.anchor(),
                    group
                        .pair_source_ids()
                        .ok_or("Invalid pair table sources")?,
                )?;
                for row in 0..rows {
                    let value = if row + 1 == rows {
                        values.last().unwrap()
                    } else {
                        &values[row]
                    };
                    changed.set_pair_row_values(row, *value)?;
                }
                Kind::ValueTablePair(*first, *second, changed)
            }
            _ => return Err("Object is not a table".into()),
        };
        self.checkpoint();
        self.objects[id - 1].kind = kind;
        Ok(())
    }
}
