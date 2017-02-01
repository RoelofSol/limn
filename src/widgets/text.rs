use std::collections::BTreeSet;

use graphics;
use graphics::types::Color;

use backend::glyph::{self, GlyphCache};
use backend::gfx::ImageSize;

use text::{self, Wrap};
use resources::{Id, resources};
use util::{self, Dimensions, Align, Scalar};
use color::*;
use widget::{Drawable, WidgetStyle, StyleArgs, DrawArgs, Property, PropSet};
use widget::style::Value;

pub fn text_drawable(style: TextStyle) -> Drawable {
    let draw_state = TextDrawState::new_style(&style);
    let mut drawable = Drawable::new(Box::new(draw_state), draw_text);
    drawable.style = Some(WidgetStyle::new(Box::new(style), apply_text_style));
    drawable
}

pub struct TextDrawState {
    pub text: String,
    pub font_id: Id,
    pub font_size: Scalar,
    pub text_color: Color,
    pub background_color: Color,
}

pub fn apply_text_style(args: StyleArgs) {
    let state: &mut TextDrawState = args.state.downcast_mut().unwrap();
    let style: &TextStyle = args.style.downcast_ref().unwrap();
    let props = args.props;
    state.text = style.text.from_props(props);
    state.font_id = style.font_id.from_props(props);
    state.font_size = style.font_size.from_props(props);
    state.text_color = style.text_color.from_props(props);
    state.background_color = style.background_color.from_props(props);
}

#[derive(Clone)]
pub struct TextStyle {
    pub text: Value<String>,
    pub font_id: Value<Id>,
    pub font_size: Value<Scalar>,
    pub text_color: Value<Color>,
    pub background_color: Value<Color>,
}
impl TextStyle {
    pub fn with_text(&mut self, text: &str) -> &mut Self {
        self.text = Value::Single(text.to_owned());
        self
    }
    pub fn with_text_color(&mut self, text_color: Color) -> &mut Self {
        self.text_color = Value::Single(text_color);
        self
    }
    pub fn with_background_color(&mut self, background_color: Color) -> &mut Self {
        self.background_color = Value::Single(background_color);
        self
    }
}

pub fn measure_dims_no_wrap(drawable: &Drawable) -> Dimensions {
    let draw_state: &TextDrawState = drawable.state();
    draw_state.measure_dims_no_wrap()
}

impl TextDrawState {
    pub fn new_default(text: String, font_id: Id) -> Self {
        TextDrawState {
            text: text,
            font_id: font_id,
            font_size: 24.0,
            text_color: BLACK,
            background_color: TRANSPARENT,
        }
    }
    pub fn new_style(style: &TextStyle) -> Self {
        TextDrawState::new(style.text.default(),
                           style.font_id.default(),
                           style.font_size.default(),
                           style.text_color.default(),
                           style.background_color.default())
    }
    pub fn new(text: String,
               font_id: Id,
               font_size: Scalar,
               text_color: Color,
               background_color: Color)
               -> Self {
        TextDrawState {
            text: text,
            font_id: font_id,
            font_size: font_size,
            text_color: text_color,
            background_color: background_color,
        }
    }
    pub fn measure_dims_no_wrap(&self) -> Dimensions {
        let res = resources();
        let font = res.fonts.get(self.font_id).unwrap();
        text::get_text_dimensions(&self.text,
                                  font,
                                  self.font_size,
                                  self.font_size * 1.25,
                                  Align::Start,
                                  Align::Start)
    }
    pub fn measure_height_wrapped(&self, width: Scalar) -> Scalar {
        let res = resources();
        let font = res.fonts.get(self.font_id).unwrap();
        text::get_text_height(&self.text,
                              font,
                              self.font_size,
                              self.font_size * 1.25,
                              width,
                              Wrap::Character,
                              Align::Start,
                              Align::Start)
    }
}

pub fn draw_text(draw_args: DrawArgs) {

    let DrawArgs { state, bounds, glyph_cache, context, graphics, .. } = draw_args;
    let state: &TextDrawState = state.downcast_ref().unwrap();

    graphics::Rectangle::new(state.background_color)
        .draw(bounds, &context.draw_state, context.transform, graphics);

    let &mut GlyphCache { texture: ref mut text_texture_cache,
                          cache: ref mut glyph_cache,
                          ref mut vertex_data } = glyph_cache;

    let res = resources();
    let font = res.fonts.get(state.font_id).unwrap();
    let line_wrap = Wrap::Character;

    let positioned_glyphs = &text::get_positioned_glyphs(&state.text,
                                                         bounds,
                                                         font,
                                                         state.font_size,
                                                         state.font_size * 1.25,
                                                         line_wrap,
                                                         Align::Start,
                                                         Align::Start);

    // Queue the glyphs to be cached.
    for glyph in positioned_glyphs.iter() {
        glyph_cache.queue_glyph(state.font_id.index(), glyph.clone());
    }

    // Cache the glyphs within the GPU cache.
    glyph_cache.cache_queued(|rect, data| {
            glyph::cache_queued_glyphs(graphics, text_texture_cache, rect, data, vertex_data)
        })
        .unwrap();

    let tex_dim = {
        let (tex_w, tex_h) = text_texture_cache.get_size();
        Dimensions {
            width: tex_w as f64,
            height: tex_h as f64,
        }
    };

    let rectangles = positioned_glyphs.into_iter()
        .filter_map(|g| glyph_cache.rect_for(state.font_id.index(), g).ok().unwrap_or(None))
        .map(|(uv_rect, screen_rect)| {
            (util::map_rect_i32(screen_rect), util::map_rect_f32(uv_rect) * tex_dim)
        });
    // A re-usable buffer of rectangles describing the glyph's screen and texture positions.
    let mut glyph_rectangles = Vec::new();
    glyph_rectangles.extend(rectangles);
    graphics::image::draw_many(&glyph_rectangles,
                               state.text_color,
                               text_texture_cache,
                               &context.draw_state,
                               context.transform,
                               graphics);
}
