use std::sync::Arc;

use rusttype::{point, Rect};
use vulkano::buffer::{BufferUsage, CpuBufferPool};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::viewport::{Scissor, Viewport};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};

use {Error, GlyphData, GpuCache};

struct Vertex {
    tl: [f32; 2],
    br: [f32; 2],
    tex_tl: [f32; 2],
    tex_br: [f32; 2],
    color: [f32; 4],
    z: f32,
}

impl_vertex! { Vertex, tl, br, tex_tl, tex_br, color, z }

#[allow(unused)]
mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[path = "shader/vert.glsl"]
    struct Dummy;
}

#[allow(unused)]
mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[path = "shader/frag.glsl"]
    struct Dummy;
}

type Pipeline = Arc<
    GraphicsPipeline<
        SingleBufferDefinition<Vertex>,
        Box<PipelineLayoutAbstract + Send + Sync>,
        Arc<RenderPassAbstract + Send + Sync>,
    >,
>;

pub(crate) struct Draw {
    pipe: Pipeline,
    vbuf: CpuBufferPool<Vertex>,
    ubuf: CpuBufferPool<vs::ty::Data>,
    pool: FixedSizeDescriptorSetsPool<Pipeline>,
    sampler: Arc<Sampler>,
}

impl Draw {
    pub(crate) fn new(
        device: &Arc<Device>,
        subpass: Subpass<Arc<RenderPassAbstract + Send + Sync>>,
    ) -> Result<Self, Error> {
        let vs = vs::Shader::load(Arc::clone(device))?;
        let fs = fs::Shader::load(Arc::clone(device))?;

        let pipe = Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(subpass)
            .build(Arc::clone(device))?);

        let vbuf = CpuBufferPool::new(Arc::clone(device), BufferUsage::vertex_buffer());
        let ubuf = CpuBufferPool::new(Arc::clone(device), BufferUsage::uniform_buffer());

        let pool = FixedSizeDescriptorSetsPool::new(Arc::clone(&pipe), 0);

        let sampler = Sampler::new(
            Arc::clone(device),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0,
            1.0,
            0.0,
            0.0,
        )?;

        Ok(Draw {
            pipe,
            vbuf,
            ubuf,
            pool,
            sampler,
        })
    }

    pub(crate) fn draw(
        &mut self,
        cmd: AutoCommandBufferBuilder,
        data: &GlyphData,
        cache: &GpuCache,
        transform: [[f32; 4]; 4],
        [w, h]: [u32; 2],
    ) -> Result<AutoCommandBufferBuilder, Error> {
        let vertices = text_vertices(data, cache, (w as f32, h as f32))?;
        let vbuf = self.vbuf.chunk(vertices)?;
        let ubuf = self.ubuf.next(vs::ty::Data { transform })?;
        let set = self.pool
            .next()
            .add_buffer(ubuf)?
            .add_sampled_image(Arc::clone(cache.image()), Arc::clone(&self.sampler))?
            .build()?;
        let state = DynamicState {
            line_width: None,
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [w as f32, h as f32],
                depth_range: 0.0..1.0,
            }]),
            scissors: Some(vec![Scissor {
                origin: [data.bounds.min.x as i32, data.bounds.min.y as i32],
                dimensions: [data.bounds.width() as u32, data.bounds.height() as u32],
            }]),
        };
        Ok(cmd.draw(Arc::clone(&self.pipe), state, vbuf, set, ())?)
    }
}

fn text_vertices<'font>(
    data: &GlyphData,
    cache: &GpuCache<'font>,
    (screen_width, screen_height): (f32, f32),
) -> Result<Vec<Vertex>, Error> {
    // max 1 vertex per glyph
    let mut vertices = Vec::with_capacity(data.glyphs.len());

    /*
    let gl_bounds = Rect {
        min: point(
            2.0 * (data.bounds.min.x / screen_width - 0.5),
            2.0 * (0.5 - data.bounds.min.y / screen_height),
        ),
        max: point(
            2.0 * (data.bounds.max.x / screen_width - 0.5),
            2.0 * (0.5 - data.bounds.max.y / screen_height),
        ),
    };
*/

    for gly in &data.glyphs {
        if let Some((mut uv_rect, screen_rect)) = cache.rect_for(data.font, gly)? {
            if screen_rect.min.x as f32 > data.bounds.max.x
                || screen_rect.min.y as f32 > data.bounds.max.y
                || data.bounds.min.x > screen_rect.max.x as f32
                || data.bounds.min.y > screen_rect.max.y as f32
            {
                // glyph is totally outside the bounds
                continue;
            }

            let mut gl_rect = Rect {
                min: point(
                    2.0 * (screen_rect.min.x as f32 / screen_width - 0.5),
                    2.0 * (0.5 - screen_rect.min.y as f32 / screen_height),
                ),
                max: point(
                    2.0 * (screen_rect.max.x as f32 / screen_width - 0.5),
                    2.0 * (0.5 - screen_rect.max.y as f32 / screen_height),
                ),
            };

            /*
            // handle overlapping bounds, modify uv_rect to preserve texture aspect
            if gl_rect.max.x > gl_bounds.max.x {
                let old_width = gl_rect.width();
                gl_rect.max.x = gl_bounds.max.x;
                uv_rect.max.x = uv_rect.min.x + uv_rect.width() * gl_rect.width() / old_width;
            }
            if gl_rect.min.x < gl_bounds.min.x {
                let old_width = gl_rect.width();
                gl_rect.min.x = gl_bounds.min.x;
                uv_rect.min.x = uv_rect.max.x - uv_rect.width() * gl_rect.width() / old_width;
            }
            // note: y access is flipped gl compared with screen,
            // texture is not flipped (ie is a headache)
            if gl_rect.max.y < gl_bounds.max.y {
                let old_height = gl_rect.height();
                gl_rect.max.y = gl_bounds.max.y;
                uv_rect.max.y = uv_rect.min.y + uv_rect.height() * gl_rect.height() / old_height;
            }
            if gl_rect.min.y > gl_bounds.min.y {
                let old_height = gl_rect.height();
                gl_rect.min.y = gl_bounds.min.y;
                uv_rect.min.y = uv_rect.max.y - uv_rect.height() * gl_rect.height() / old_height;
            }
*/

            vertices.push(Vertex {
                tl: [gl_rect.min.x, gl_rect.max.y],
                br: [gl_rect.max.x, gl_rect.min.y],
                tex_tl: [uv_rect.min.x, uv_rect.max.y],
                tex_br: [uv_rect.max.x, uv_rect.min.y],
                color: data.color,
                z: data.z,
            });
        }
    }
    Ok(vertices)
}
