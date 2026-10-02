use crate::model::{Model, Position, Selection};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LineNumber {
    pub line: usize,
    pub text: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextRun {
    pub line: usize,
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrameDescription {
    pub viewport: Size,
    pub background: Color,
    pub gutter: Rect,
    pub line_numbers: Vec<LineNumber>,
    pub text_runs: Vec<TextRun>,
    pub selections: Vec<Rect>,
    pub cursors: Vec<Rect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewConfig {
    pub background: Color,
    pub line_height: f32,
    pub advance: f32,
    pub gutter_padding: f32,
    pub gutter_minimum_width: f32,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            background: Color(30, 30, 30, 255),
            line_height: 20.0,
            advance: 10.0,
            gutter_padding: 12.0,
            gutter_minimum_width: 36.0,
        }
    }
}

pub fn build_frame(model: &Model, config: ViewConfig) -> FrameDescription {
    let viewport = Size {
        width: model.viewport.width as f32,
        height: model.viewport.height as f32,
    };
    let visible_count = visible_line_count(viewport.height, config.line_height);
    let first_line = model
        .viewport
        .vertical_offset
        .min(model.document.line_count().saturating_sub(1));
    let last_line = (first_line + visible_count).min(model.document.line_count());
    let gutter_width = gutter_width(model.document.line_count(), config);

    let line_numbers = (first_line..last_line)
        .enumerate()
        .map(|(visible_index, line)| LineNumber {
            line,
            text: (line + 1).to_string(),
            x: gutter_width
                - config.gutter_padding
                - (line + 1).to_string().len() as f32 * config.advance,
            y: visible_index as f32 * config.line_height,
        })
        .collect();

    let text_runs = (first_line..last_line)
        .map(|line| {
            let text = &model.document.lines[line];
            let visible_text = text
                .chars()
                .skip(model.viewport.horizontal_offset)
                .collect::<String>();
            let x = gutter_width - model.viewport.horizontal_offset as f32 * config.advance;

            TextRun {
                line,
                width: visible_text.chars().count() as f32 * config.advance,
                text: visible_text,
                x,
                y: (line - first_line) as f32 * config.line_height,
                height: config.line_height,
            }
        })
        .collect();

    let cursors = if model.cursor.line >= first_line && model.cursor.line < last_line {
        vec![Rect {
            x: gutter_width
                + (model.cursor.column as f32 - model.viewport.horizontal_offset as f32)
                    * config.advance,
            y: (model.cursor.line - first_line) as f32 * config.line_height,
            width: 2.0,
            height: config.line_height,
        }]
    } else {
        Vec::new()
    };

    let selections = selection_rects(
        model,
        model.selection.as_ref(),
        config,
        gutter_width,
        first_line,
        last_line,
    );

    FrameDescription {
        viewport,
        background: config.background,
        gutter: Rect {
            x: 0.0,
            y: 0.0,
            width: gutter_width,
            height: viewport.height,
        },
        line_numbers,
        text_runs,
        selections,
        cursors,
    }
}

fn selection_rects(
    model: &Model,
    selection: Option<&Selection>,
    config: ViewConfig,
    gutter_width: f32,
    first_line: usize,
    last_line: usize,
) -> Vec<Rect> {
    let Some(selection) = selection else {
        return Vec::new();
    };

    let (start, end) = if selection.anchor <= selection.focus {
        (&selection.anchor, &selection.focus)
    } else {
        (&selection.focus, &selection.anchor)
    };
    let first_selected_line = start.line.max(first_line);
    let last_selected_line = end.line.min(last_line.saturating_sub(1));
    if first_selected_line > last_selected_line {
        return Vec::new();
    }

    let horizontal_start = model.viewport.horizontal_offset;
    let horizontal_end = horizontal_start
        + ((model.viewport.width as f32 - gutter_width) / config.advance.max(1.0)).max(0.0)
            as usize;

    (first_selected_line..=last_selected_line)
        .filter_map(|line| {
            let line_start = if line == start.line { start.column } else { 0 };
            let line_end = if line == end.line {
                end.column
            } else {
                model.document.line_len(line)
            };
            let visible_start = line_start.max(horizontal_start).min(horizontal_end);
            let visible_end = line_end.max(horizontal_start).min(horizontal_end);
            (visible_end > visible_start).then_some(Rect {
                x: gutter_width + (visible_start - horizontal_start) as f32 * config.advance,
                y: (line - first_line) as f32 * config.line_height,
                width: (visible_end - visible_start) as f32 * config.advance,
                height: config.line_height,
            })
        })
        .collect()
}

pub fn position_at_point(model: &Model, config: ViewConfig, x: f32, y: f32) -> Position {
    let visible_line_count = model.document.line_count().max(1);
    let line = model.viewport.vertical_offset
        + (y.max(0.0) / config.line_height.max(1.0)).floor() as usize;
    let line = line.min(visible_line_count - 1);
    let gutter_width = gutter_width(model.document.line_count(), config);
    let column = ((x - gutter_width).max(0.0) / config.advance.max(1.0)).floor() as usize
        + model.viewport.horizontal_offset;

    model.document.clamp_position(Position { line, column })
}

fn visible_line_count(height: f32, line_height: f32) -> usize {
    if line_height <= 0.0 {
        1
    } else {
        (height / line_height).ceil().max(1.0) as usize
    }
}

fn gutter_width(line_count: usize, config: ViewConfig) -> f32 {
    let digits = line_count.max(1).to_string().len() as f32;
    (digits * config.advance + config.gutter_padding).max(config.gutter_minimum_width)
}

#[cfg(test)]
mod tests {
    use super::{Color, ViewConfig, build_frame, position_at_point};
    use crate::model::{Model, Position, Selection};

    fn model_with_text(text: &str) -> Model {
        let mut model = Model::default();
        model.document.insert_text(&mut model.cursor, text);
        model
    }

    #[test]
    fn frame_slices_lines_and_places_line_numbers() {
        let mut model = model_with_text("one\ntwo\nthree");
        model.viewport.width = 200;
        model.viewport.height = 40;

        let frame = build_frame(&model, ViewConfig::default());

        assert_eq!(frame.line_numbers.len(), 2);
        assert_eq!(frame.line_numbers[0].text, "1");
        assert_eq!(frame.line_numbers[1].text, "2");
        assert_eq!(frame.text_runs[1].text, "two");
    }

    #[test]
    fn text_run_covers_each_source_scalar_without_gaps() {
        let model = model_with_text("abcd");
        let config = ViewConfig::default();
        let frame = build_frame(&model, config);

        assert_eq!(frame.text_runs[0].text, "abcd");
        assert_eq!(frame.text_runs[0].width, 4.0 * config.advance);
        assert_eq!(frame.text_runs[0].x, frame.gutter.width);
    }

    #[test]
    fn frame_respects_vertical_and_horizontal_offsets() {
        let mut model = model_with_text("abcdef\nsecond");
        model.viewport.width = 200;
        model.viewport.height = 20;
        model.viewport.vertical_offset = 1;
        model.viewport.horizontal_offset = 2;
        model.cursor = Position { line: 1, column: 3 };

        let frame = build_frame(&model, ViewConfig::default());

        assert_eq!(frame.text_runs[0].line, 1);
        assert_eq!(frame.text_runs[0].text, "cond");
        assert_eq!(frame.cursors.len(), 1);
        assert_eq!(frame.cursors[0].x, 46.0);
    }

    #[test]
    fn point_mapping_clamps_to_document_and_uses_scalar_columns() {
        let mut model = model_with_text("😀ab");
        model.viewport.width = 200;
        model.viewport.height = 40;
        let config = ViewConfig::default();

        assert_eq!(
            position_at_point(&model, config, 0.0, 0.0),
            Position { line: 0, column: 0 }
        );
        assert_eq!(
            position_at_point(&model, config, 1000.0, 0.0),
            Position { line: 0, column: 3 }
        );
    }

    #[test]
    fn frame_keeps_configured_background() {
        let config = ViewConfig {
            background: Color(1, 2, 3, 255),
            ..ViewConfig::default()
        };

        assert_eq!(
            build_frame(&Model::default(), config).background,
            Color(1, 2, 3, 255)
        );
    }

    #[test]
    fn unicode_scalars_keep_fixed_layout_widths() {
        let model = model_with_text("😀e\u{301}");
        let frame = build_frame(&model, ViewConfig::default());

        assert_eq!(frame.text_runs[0].text, "😀e\u{301}");
        assert_eq!(frame.text_runs[0].width, 30.0);
    }

    #[test]
    fn tabs_use_one_scalar_advance_until_tab_stops_are_added() {
        let model = model_with_text("a\tb");
        let frame = build_frame(&model, ViewConfig::default());

        assert_eq!(frame.text_runs[0].width, 30.0);
    }

    #[test]
    fn cursor_at_line_end_is_after_all_unicode_scalars() {
        let mut model = model_with_text("😀ab");
        model.viewport.width = 200;
        model.viewport.height = 40;
        let config = ViewConfig::default();
        let frame = build_frame(&model, config);

        assert_eq!(model.cursor.column, 3);
        assert_eq!(frame.cursors[0].x, 66.0);
    }

    #[test]
    fn frame_emits_visible_selection_rectangles_for_multiline_ranges() {
        let mut model = model_with_text("abcd\nefgh");
        model.selection = Some(Selection {
            anchor: Position { line: 0, column: 2 },
            focus: Position { line: 1, column: 2 },
        });
        model.viewport.width = 200;
        model.viewport.height = 40;

        let frame = build_frame(&model, ViewConfig::default());

        assert_eq!(frame.selections.len(), 2);
        assert_eq!(frame.selections[0].x, 56.0);
        assert_eq!(frame.selections[0].width, 20.0);
        assert_eq!(frame.selections[1].width, 20.0);
    }
}
