use webrender::api::{LayoutPoint, GlyphInstance, PrimitiveInfo, FontInstanceKey};
use rusttype::{Scale, GlyphId, VMetrics};

use render::RenderBuilder;
use text_layout::{self, Wrap, Align};
use resources::resources;
use resources::font::FontDescriptor;
use geometry::{Size, Rect, RectExt, Vector};
use render;
use widget::draw::Draw;
use color::*;

const DEBUG_LINE_BOUNDS: bool = false;

component_style!{pub struct TextState<name="text", style=TextStyle> {
    text: String = String::from(""),
    font: FontDescriptor = FontDescriptor::from_family("Verdana"),
    font_size: f32 = 24.0,
    text_color: Color = BLACK,
    background_color: Color = TRANSPARENT,
    wrap: Wrap = Wrap::Whitespace,
    align: Align = Align::Start,
}}

impl TextStyle {
    pub fn from_text(text: &str) -> Self {
        Self {
            text: Some(String::from(text)),
            ..Self::default()
        }
    }
}

impl TextState {
    pub fn measure(&self) -> Size {
        let line_height = self.line_height();
        let mut resources = resources();
        let font = resources.font_loader.get_font(&self.font).unwrap();
        Size::from_untyped(&text_layout::get_text_size(
            &self.text,
            &font.info,
            self.font_size,
            line_height,
            self.wrap))
    }
    pub fn min_height(&self) -> f32 {
        self.line_height()
    }
    pub fn line_height(&self) -> f32 {
        self.font_size + self.v_metrics().line_gap
    }
    pub fn text_fits(&self, text: &str, bounds: Rect) -> bool {
        let line_height = self.line_height();
        let mut resources = resources();
        let font = resources.font_loader.get_font(&self.font).unwrap();
        let height = text_layout::get_text_height(
            text,
            &font.info,
            self.font_size,
            line_height,
            self.wrap,
            bounds.width());
        height <= bounds.height()
    }
    fn get_line_rects(&self, bounds: Rect) -> Vec<Rect> {
        let line_height = self.line_height();
        let mut resources = resources();
        let font = resources.font_loader.get_font(&self.font).unwrap();
        text_layout::get_line_rects(
            &self.text,
            bounds.to_untyped(),
            &font.info,
            self.font_size,
            line_height,
            self.wrap,
            self.align).iter().map(|rect| Rect::from_untyped(rect)).collect()
    }
    fn position_glyphs(&self, bounds: Rect) -> Vec<GlyphInstance> {
        let line_height = self.line_height();
        let descent = self.v_metrics().descent;
        let mut resources = resources();
        let font = resources.font_loader.get_font(&self.font).unwrap();
        text_layout::get_positioned_glyphs(
            &self.text,
            bounds.to_untyped(),
            &font.info,
            self.font_size,
            line_height,
            self.wrap,
            self.align).iter().map(|glyph| {
                let position = glyph.position();
                GlyphInstance {
                    index: glyph.id().0,
                    point: LayoutPoint::new(position.x, position.y + descent),
                }
            }).collect()
    }
    fn font_instance_key(&self) -> FontInstanceKey {
        *resources().font_loader.get_font_instance(&self.font, self.font_size).unwrap()
    }
    fn v_metrics(&self) -> VMetrics {
        let mut resources = resources();
        let font = resources.font_loader.get_font(&self.font).unwrap();
        font.info.v_metrics(Scale::uniform(self.font_size))
    }
}

impl Draw for TextState {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        let glyphs = self.position_glyphs(bounds);
        if DEBUG_LINE_BOUNDS {
            let line_rects = self.get_line_rects(bounds);
            let v_metrics = self.v_metrics();
            let mut resources = resources();
            let font = resources.font_loader.get_font(&self.font).unwrap();
            for mut rect in line_rects {
                render::draw_rect_outline(rect, CYAN, renderer);
                rect.origin.y = rect.bottom() + v_metrics.descent;
                rect.size.height = 1.0;
                render::draw_rect_outline(rect, RED, renderer);
            }
            let scale = Scale::uniform(self.font_size);
            for glyph in &glyphs {
                let scaled_glyph = font.info.glyph(GlyphId(glyph.index)).unwrap().scaled(scale);
                if let Some(rect) = scaled_glyph.exact_bounding_box() {
                    let origin = glyph.point.to_vector() + Vector::new(0.0, -1.0);
                    let rect = Rect::from_rusttype(rect).translate(&origin);
                    render::draw_rect_outline(rect, BLUE, renderer);
                }
            }
        }
        if self.background_color != TRANSPARENT {
            renderer.builder.push_rect(&PrimitiveInfo::new(bounds.clone()), self.background_color.into());
        }
        let key = self.font_instance_key();
        let info = PrimitiveInfo::new(bounds);
        renderer.builder.push_text(
            &info,
            &glyphs,
            key,
            self.text_color.into(),
            None,
        );
    }
}
