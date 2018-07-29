#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
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
    AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture,
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

#[derive(Clone, Debug)]
pub struct Section {
    font: FontId,
    color: [f32; 4],
    range: Range<usize>,
}

impl<'font> GlyphBrush<'font> {
    pub fn new<'a>(
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

    pub fn queue_glyphs<I>(&mut self, glyphs: I, font: FontId, color: [f32; 4]) -> Section
    where
        I: IntoIterator<Item = PositionedGlyph<'font>>,
    {
        let old_len = self.glyphs.len();
        self.glyphs.extend(glyphs);
        let range = old_len..self.glyphs.len();
        Section { range, font, color }
    }

    pub fn cache_sections<'a, I>(
        &mut self,
        queue: &Arc<Queue>,
        sections: I,
    ) -> Result<Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>>
    where
        I: Iterator<Item = &'a Section> + Clone,
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

    pub fn draw<'a, I>(
        &mut self,
        mut cmd: AutoCommandBufferBuilder,
        sections: I,
        transform: [[f32; 4]; 4],
        dims: [u32; 2],
    ) -> Result<AutoCommandBufferBuilder>
    where
        I: IntoIterator<Item = &'a Section>,
    {
        for section in sections {
            cmd = self.draw.draw(
                cmd,
                &self.glyphs[section.range.clone()],
                section,
                &self.cache,
                transform,
                dims,
            )?;
        }
        Ok(cmd)
    }
}
