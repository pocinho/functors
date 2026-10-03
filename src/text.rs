use std::cmp::Ordering;

/// Identifies a font resource without exposing a font backend to layout users.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontStack {
    families: Vec<String>,
}

impl FontStack {
    pub fn new(primary: impl Into<String>) -> Self {
        Self {
            families: vec![primary.into()],
        }
    }

    pub fn with_fallback(mut self, family: impl Into<String>) -> Self {
        self.families.push(family.into());
        self
    }

    pub fn families(&self) -> &[String] {
        &self.families
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FontCatalog {
    entries: Vec<(String, FontId)>,
}

pub trait TextLayoutProvider {
    fn layout_single_line(&mut self, text: &str, font: FontId, font_size: f32) -> SingleLineLayout;
}

#[derive(Default)]
pub struct FixedTextLayoutProvider;

impl TextLayoutProvider for FixedTextLayoutProvider {
    fn layout_single_line(&mut self, text: &str, font: FontId, font_size: f32) -> SingleLineLayout {
        SingleLineLayout::new(text, font, font_size, font_size)
    }
}

impl FontCatalog {
    pub fn register(&mut self, family: impl Into<String>, font: FontId) {
        let family = family.into();
        if let Some((registered_family, registered_font)) = self
            .entries
            .iter_mut()
            .find(|(registered_family, _)| registered_family.eq_ignore_ascii_case(&family))
        {
            *registered_family = family;
            *registered_font = font;
        } else {
            self.entries.push((family, font));
        }
    }

    pub fn resolve(&self, stack: &FontStack) -> Option<FontId> {
        stack.families.iter().find_map(|family| {
            self.entries
                .iter()
                .find(|(registered_family, _)| registered_family.eq_ignore_ascii_case(family))
                .map(|(_, font)| *font)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphPosition {
    pub glyph_id: u32,
    /// Source offsets are Unicode scalar offsets, not UTF-8 byte offsets.
    pub source_start: usize,
    pub source_end: usize,
    pub x: f32,
    pub y: f32,
    pub advance: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphRun {
    pub font: FontId,
    pub glyphs: Vec<GlyphPosition>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SingleLineLayout {
    pub text: String,
    pub run: GlyphRun,
    pub metrics: TextMetrics,
    pub bounds: TextBounds,
    pub caret_positions: Vec<f32>,
}

impl SingleLineLayout {
    pub fn new(text: &str, font: FontId, advance: f32, line_height: f32) -> Self {
        let advance = finite_positive(advance);
        let line_height = finite_positive(line_height);
        let glyphs = text
            .chars()
            .enumerate()
            .map(|(source_start, character)| GlyphPosition {
                glyph_id: character as u32,
                source_start,
                source_end: source_start + 1,
                x: source_start as f32 * advance,
                y: 0.0,
                advance,
            })
            .collect::<Vec<_>>();
        let width = glyphs.len() as f32 * advance;
        let caret_positions = (0..=glyphs.len())
            .map(|scalar_offset| scalar_offset as f32 * advance)
            .collect();

        Self {
            text: text.to_owned(),
            run: GlyphRun { font, glyphs },
            metrics: TextMetrics {
                ascent: line_height * 0.8,
                descent: line_height * 0.2,
                line_height,
            },
            bounds: TextBounds {
                x: 0.0,
                y: 0.0,
                width,
                height: line_height,
            },
            caret_positions,
        }
    }

    pub(crate) fn from_parts(
        text: String,
        run: GlyphRun,
        metrics: TextMetrics,
        bounds: TextBounds,
        caret_positions: Vec<f32>,
    ) -> Self {
        Self {
            text,
            run,
            metrics,
            bounds,
            caret_positions,
        }
    }

    pub fn scalar_len(&self) -> usize {
        self.caret_positions.len().saturating_sub(1)
    }

    pub fn cursor_x(&self, scalar_offset: usize) -> f32 {
        let offset = scalar_offset.min(self.scalar_len());
        self.caret_positions
            .get(offset)
            .copied()
            .unwrap_or(self.bounds.width)
    }

    pub fn selection_bounds(&self, start: usize, end: usize) -> TextBounds {
        let (start, end) = match start.cmp(&end) {
            Ordering::Greater => (end, start),
            _ => (start, end),
        };
        let start_x = self.cursor_x(start);
        let end_x = self.cursor_x(end);
        TextBounds {
            x: start_x,
            y: 0.0,
            width: (end_x - start_x).max(0.0),
            height: self.metrics.line_height,
        }
    }

    pub fn hit_test(&self, x: f32) -> usize {
        if !x.is_finite() || x <= 0.0 {
            return 0;
        }

        self.caret_positions
            .windows(2)
            .position(|caret_pair| x < caret_pair[0] + (caret_pair[1] - caret_pair[0]) * 0.5)
            .unwrap_or(self.scalar_len())
    }
}

fn finite_positive(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::{FontCatalog, FontId, FontStack, SingleLineLayout};

    #[test]
    fn font_catalog_resolves_the_first_available_fallback() {
        let mut catalog = FontCatalog::default();
        catalog.register("Editor Sans", FontId(7));
        catalog.register("Emoji Fallback", FontId(9));
        let stack = FontStack::new("Missing").with_fallback("emoji fallback");

        assert_eq!(stack.families(), ["Missing", "emoji fallback"]);
        assert_eq!(catalog.resolve(&stack), Some(FontId(9)));
    }

    #[test]
    fn registering_a_family_replaces_its_resource_case_insensitively() {
        let mut catalog = FontCatalog::default();
        catalog.register("Editor Sans", FontId(1));
        catalog.register("editor sans", FontId(2));

        assert_eq!(
            catalog.resolve(&FontStack::new("EDITOR SANS")),
            Some(FontId(2))
        );
    }

    #[test]
    fn layout_tracks_scalar_offsets_and_fractional_positions() {
        let layout = SingleLineLayout::new("😀e\u{301}", FontId(1), 7.5, 18.0);

        assert_eq!(layout.scalar_len(), 3);
        assert_eq!(layout.run.glyphs[1].source_start, 1);
        assert_eq!(layout.run.glyphs[2].x, 15.0);
        assert_eq!(layout.bounds.width, 22.5);
    }

    #[test]
    fn layout_handles_empty_text_and_out_of_range_cursor() {
        let layout = SingleLineLayout::new("", FontId(0), 10.0, 20.0);

        assert_eq!(layout.cursor_x(99), 0.0);
        assert_eq!(layout.hit_test(99.0), 0);
        assert_eq!(layout.bounds.height, 20.0);
    }

    #[test]
    fn hit_testing_snaps_to_nearest_scalar_boundary() {
        let layout = SingleLineLayout::new("abc", FontId(0), 10.0, 20.0);

        assert_eq!(layout.hit_test(4.9), 0);
        assert_eq!(layout.hit_test(5.0), 1);
        assert_eq!(layout.hit_test(29.0), 3);
        assert_eq!(layout.hit_test(-1.0), 0);
    }

    #[test]
    fn reversed_selection_is_normalized() {
        let layout = SingleLineLayout::new("abcd", FontId(0), 10.0, 20.0);

        assert_eq!(layout.selection_bounds(3, 1).x, 10.0);
        assert_eq!(layout.selection_bounds(3, 1).width, 20.0);
    }
}
