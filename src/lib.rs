//! This crate provides integration of `vulkano` with the font caching feature of `rusttype`, and
//! a basic pipeline for drawing text to the screen.

#[macro_use]
extern crate vulkano;
extern crate vulkano_shaders;
extern crate rusttype;
#[macro_use]
extern crate log;

mod cache;
mod draw;
mod error;

pub use self::cache::GpuCache;
pub use self::error::{Error, ErrorKind, Result};

use std::ops::Range;
use std::sync::Arc;

use rusttype::PositionedGlyph;
use vulkano::command_buffer::{
    AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture, DynamicState,
};
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::sync::NowFuture;

use draw::Draw;

/// A unique identifier representing a font. Assigning each `Font` a `FontId`
/// is left to the user.
pub type FontId = usize;

/// Object responsible for drawing text to the screen.
pub struct GlyphBrush<'font> {
    glyphs: Vec<PositionedGlyph<'font>>,
    cache: GpuCache<'font>,
    draw: Draw,
}

/// An index for a range of glyphs with the same colour and font.
#[derive(Clone, Debug)]
pub struct Section {
    font: FontId,
    color: [f32; 4],
    range: Range<usize>,
}

impl<'font> GlyphBrush<'font> {
    /// Create a new `GlyphBrush` for use in the given subpass.
    pub fn new(
        device: &Arc<Device>,
        subpass: Subpass<Arc<RenderPassAbstract + Send + Sync>>,
    ) -> Result<Self> {
        let draw = Draw::new(device, subpass)?;
        let cache = GpuCache::new(device)?;
        Ok(GlyphBrush {
            draw,
            cache,
            glyphs: Vec::new(),
        })
    }

    /// Queue some glyphs for later drawing. The `Section` returned is valid until a later call
    /// to `GlyphBrush::clear`.
    pub fn queue_glyphs<I>(&mut self, glyphs: I, font: FontId, color: [f32; 4]) -> Section
    where
        I: IntoIterator<Item = PositionedGlyph<'font>>,
    {
        let old_len = self.glyphs.len();
        self.glyphs.extend(glyphs);
        let range = old_len..self.glyphs.len();
        Section { range, font, color }
    }

    /// Cache some sections of text. If a future is returned, it should be executed before
    /// drawing those sections. This may overwrite cached sections from previous calls to this
    /// function.
    pub fn cache_sections<'a, I>(
        &mut self,
        queue: &Arc<Queue>,
        sections: I,
    ) -> Result<Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>>
    where
        I: IntoIterator<Item = &'a Section>,
    {
        let glyphs = &self.glyphs;
        self.cache.cache(
            queue,
            sections.into_iter().flat_map(|section| {
                glyphs[section.range.clone()]
                    .iter()
                    .map(move |gly| (section.font, gly.clone()))
            }),
        )
    }

    /// Draw a section of text to the screen. The section should have been previously cached
    /// using `GlyphBrush::cache_sections`.
    pub fn draw<'a, I>(
        &mut self,
        cmd: AutoCommandBufferBuilder,
        sections: I,
        state: &DynamicState,
        transform: [[f32; 4]; 4],
        dims: [f32; 2],
    ) -> Result<AutoCommandBufferBuilder>
    where
        I: IntoIterator<Item = &'a Section>,
    {
        self.draw.draw(
            cmd,
            &self.glyphs,
            sections,
            &self.cache,
            state,
            transform,
            dims,
        )
    }

    /// Clear the internal glyph buffer. This invalidates all `Section` objects created by this
    /// `GlyphBrush`.
    pub fn clear(&mut self) {
        self.glyphs.clear();
    }
}
