use std::sync::Arc;
use std::{iter, result};

use rusttype::gpu_cache::{Cache, CacheBuilder, CacheReadErr, CacheWriteErr, TextureCoords};
use rusttype::{PositionedGlyph, Rect};
use vulkano::buffer::CpuBufferPool;
use vulkano::command_buffer::{
    AutoCommandBuffer, AutoCommandBufferBuilder, CommandBuffer, CommandBufferExecFuture,
};
use vulkano::device::{Device, Queue};
use vulkano::format::R8Unorm;
use vulkano::image::{Dimensions, ImageUsage, StorageImage};
use vulkano::sync::NowFuture;

use {FontId, Result};

const INITIAL_WIDTH: u32 = 256;
const INITIAL_HEIGHT: u32 = 256;

/// Wraps `rusttype`'s cache for use with `vulkano`.
pub struct GpuCache<'font> {
    cache: Cache<'font>,
    img: Arc<StorageImage<R8Unorm>>,
    buf: CpuBufferPool<u8>,
}

impl<'font> GpuCache<'font> {
    /// Create a new `GpuCache` for use on the given device.
    pub fn new<'a>(device: &Arc<Device>) -> Result<Self> {
        let (img, cache) = create(device, INITIAL_WIDTH, INITIAL_HEIGHT)?;
        let buf = CpuBufferPool::upload(Arc::clone(device));

        Ok(GpuCache { cache, img, buf })
    }

    /// Overwrite the cache with a new collection of glyphs. If the cache is too small, it
    /// will be resized until it is big enough.
    pub fn cache<I>(
        &mut self,
        queue: &Arc<Queue>,
        glyphs: I,
    ) -> Result<Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>>
    where
        I: IntoIterator<Item = (FontId, PositionedGlyph<'font>)> + Clone,
    {
        let mut result = Ok(None);
        while let Err(write_err) = self.try_cache(queue, glyphs.clone(), &mut result) {
            // Cache too small, grow it and retry.
            let (old_w, old_h) = self.cache.dimensions();
            let (new_w, new_h) = (old_w * 2, old_h * 2);
            info!(
                "Resizing glyph cache from {}×{} to {}×{}. (Reason: {})",
                old_w, old_h, new_w, new_h, write_err
            );
            let (new_img, new_cache) = create(queue.device(), new_w, new_h)?;
            self.img = new_img;
            self.cache = new_cache;
        }

        result.and_then(|cmd| {
            Ok(match cmd {
                Some(cmd) => Some(cmd.build()?.execute(Arc::clone(queue))?),
                None => None,
            })
        })
    }

    fn try_cache<I>(
        &mut self,
        queue: &Arc<Queue>,
        glyphs: I,
        result: &mut Result<Option<AutoCommandBufferBuilder>>,
    ) -> result::Result<(), CacheWriteErr>
    where
        I: IntoIterator<Item = (FontId, PositionedGlyph<'font>)>,
    {
        for (font, gly) in glyphs {
            self.cache.queue_glyph(font, gly);
        }

        let GpuCache { cache, buf, img } = self;
        cache.cache_queued(|rect, data| {
            let cmd = match result {
                Ok(cmd) => cmd.take(),
                Err(_) => return,
            };

            *result = upload(rect, data, queue, cmd, img, buf).map(Some);
        })
    }

    /// Get the coordinates of a glyph on the image.
    pub fn rect_for(
        &self,
        font_id: FontId,
        glyph: &PositionedGlyph,
    ) -> result::Result<Option<TextureCoords>, CacheReadErr> {
        self.cache.rect_for(font_id, glyph)
    }

    /// The GPU image containing cached glyphs.
    pub fn image(&self) -> &Arc<StorageImage<R8Unorm>> {
        &self.img
    }
}

fn create<'font>(
    device: &Arc<Device>,
    width: u32,
    height: u32,
) -> Result<(Arc<StorageImage<R8Unorm>>, Cache<'font>)> {
    let img = StorageImage::with_usage(
        Arc::clone(device),
        Dimensions::Dim2d { width, height },
        R8Unorm,
        ImageUsage {
            transfer_destination: true,
            transfer_source: true,
            sampled: true,
            ..ImageUsage::none()
        },
        iter::empty(),
    )?;

    let cache = CacheBuilder {
        width,
        height,
        ..Default::default()
    }.build();

    Ok((img, cache))
}

fn upload(
    rect: Rect<u32>,
    data: &[u8],
    queue: &Arc<Queue>,
    cmd: Option<AutoCommandBufferBuilder>,
    img: &Arc<StorageImage<R8Unorm>>,
    buf: &CpuBufferPool<u8>,
) -> Result<AutoCommandBufferBuilder> {
    let chunk = buf.chunk(data.iter().cloned())?;

    let cmd = match cmd {
        Some(cmd) => cmd,
        None => AutoCommandBufferBuilder::new(Arc::clone(queue.device()), queue.family())?,
    };

    let cmd = cmd.copy_buffer_to_image_dimensions(
        chunk,
        Arc::clone(img),
        [rect.min.x, rect.min.y, 0],
        [rect.width(), rect.height(), 0],
        0,
        1,
        0,
    )?;

    Ok(cmd)
}
