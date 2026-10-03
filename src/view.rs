use crate::model::{Model, Position, Selection};
use crate::mvu::COMMAND_BAR_COMMANDS;
use crate::syntax::{HighlightKind, SyntaxLanguage, highlight_line};
use crate::text::{FixedTextLayoutProvider, FontId, TextLayoutProvider};

const MENU_BAR_HEIGHT: f32 = 24.0;

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
pub struct Scrollbar {
    pub track: Rect,
    pub thumb: Rect,
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
    pub color: Color,
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
    pub vertical_scrollbar: Option<Scrollbar>,
    pub horizontal_scrollbar: Option<Scrollbar>,
    pub menu_bar: Rect,
    pub command_bar: Option<Rect>,
    pub settings_panel: Option<Rect>,
    pub overlay_text: Vec<TextRun>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewConfig {
    pub background: Color,
    pub line_height: f32,
    pub advance: f32,
    pub gutter_padding: f32,
    pub gutter_minimum_width: f32,
    pub scrollbar_size: f32,
}

#[derive(Clone, Copy)]
struct TextLayout {
    first_line: usize,
    content_top: f32,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            background: Color(30, 30, 30, 255),
            line_height: 20.0,
            advance: 10.0,
            gutter_padding: 12.0,
            gutter_minimum_width: 36.0,
            scrollbar_size: 12.0,
        }
    }
}

#[allow(dead_code)]
pub fn build_frame(model: &Model, config: ViewConfig) -> FrameDescription {
    let mut text_provider = FixedTextLayoutProvider;
    build_frame_with_text(model, config, &mut text_provider)
}

pub fn build_frame_with_text(
    model: &Model,
    config: ViewConfig,
    text_provider: &mut dyn TextLayoutProvider,
) -> FrameDescription {
    let viewport = Size {
        width: model.viewport.width as f32,
        height: model.viewport.height as f32,
    };
    let content_height = (viewport.height - MENU_BAR_HEIGHT).max(0.0);
    let visible_count = visible_line_count(content_height, config.line_height);
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
            y: MENU_BAR_HEIGHT + visible_index as f32 * config.line_height,
        })
        .collect();

    let text_runs = (first_line..last_line)
        .flat_map(|line| {
            text_runs_for_line(
                line,
                &model.document.line_text(line),
                model.document.language,
                model.viewport.horizontal_offset,
                gutter_width,
                config,
                text_provider,
                TextLayout {
                    first_line,
                    content_top: MENU_BAR_HEIGHT,
                },
            )
        })
        .collect();

    let cursors = if model.cursor.line >= first_line && model.cursor.line < last_line {
        let line_layout = text_provider.layout_single_line(
            &model.document.line_text(model.cursor.line),
            FontId(0),
            config.advance,
        );
        vec![Rect {
            x: gutter_width + line_layout.cursor_x(model.cursor.column)
                - model.viewport.horizontal_offset as f32 * config.advance,
            y: MENU_BAR_HEIGHT + (model.cursor.line - first_line) as f32 * config.line_height,
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
        MENU_BAR_HEIGHT,
        text_provider,
    );
    let (vertical_scrollbar, horizontal_scrollbar) =
        scrollbars(model, config, gutter_width, MENU_BAR_HEIGHT);
    let menu_bar = Rect {
        x: 0.0,
        y: 0.0,
        width: viewport.width,
        height: MENU_BAR_HEIGHT,
    };
    let command_bar = model.command_bar_open.then_some(Rect {
        x: 120.0,
        y: 32.0,
        width: (viewport.width - 160.0).max(180.0),
        height: 28.0 + COMMAND_BAR_COMMANDS.len() as f32 * 18.0,
    });
    let settings_panel = model.settings_open.then_some(Rect {
        x: 120.0,
        y: 72.0,
        width: (viewport.width - 160.0).max(240.0),
        height: 120.0,
    });
    let mut overlay_text = vec![TextRun {
        line: 0,
        text: "File    View    Settings    Ctrl+K Command Bar".into(),
        x: 12.0,
        y: 5.0,
        width: 0.0,
        height: 0.0,
        color: Color(220, 220, 220, 255),
    }];
    if model.command_bar_open {
        overlay_text.push(TextRun {
            line: 0,
            text: format!("> {}", model.command_bar_query),
            x: 132.0,
            y: 38.0,
            width: 0.0,
            height: 0.0,
            color: Color(245, 210, 120, 255),
        });
        for (index, command) in COMMAND_BAR_COMMANDS.iter().enumerate() {
            overlay_text.push(TextRun {
                line: 0,
                text: format!(
                    "{}{}",
                    if index == model.command_bar_selection {
                        "> "
                    } else {
                        "  "
                    },
                    command
                ),
                x: 132.0,
                y: 62.0 + index as f32 * 18.0,
                width: 0.0,
                height: 0.0,
                color: if index == model.command_bar_selection {
                    Color(245, 210, 120, 255)
                } else {
                    Color(220, 220, 220, 255)
                },
            });
        }
    }
    if model.settings_open {
        overlay_text.push(TextRun {
            line: 0,
            text: "Settings  |  Font: Consolas / Segoe UI Emoji".into(),
            x: 132.0,
            y: 80.0,
            width: 0.0,
            height: 0.0,
            color: Color(220, 220, 220, 255),
        });
    }

    FrameDescription {
        viewport,
        background: config.background,
        gutter: Rect {
            x: 0.0,
            y: MENU_BAR_HEIGHT,
            width: gutter_width,
            height: content_height,
        },
        line_numbers,
        text_runs,
        selections,
        cursors,
        vertical_scrollbar,
        horizontal_scrollbar,
        menu_bar,
        command_bar,
        settings_panel,
        overlay_text,
    }
}

fn scrollbars(
    model: &Model,
    config: ViewConfig,
    gutter_width: f32,
    content_top: f32,
) -> (Option<Scrollbar>, Option<Scrollbar>) {
    let viewport = Size {
        width: model.viewport.width as f32,
        height: model.viewport.height as f32,
    };
    let visible_lines =
        visible_line_count((viewport.height - content_top).max(0.0), config.line_height);
    let vertical = (model.document.line_count() > visible_lines).then(|| {
        let track = Rect {
            x: (viewport.width - config.scrollbar_size).max(0.0),
            y: content_top,
            width: config.scrollbar_size,
            height: (viewport.height - content_top).max(0.0),
        };
        let thumb_height = (track.height * visible_lines as f32
            / model.document.line_count() as f32)
            .max(config.scrollbar_size * 2.0)
            .min(track.height);
        let max_offset = model.document.line_count() - visible_lines;
        let travel = (track.height - thumb_height).max(0.0);
        let thumb = Rect {
            x: track.x,
            y: track.y
                + travel * model.viewport.vertical_offset.min(max_offset) as f32
                    / max_offset.max(1) as f32,
            width: track.width,
            height: thumb_height,
        };
        Scrollbar { track, thumb }
    });

    let longest_line = (0..model.document.line_count())
        .map(|line| model.document.line_len(line))
        .max()
        .unwrap_or(0);
    let visible_columns = ((viewport.width - gutter_width - config.scrollbar_size).max(0.0)
        / config.advance.max(1.0)) as usize;
    let horizontal = (longest_line > visible_columns).then(|| {
        let track = Rect {
            x: gutter_width,
            y: (viewport.height - config.scrollbar_size).max(content_top),
            width: (viewport.width - gutter_width).max(0.0),
            height: config.scrollbar_size,
        };
        let thumb_width = (track.width * visible_columns as f32 / longest_line as f32)
            .max(config.scrollbar_size * 2.0)
            .min(track.width);
        let max_offset = longest_line - visible_columns;
        let travel = (track.width - thumb_width).max(0.0);
        let thumb = Rect {
            x: track.x
                + travel * model.viewport.horizontal_offset.min(max_offset) as f32
                    / max_offset.max(1) as f32,
            y: track.y,
            width: thumb_width,
            height: track.height,
        };
        Scrollbar { track, thumb }
    });

    (vertical, horizontal)
}

pub fn scroll_limits(model: &Model, config: ViewConfig) -> (usize, usize) {
    let viewport = Size {
        width: model.viewport.width as f32,
        height: model.viewport.height as f32,
    };
    let content_top = MENU_BAR_HEIGHT;
    let visible_lines =
        visible_line_count((viewport.height - content_top).max(0.0), config.line_height);
    let vertical = model.document.line_count().saturating_sub(visible_lines);
    let gutter_width = gutter_width(model.document.line_count(), config);
    let visible_columns = ((viewport.width - gutter_width - config.scrollbar_size).max(0.0)
        / config.advance.max(1.0)) as usize;
    let longest_line = (0..model.document.line_count())
        .map(|line| model.document.line_len(line))
        .max()
        .unwrap_or(0);
    let horizontal = longest_line.saturating_sub(visible_columns);

    (vertical, horizontal)
}

fn text_runs_for_line(
    line: usize,
    text: &str,
    language: SyntaxLanguage,
    horizontal_offset: usize,
    gutter_width: f32,
    config: ViewConfig,
    text_provider: &mut dyn TextLayoutProvider,
    layout: TextLayout,
) -> Vec<TextRun> {
    let character_count = text.chars().count();
    let line_layout = text_provider.layout_single_line(text, FontId(0), config.advance);
    let spans = highlight_line(text, language);
    let mut boundaries = vec![0, character_count];
    boundaries.extend(spans.iter().flat_map(|span| [span.start, span.end]));
    boundaries.sort_unstable();
    boundaries.dedup();

    boundaries
        .windows(2)
        .filter_map(|window| {
            let start = window[0];
            let end = window[1];
            let visible_start = start.max(horizontal_offset);
            let visible_end = end.max(horizontal_offset);
            (visible_end > visible_start).then(|| {
                let bounds = line_layout.selection_bounds(visible_start, visible_end);
                let run_text = text
                    .chars()
                    .skip(visible_start)
                    .take(visible_end - visible_start)
                    .collect::<String>();
                let kind = spans
                    .iter()
                    .find(|span| span.start <= start && start < span.end)
                    .map(|span| span.kind);
                TextRun {
                    line,
                    text: run_text,
                    x: gutter_width + bounds.x - horizontal_offset as f32 * config.advance,
                    y: layout.content_top + (line - layout.first_line) as f32 * config.line_height,
                    width: bounds.width,
                    height: bounds.height,
                    color: color_for_highlight(kind),
                }
            })
        })
        .collect()
}

fn color_for_highlight(kind: Option<HighlightKind>) -> Color {
    match kind {
        Some(HighlightKind::Keyword) => Color(238, 130, 238, 255),
        Some(HighlightKind::String) => Color(152, 195, 121, 255),
        Some(HighlightKind::Comment) => Color(128, 140, 128, 255),
        Some(HighlightKind::Number) => Color(209, 154, 102, 255),
        Some(HighlightKind::Punctuation) => Color(171, 178, 191, 255),
        Some(HighlightKind::Heading) => Color(97, 175, 239, 255),
        Some(HighlightKind::Emphasis) => Color(229, 192, 123, 255),
        Some(HighlightKind::Code) => Color(198, 120, 221, 255),
        Some(HighlightKind::Link) => Color(86, 182, 194, 255),
        None => Color(220, 220, 220, 255),
    }
}

fn selection_rects(
    model: &Model,
    selection: Option<&Selection>,
    config: ViewConfig,
    gutter_width: f32,
    first_line: usize,
    last_line: usize,
    content_top: f32,
    text_provider: &mut dyn TextLayoutProvider,
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
            let line_layout = text_provider.layout_single_line(
                &model.document.line_text(line),
                FontId(0),
                config.advance,
            );
            let line_start = if line == start.line { start.column } else { 0 };
            let line_end = if line == end.line {
                end.column
            } else {
                model.document.line_len(line)
            };
            let visible_start = line_start.max(horizontal_start).min(horizontal_end);
            let visible_end = line_end.max(horizontal_start).min(horizontal_end);
            (visible_end > visible_start).then_some({
                let bounds = line_layout.selection_bounds(visible_start, visible_end);
                Rect {
                    x: gutter_width + bounds.x - horizontal_start as f32 * config.advance,
                    y: content_top + (line - first_line) as f32 * config.line_height,
                    width: bounds.width,
                    height: bounds.height,
                }
            })
        })
        .collect()
}

#[allow(dead_code)]
pub fn position_at_point(model: &Model, config: ViewConfig, x: f32, y: f32) -> Position {
    let mut text_provider = FixedTextLayoutProvider;
    position_at_point_with_text(model, config, x, y, &mut text_provider)
}

pub fn position_at_point_with_text(
    model: &Model,
    config: ViewConfig,
    x: f32,
    y: f32,
    text_provider: &mut dyn TextLayoutProvider,
) -> Position {
    let visible_line_count = model.document.line_count().max(1);
    let line = model.viewport.vertical_offset
        + ((y - MENU_BAR_HEIGHT).max(0.0) / config.line_height.max(1.0)).floor() as usize;
    let line = line.min(visible_line_count - 1);
    let gutter_width = gutter_width(model.document.line_count(), config);
    let line_layout = text_provider.layout_single_line(
        &model.document.line_text(line),
        FontId(0),
        config.advance,
    );
    let column = line_layout.hit_test(
        (x - gutter_width).max(0.0) + model.viewport.horizontal_offset as f32 * config.advance,
    );

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
    use crate::syntax::SyntaxLanguage;

    fn model_with_text(text: &str) -> Model {
        let mut model = Model::default();
        model.document.insert_text(&mut model.cursor, text);
        model
    }

    #[test]
    fn frame_slices_lines_and_places_line_numbers() {
        let mut model = model_with_text("one\ntwo\nthree");
        model.viewport.width = 200;
        model.viewport.height = 64;

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
        model.viewport.height = 64;
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
    fn rust_lines_are_split_into_styled_text_runs() {
        let mut model = model_with_text("fn main() {}");
        model.document.language = SyntaxLanguage::Rust;

        let frame = build_frame(&model, ViewConfig::default());

        assert!(frame.text_runs.len() > 1);
        assert_eq!(frame.text_runs[0].text, "fn");
        assert_ne!(frame.text_runs[0].color, frame.text_runs[1].color);
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
        model.viewport.height = 64;

        let frame = build_frame(&model, ViewConfig::default());

        assert_eq!(frame.selections.len(), 2);
        assert_eq!(frame.selections[0].x, 56.0);
        assert_eq!(frame.selections[0].width, 20.0);
        assert_eq!(frame.selections[1].width, 20.0);
    }

    #[test]
    fn frame_emits_scrollbars_for_content_beyond_the_viewport() {
        let mut model = model_with_text(&format!("{}\nlong line", "line\n".repeat(20)));
        model.viewport.width = 120;
        model.viewport.height = 40;
        model.viewport.vertical_offset = 2;
        model.viewport.horizontal_offset = 3;

        let frame = build_frame(&model, ViewConfig::default());

        assert!(frame.vertical_scrollbar.is_some());
        assert!(frame.horizontal_scrollbar.is_some());
        assert!(frame.vertical_scrollbar.unwrap().thumb.y > 0.0);
        assert!(frame.horizontal_scrollbar.unwrap().thumb.x > 0.0);
    }
}
