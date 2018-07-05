#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate id_map;
extern crate rusttype;
#[macro_use]
extern crate log;

mod cache;
mod draw;
mod error;

pub use self::cache::GpuCache;
pub use self::error::{Error, Result};

use std::sync::Arc;

use id_map::{Id, IdMap};
use rusttype::{point, Font, PositionedGlyph, Scale};
use vulkano::command_buffer::{
    AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture,
};
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::sync::NowFuture;

use draw::Draw;

pub struct GlyphBrush<'font> {
    fonts: IdMap<Font<'font>>,
    queue: Vec<GlyphData<'font>>,
    cache: GpuCache<'font>,
    draw: Draw,
}

struct GlyphData<'font> {
    glyphs: Vec<PositionedGlyph<'font>>,
    font: FontId,
    color: [f32; 4],
    z: f32,
}

pub type FontId = Id;

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
            queue: Vec::new(),
            fonts: IdMap::new(),
        })
    }

    pub fn add_font(&mut self, font: Font<'font>) -> FontId {
        self.fonts.insert(font)
    }

    pub fn queue(
        &mut self,
        font: FontId,
        text: &str,
        (x, y): (f32, f32),
        size: f32,
        z: f32,
        color: [f32; 4],
    ) {
        let glyphs = self.fonts[font]
            .layout(text, Scale::uniform(size), point(x, y))
            .map(|gly| gly.standalone())
            .collect();
        self.queue.push(GlyphData {
            font,
            glyphs,
            color,
            z,
        });
    }

    pub fn cache_queued(
        &mut self,
        queue: &Arc<Queue>,
    ) -> Result<Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>> {
        self.cache.cache(
            queue,
            self.queue
                .iter()
                .flat_map(|data| data.glyphs.iter().cloned().map(move |gly| (data.font, gly))),
        )
    }

    pub fn draw(
        &mut self,
        mut cmd: AutoCommandBufferBuilder,
        transform: [[f32; 4]; 4],
        dims: [u32; 2],
    ) -> Result<AutoCommandBufferBuilder> {
        for data in self.queue.drain(..) {
            cmd = self.draw.draw(cmd, &data, &self.cache, transform, dims)?;
        }
        Ok(cmd)
    }
}
