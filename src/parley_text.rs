use parley::{
    Alignment, AlignmentOptions, FontContext, LayoutContext, PositionedLayoutItem, StyleProperty,
};

use crate::text::{
    FontId, GlyphPosition, GlyphRun, SingleLineLayout, TextBounds, TextLayoutProvider, TextMetrics,
};

pub struct ParleyGlyphRun {
    pub font: parley::FontData,
    pub glyphs: Vec<GlyphPosition>,
}

pub struct ParleyTextEngine {
    font_context: FontContext,
    layout_context: LayoutContext,
}

impl ParleyTextEngine {
    pub fn new() -> Self {
        Self {
            font_context: FontContext::new(),
            layout_context: LayoutContext::new(),
        }
    }

    pub fn layout_single_line(
        &mut self,
        text: &str,
        font: FontId,
        font_size: f32,
    ) -> SingleLineLayout {
        let font_size = if font_size.is_finite() && font_size > 0.0 {
            font_size
        } else {
            1.0
        };
        let mut builder =
            self.layout_context
                .ranged_builder(&mut self.font_context, text, 1.0, true);
        builder.push_default(StyleProperty::FontSize(font_size));

        let mut layout = builder.build(text);
        layout.break_all_lines(None);
        layout.align(Alignment::Start, AlignmentOptions::default());

        let line = layout.get(0);
        let (metrics, glyphs) = line
            .map(|line| {
                let metrics = *line.metrics();
                let mut glyphs = Vec::new();
                for item in line.items() {
                    let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                        continue;
                    };
                    let source_ranges = glyph_run.run().visual_clusters().flat_map(|cluster| {
                        let range = cluster.text_range();
                        std::iter::repeat((
                            scalar_offset(text, range.start),
                            scalar_offset(text, range.end),
                        ))
                        .take(cluster.glyphs().count())
                    });
                    for (glyph, (source_start, source_end)) in
                        glyph_run.positioned_glyphs().zip(source_ranges)
                    {
                        glyphs.push(GlyphPosition {
                            glyph_id: glyph.id,
                            source_start,
                            source_end,
                            x: glyph.x,
                            y: glyph.y,
                            advance: glyph.advance,
                        });
                    }
                }
                (metrics, glyphs)
            })
            .unwrap_or_default();
        let scalar_count = text.chars().count();
        let mut caret_positions = vec![0.0; scalar_count + 1];
        for glyph in &glyphs {
            let start = glyph.source_start.min(scalar_count);
            let end = glyph.source_end.min(scalar_count);
            for caret in &mut caret_positions[start..end] {
                *caret = glyph.x;
            }
            if end > start {
                caret_positions[end] = glyph.x + glyph.advance;
            }
        }

        SingleLineLayout::from_parts(
            text.to_owned(),
            GlyphRun { font, glyphs },
            TextMetrics {
                ascent: metrics.ascent,
                descent: metrics.descent,
                line_height: metrics.line_height.max(1.0),
            },
            TextBounds {
                x: 0.0,
                y: 0.0,
                width: layout.full_width(),
                height: layout.height().max(1.0),
            },
            caret_positions,
        )
    }

    pub fn layout_single_line_runs(&mut self, text: &str, font_size: f32) -> Vec<ParleyGlyphRun> {
        let font_size = if font_size.is_finite() && font_size > 0.0 {
            font_size
        } else {
            1.0
        };
        let mut builder =
            self.layout_context
                .ranged_builder(&mut self.font_context, text, 1.0, true);
        builder.push_default(StyleProperty::FontSize(font_size));

        let mut layout = builder.build(text);
        layout.break_all_lines(None);
        layout.align(Alignment::Start, AlignmentOptions::default());

        layout
            .get(0)
            .into_iter()
            .flat_map(|line| line.items())
            .filter_map(|item| {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    return None;
                };
                let source_ranges = glyph_run.run().visual_clusters().flat_map(|cluster| {
                    let range = cluster.text_range();
                    std::iter::repeat((
                        scalar_offset(text, range.start),
                        scalar_offset(text, range.end),
                    ))
                    .take(cluster.glyphs().count())
                });
                let glyphs = glyph_run
                    .positioned_glyphs()
                    .zip(source_ranges)
                    .map(|(glyph, (source_start, source_end))| GlyphPosition {
                        glyph_id: glyph.id,
                        source_start,
                        source_end,
                        x: glyph.x,
                        y: glyph.y,
                        advance: glyph.advance,
                    })
                    .collect();
                Some(ParleyGlyphRun {
                    font: glyph_run.run().font().clone(),
                    glyphs,
                })
            })
            .collect()
    }
}

impl Default for ParleyTextEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TextLayoutProvider for ParleyTextEngine {
    fn layout_single_line(&mut self, text: &str, font: FontId, font_size: f32) -> SingleLineLayout {
        ParleyTextEngine::layout_single_line(self, text, font, font_size)
    }
}

fn scalar_offset(text: &str, byte_offset: usize) -> usize {
    text.get(..byte_offset)
        .map(|prefix| prefix.chars().count())
        .unwrap_or_else(|| text.chars().count())
}

#[cfg(test)]
mod tests {
    use super::ParleyTextEngine;
    use crate::text::FontId;

    #[test]
    fn shaped_line_exposes_metrics_and_scalar_cluster_ranges() {
        let mut engine = ParleyTextEngine::new();
        let layout = engine.layout_single_line("cafe\u{301}", FontId(4), 16.0);

        assert_eq!(layout.text, "cafe\u{301}");
        assert!(!layout.run.glyphs.is_empty());
        assert_eq!(layout.scalar_len(), 5);
        assert!(layout.bounds.width > 0.0);
        assert!(layout.metrics.line_height > 0.0);
        assert!(
            layout
                .run
                .glyphs
                .iter()
                .all(|glyph| { glyph.source_start <= glyph.source_end && glyph.source_end <= 5 })
        );
    }
}
