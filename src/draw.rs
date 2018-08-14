use std::iter;
use std::sync::Arc;

use rusttype::PositionedGlyph;
use vulkano::buffer::{BufferUsage, CpuBufferPool};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DrawIndirectCommand, DynamicState};
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::vertex::SingleInstanceBufferDefinition;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};

use {Error, GpuCache, Section};

#[derive(Debug)]
struct Vertex {
    tl: [f32; 2],
    br: [f32; 2],
    tex_tl: [f32; 2],
    tex_br: [f32; 2],
    color: [f32; 4],
}

impl_vertex! { Vertex, tl, br, tex_tl, tex_br, color }

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
        SingleInstanceBufferDefinition<Vertex>,
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
    ibuf: CpuBufferPool<DrawIndirectCommand>,
}

impl Draw {
    pub(crate) fn new(
        device: &Arc<Device>,
        subpass: Subpass<Arc<RenderPassAbstract + Send + Sync>>,
    ) -> Result<Self, Error> {
        let vs = vs::Shader::load(Arc::clone(device))?;
        let fs = fs::Shader::load(Arc::clone(device))?;

        let pipe = Arc::new(
            GraphicsPipeline::start()
                .vertex_input(SingleInstanceBufferDefinition::<Vertex>::new())
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_strip()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(subpass)
                .build(Arc::clone(device))?,
        );

        let vbuf = CpuBufferPool::new(Arc::clone(device), BufferUsage::vertex_buffer());
        let ubuf = CpuBufferPool::new(Arc::clone(device), BufferUsage::uniform_buffer());
        let ibuf = CpuBufferPool::new(Arc::clone(device), BufferUsage::indirect_buffer());

        let pool = FixedSizeDescriptorSetsPool::new(Arc::clone(&pipe), 0);

        let sampler = Sampler::new(
            Arc::clone(device),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
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
            ibuf,
        })
    }

    pub(crate) fn draw<'font>(
        &mut self,
        cmd: AutoCommandBufferBuilder,
        glyphs: &[PositionedGlyph<'font>],
        section: &Section,
        cache: &GpuCache<'font>,
        dynamic_state: &DynamicState,
        transform: [[f32; 4]; 4],
        [w, h]: [u32; 2],
    ) -> Result<AutoCommandBufferBuilder, Error> {
        let vertices = text_vertices(glyphs, section, cache, (w as f32, h as f32))?;
        let instance_count = vertices.len() as u32;
        let vbuf = self.vbuf.chunk(vertices)?;
        let ubuf = self.ubuf.next(vs::ty::Data { transform })?;
        let ibuf = self.ibuf.chunk(iter::once(DrawIndirectCommand {
            vertex_count: 4,
            instance_count,
            first_vertex: 0,
            first_instance: 0,
        }))?;

        let set = self
            .pool
            .next()
            .add_buffer(ubuf)?
            .add_sampled_image(cache.image(), Arc::clone(&self.sampler))?
            .build()?;

        Ok(cmd.draw_indirect(Arc::clone(&self.pipe), dynamic_state, vbuf, ibuf, set, ())?)
    }
}

fn text_vertices<'font>(
    glyphs: &[PositionedGlyph<'font>],
    data: &Section,
    cache: &GpuCache<'font>,
    (screen_width, screen_height): (f32, f32),
) -> Result<impl ExactSizeIterator<Item = Vertex>, Error> {
    let mut vertices = Vec::with_capacity(glyphs.len());
    for gly in glyphs {
        if let Some((mut uv_rect, screen_rect)) = cache.rect_for(data.font, &gly)? {
            vertices.push(Vertex {
                tl: [
                    to_ndc(screen_rect.min.x, screen_width),
                    to_ndc(screen_rect.min.y, screen_height),
                ],
                br: [
                    to_ndc(screen_rect.max.x, screen_width),
                    to_ndc(screen_rect.max.y, screen_height),
                ],
                tex_tl: [uv_rect.min.x, uv_rect.min.y],
                tex_br: [uv_rect.max.x, uv_rect.max.y],
                color: data.color,
            });
        }
    }
    Ok(vertices.into_iter())
}

fn to_ndc(x: i32, size: f32) -> f32 {
    (2 * x) as f32 / size - 1.0
}
