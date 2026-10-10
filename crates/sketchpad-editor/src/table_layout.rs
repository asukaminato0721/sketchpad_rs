//! Cell layout shared by the canvas, SVG export, and table hit testing.
use super::{Viewport, display_parameter_indices, math_text, xml_text};
use crate::document::{Document, Kind};
use crate::text_style_records::FontState;
use egui::{Color32, Pos2, Rect, Stroke, vec2};
use std::{fmt::Write, sync::Arc};

struct Cell {
    rect: Rect,
    text: String,
    font: FontState,
    galley: Arc<egui::Galley>,
}

pub(crate) struct TableLayout {
    pub rect: Rect,
    cells: Vec<Cell>,
    header_height: f32,
    rules: Vec<[Pos2; 2]>,
}

pub(crate) fn is_table(doc: &Document, id: usize) -> bool {
    id.checked_sub(1)
        .and_then(|index| doc.objects.get(index))
        .is_some_and(|object| {
            matches!(
                object.kind,
                Kind::DataTable(..)
                    | Kind::ValueTable(..)
                    | Kind::ValueTablePair(..)
                    | Kind::IterationValues(..)
            )
        })
}

type TableRows = Vec<Vec<String>>;

fn table_cells(doc: &Document, id: usize) -> Option<((i16, i16), TableRows)> {
    let object = doc.objects.get(id.checked_sub(1)?)?;
    match &object.kind {
        Kind::DataTable(_, group) => Some((group.anchor(), doc.data_table_cells(id)?)),
        Kind::ValueTable(..) => {
            let (anchor, lines) = super::number_lines_for_object(doc, id)?;
            Some((anchor, lines.into_iter().map(|line| vec![line]).collect()))
        }
        Kind::ValueTablePair(_, _, group) => {
            let mut rows = vec![vec!["t₂".into(), "t₁".into()]];
            rows.extend(
                group
                    .pair_values()?
                    .into_iter()
                    .map(|[x, y]| vec![format!("{x:.2}"), format!("{y:.2}")]),
            );
            Some((group.anchor(), rows))
        }
        Kind::IterationValues(..) => Some((
            object.measurement_anchor?,
            doc.iteration_value_table(id)?
                .into_iter()
                .map(|row| {
                    row.iter()
                        .map(|cell| display_parameter_indices(cell))
                        .collect()
                })
                .collect(),
        )),
        _ => None,
    }
}

pub(crate) fn layout(
    vp: &Viewport,
    doc: &Document,
    id: usize,
    ctx: &egui::Context,
) -> Option<TableLayout> {
    let (anchor, rows) = table_cells(doc, id)?;
    let columns = rows.iter().map(Vec::len).max().filter(|&count| count > 0)?;
    let font = doc.text_font_state(id).unwrap_or(FontState {
        family: "sans-serif".into(),
        size_code: 16,
        bold: false,
        italic: false,
        underline: false,
    });
    let size = f32::from(font.size_code).max(1.);
    let mut widths = vec![size * 3. + 20.; columns];
    let mut heights = vec![size + 12.; rows.len()];
    let mut cells = Vec::new();
    for (row_index, row) in rows.into_iter().enumerate() {
        for (column, text) in row.into_iter().enumerate() {
            let mut cell_font = font.clone();
            cell_font.bold |= row_index == 0;
            let galley = math_text::measure(ctx, &text, &cell_font, size, Color32::BLACK);
            widths[column] = widths[column].max((galley.size().x + 24.).ceil());
            heights[row_index] = heights[row_index].max((galley.size().y + 12.).ceil());
            cells.push((row_index, column, text, cell_font, galley));
        }
    }
    let origin = vp.rect.min + vec2(f32::from(anchor.0), f32::from(anchor.1));
    let edges = |start: f32, sizes: &[f32]| {
        let mut values = vec![start];
        for size in sizes {
            values.push(values.last().unwrap() + size);
        }
        values
    };
    let xs = edges(origin.x, &widths);
    let ys = edges(origin.y, &heights);
    let rect = Rect::from_min_max(origin, egui::pos2(*xs.last()?, *ys.last()?));
    let cells = cells
        .into_iter()
        .map(|(row, col, text, font, galley)| Cell {
            rect: Rect::from_min_max(
                egui::pos2(xs[col], ys[row]),
                egui::pos2(xs[col + 1], ys[row + 1]),
            ),
            text,
            font,
            galley,
        })
        .collect();
    let rules = xs[1..xs.len() - 1]
        .iter()
        .map(|&x| [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())])
        .chain(
            ys[1..ys.len() - 1]
                .iter()
                .map(|&y| [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)]),
        )
        .collect();
    Some(TableLayout {
        rect,
        cells,
        header_height: heights[0],
        rules,
    })
}

impl TableLayout {
    fn header_rect(&self) -> Rect {
        Rect::from_min_size(self.rect.min, vec2(self.rect.width(), self.header_height))
    }

    // The engine also compiles this module for SVG regression tests.
    #[allow(dead_code)]
    pub(crate) fn paint(&self, painter: &egui::Painter) {
        painter.rect_filled(self.rect, 0., Color32::WHITE);
        painter.rect_filled(self.header_rect(), 0., Color32::from_gray(235));
        for cell in &self.cells {
            let at = cell.rect.center() - cell.galley.size() * 0.5;
            painter.galley(at, cell.galley.clone(), Color32::BLACK);
            if cell.font.bold {
                painter.galley(at + vec2(0.7, 0.), cell.galley.clone(), Color32::BLACK);
            }
        }
        for rule in &self.rules {
            painter.line_segment(*rule, Stroke::new(1., Color32::from_gray(110)));
        }
        painter.rect_stroke(
            self.rect,
            0.,
            Stroke::new(1., Color32::from_gray(70)),
            egui::StrokeKind::Inside,
        );
    }

    pub(crate) fn write_svg(&self, svg: &mut String, id: usize) {
        writeln!(svg, "<g data-table-id=\"{id}\">").unwrap();
        for (rect, fill) in [(self.rect, "#ffffff"), (self.header_rect(), "#ebebeb")] {
            writeln!(
                svg,
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{fill}\"/>",
                rect.left(),
                rect.top(),
                rect.width(),
                rect.height()
            )
            .unwrap();
        }
        for cell in &self.cells {
            let center = cell.rect.center();
            writeln!(svg,
                "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\" font-style=\"{}\" text-decoration=\"{}\" fill=\"#000000\">{}</text>",
                center.x, center.y, xml_text(&cell.font.family).replace('"', "&quot;"),
                cell.font.size_code.max(1),
                if cell.font.bold { "bold" } else { "normal" },
                if cell.font.italic { "italic" } else { "normal" },
                if cell.font.underline { "underline" } else { "none" },
                xml_text(&cell.text)).unwrap();
        }
        for [a, b] in &self.rules {
            writeln!(svg, "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#6e6e6e\" stroke-width=\"1\"/>", a.x, a.y, b.x, b.y).unwrap();
        }
        writeln!(svg, "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"#464646\" stroke-width=\"1\"/>\n</g>",
            self.rect.left(), self.rect.top(), self.rect.width(), self.rect.height()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_fit_text_and_keep_columns_aligned_in_canvas_svg_and_picking() {
        let doc = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/plot_table_data_two_rows_original.gsp"),
            None,
        )
        .unwrap();
        let id = doc
            .objects
            .iter()
            .find(|object| is_table(&doc, object.id))
            .unwrap()
            .id;
        let ctx = math_text::font_context(&doc);
        let vp = Viewport {
            rect: Rect::from_min_size(egui::pos2(37., 23.), vec2(1000., 700.)),
            scale: 1.,
            center: crate::document::Point { x: 0., y: 0. },
        };
        let table = layout(&vp, &doc, id, &ctx).unwrap();
        let (anchor, rows) = table_cells(&doc, id).unwrap();
        let columns = rows[0].len();
        assert!(columns >= 2);
        assert!(rows.len() >= 3);
        assert_eq!(
            table.rect.min,
            vp.rect.min + vec2(anchor.0 as f32, anchor.1 as f32)
        );
        assert_eq!(table.cells.len(), rows.len() * columns);
        for (index, cell) in table.cells.iter().enumerate() {
            let header = &table.cells[index % columns];
            assert_eq!(cell.rect.left(), header.rect.left());
            assert_eq!(cell.rect.right(), header.rect.right());
            assert!(cell.rect.width() >= cell.galley.size().x + 20.);
            assert!(cell.rect.height() >= cell.galley.size().y + 10.);
            assert!(table.rect.contains_rect(cell.rect));
        }
        assert_eq!(
            super::super::number_screen_rect_with_fonts(&vp, &doc, id, Some(&ctx)),
            Some(table.rect),
        );
        let mut svg = String::new();
        table.write_svg(&mut svg, id);
        assert_eq!(svg.matches("<text ").count(), table.cells.len());
        assert_eq!(svg.matches("<line ").count(), rows.len() + columns - 2);
        for cell in &table.cells {
            assert!(svg.contains(&format!(
                "x=\"{}\" y=\"{}\" text-anchor=\"middle\"",
                cell.rect.center().x,
                cell.rect.center().y
            )));
            assert!(svg.contains(&xml_text(&cell.text)));
        }
        // Tables use saved screen anchors. Diagram zoom must not change their bounds.
        let zoomed = Viewport { scale: 2., ..vp };
        assert_eq!(layout(&zoomed, &doc, id, &ctx).unwrap().rect, table.rect);
    }

    #[test]
    fn named_table_keeps_recorded_rows_and_renders_the_current_value_in_its_cell() {
        let mut doc = Document::new();
        doc.add_named_parameter("speed", 2.5, (10, 5)).unwrap();
        let id = doc.tabulate_default_parameter(1, (10, 45)).unwrap();
        doc.change_table_rows(id, true).unwrap();
        doc.set_parameter_value(1, 93.25).unwrap();
        let ctx = math_text::font_context(&doc);
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, vec2(1000., 700.)),
            scale: 1.,
            center: crate::document::Point { x: 0., y: 0. },
        };
        let table = layout(&vp, &doc, id, &ctx).unwrap();
        assert_eq!(table.cells[1].text, "2.5");
        assert_eq!(table.cells[2].text, "93.25");
        assert!(
            table
                .cells
                .iter()
                .all(|cell| cell.rect.width() >= cell.galley.size().x + 20.)
        );
        let mut svg = String::new();
        table.write_svg(&mut svg, id);
        assert!(svg.contains(">speed</text>"));
        assert!(svg.contains(">93.25</text>"));
    }
}
