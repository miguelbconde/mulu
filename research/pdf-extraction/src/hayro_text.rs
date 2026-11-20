//! A text extractor on top of hayro, the pure-Rust PDF interpreter: hayro gives each
//! glyph with its Unicode value and position; this module turns glyphs into words and
//! lines. A deliberately simple layout, written for this comparison: glyphs are taken
//! in the order the page draws them, a new line starts when the baseline moves, a
//! space is inserted when the gap between two glyphs is wider than a quarter of the
//! font size, and lines in right-to-left scripts are reversed.

use hayro_interpret::font::{Glyph, GlyphRun};
use hayro_interpret::hayro_cmap::BfString;
use hayro_interpret::hayro_syntax::Pdf;
use hayro_interpret::kurbo::{Affine, BezPath, Point, Rect, Shape};
use hayro_interpret::{
    BlendMode, ClipPath, Context, Device, DrawMode, DrawProps, Image, ImageDrawProps,
    InterpreterCache, InterpreterSettings, SoftMask, interpret_page,
};

pub fn extract(path: &str) -> Result<Vec<String>, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let pdf = Pdf::new(data).map_err(|e| format!("{e:?}"))?;
    let cache = InterpreterCache::new();
    let mut pages = Vec::new();
    for page in pdf.pages().iter() {
        let mut context = Context::new(
            Affine::IDENTITY,
            Rect::new(0.0, 0.0, 1.0, 1.0),
            &cache,
            pdf.xref(),
            InterpreterSettings::default(),
        );
        let mut collector = Collector::default();
        interpret_page(page, &mut context, &mut collector);
        pages.push(layout(&collector.glyphs));
    }
    Ok(pages)
}

struct Item {
    text: String,
    /// Baseline origin, in page units (y grows upwards).
    x: f64,
    y: f64,
    /// Right edge of the glyph's ink, or the origin plus half the size if none.
    right: f64,
    size: f64,
}

#[derive(Default)]
struct Collector {
    glyphs: Vec<Item>,
}

impl Device<'_> for Collector {
    fn draw_path(&mut self, _: &BezPath, _: DrawProps<'_>, _: &DrawMode) {}
    fn push_clip_path(&mut self, _: &ClipPath) {}
    fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'_>>, _: BlendMode) {}
    fn pop_clip(&mut self) {}
    fn pop_transparency_group(&mut self) {}
    fn draw_image(&mut self, _: Image<'_, '_>, _: ImageDrawProps<'_>) {}

    fn draw_glyph_run(&mut self, run: &GlyphRun<'_, '_>, props: DrawProps<'_>, _: &DrawMode) {
        for glyph in run.glyphs() {
            let text = match glyph.as_unicode() {
                Some(BfString::Char(c)) => c.to_string(),
                Some(BfString::String(s)) => s,
                None => continue,
            };
            let transform = props.transform * glyph.transform();
            let origin = transform * Point::ZERO;
            // Glyph space is in font units: 1000 per em (hayro_font::UNITS_PER_EM).
            let unit = transform * Point::new(0.0, 1000.0);
            let size = ((unit.x - origin.x).powi(2) + (unit.y - origin.y).powi(2)).sqrt();
            let ink = match &**glyph {
                Glyph::Outline(outline) => {
                    let bbox = outline.outline().bounding_box();
                    (bbox.width() > 0.0).then(|| transform.transform_rect_bbox(bbox))
                }
                Glyph::Type3(_) => None,
            };
            let right = ink.map_or(origin.x + size * 0.5, |r| r.x1);
            self.glyphs.push(Item { text, x: origin.x, y: origin.y, right, size });
        }
    }
}

fn is_rtl(c: char) -> bool {
    matches!(c as u32, 0x0590..=0x08FF | 0xFB1D..=0xFDFF | 0xFE70..=0xFEFF)
}

fn layout(glyphs: &[Item]) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    let mut previous: Option<&Item> = None;
    for glyph in glyphs {
        if let Some(p) = previous {
            let size = p.size.max(glyph.size).max(1.0);
            let new_line = (glyph.y - p.y).abs() > size * 0.5 || glyph.x < p.x - size * 2.0;
            if new_line {
                lines.push(std::mem::take(&mut line));
            } else if glyph.x - p.right > size * 0.25 && !line.ends_with(' ') && glyph.text != " " {
                line.push(' ');
            }
        }
        if !(glyph.text == " " && (line.is_empty() || line.ends_with(' '))) {
            line.push_str(&glyph.text);
        }
        previous = Some(glyph);
    }
    lines.push(line);
    lines
        .into_iter()
        .map(|line| {
            let rtl = line.chars().filter(|c| is_rtl(*c)).count();
            let letters = line.chars().filter(|c| c.is_alphabetic()).count();
            if rtl * 2 > letters { line.chars().rev().collect() } else { line }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
