#[macro_use]
extern crate vulkano;
extern crate env_logger;
extern crate rusttype;
extern crate vulkano_glyph;
extern crate vulkano_win;
extern crate winit;

use std::env;
use std::fs::File;
use std::io::Read;

use rusttype::{point, Font, Scale};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::Device;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::instance::Instance;
use vulkano::swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::sync::now;
use vulkano::sync::GpuFuture;
use vulkano_glyph::GlyphBrush;
use vulkano_win::VkSurfaceBuild;

use std::mem;
use std::sync::Arc;

fn main() {
    env_logger::init();

    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("failed to create Vulkan instance")
    };

    let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");
    println!(
        "Using device: {} (type: {:?})",
        physical.name(),
        physical.ty()
    );

    let mut events_loop = winit::EventsLoop::new();
    let surface = winit::WindowBuilder::new()
        .with_dimensions(winit::dpi::LogicalSize::new(1000.0, 1000.0))
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    let queue = physical
        .queue_families()
        .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        .expect("couldn't find a graphical queue family");

    let (device, mut queues) = {
        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            ..vulkano::device::DeviceExtensions::none()
        };

        Device::new(
            physical,
            physical.supported_features(),
            &device_ext,
            [(queue, 0.5)].iter().cloned(),
        ).expect("failed to create device")
    };

    let queue = queues.next().unwrap();
    let mut dimensions;
    let (mut swapchain, mut images) = {
        let caps = surface
            .capabilities(physical)
            .expect("failed to get surface capabilities");

        dimensions = caps.current_extent.unwrap_or([1024, 768]);
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

        Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            caps.supported_usage_flags,
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            true,
            None,
        ).expect("failed to create swapchain")
    };

    let render_pass = Arc::new(
        single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    ).unwrap(),
    );

    let mut font_data = Vec::new();
    File::open(env::args_os().nth(1).unwrap())
        .expect("No font specified")
        .read_to_end(&mut font_data)
        .unwrap();
    let font = Font::from_bytes(font_data).unwrap();

    let mut glyph_brush = GlyphBrush::new(
        &device,
        Subpass::from(
            render_pass.clone() as Arc<RenderPassAbstract + Send + Sync>,
            0,
        ).unwrap(),
    ).unwrap();

    let mut framebuffers: Option<Vec<Arc<vulkano::framebuffer::Framebuffer<_, _>>>> = None;
    let mut recreate_swapchain = false;
    let mut previous_frame_end = Box::new(now(device.clone())) as Box<GpuFuture>;

    let section1 = glyph_brush.queue_glyphs(
        font.layout("Hello, world!", Scale::uniform(100.0), point(0.0, 100.0)),
        0,
        [0.0, 0.0, 1.0, 1.0],
    );
    let section2 = glyph_brush.queue_glyphs(
        font.layout("Lower!", Scale::uniform(100.0), point(0.0, 150.0)),
        0,
        [0.0, 1.0, 0.0, 1.0],
    );

    let mut copy_future = glyph_brush
        .cache_sections(&queue, vec![&section1, &section2].iter().cloned())
        .unwrap()
        .map(|f| Box::new(f) as Box<GpuFuture + Send + Sync>);

    loop {
        previous_frame_end.cleanup_finished();

        if recreate_swapchain {
            dimensions = surface
                .capabilities(physical)
                .expect("failed to get surface capabilities")
                .current_extent
                .unwrap();

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => {
                    continue;
                }
                Err(err) => panic!("{:?}", err),
            };

            mem::replace(&mut swapchain, new_swapchain);
            mem::replace(&mut images, new_images);

            framebuffers = None;

            recreate_swapchain = false;
        }

        if framebuffers.is_none() {
            let new_framebuffers = Some(
                images
                    .iter()
                    .map(|image| {
                        Arc::new(
                            Framebuffer::start(render_pass.clone())
                                .add(image.clone())
                                .unwrap()
                                .build()
                                .unwrap(),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            mem::replace(&mut framebuffers, new_framebuffers);
        }

        let copy_future = copy_future
            .take()
            .unwrap_or_else(|| Box::new(now(device.clone())));

        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    continue;
                }
                Err(err) => panic!("{:?}", err),
            };

        let command_buffer =
            AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .unwrap()
                .begin_render_pass(
                    framebuffers.as_ref().unwrap()[image_num].clone(),
                    false,
                    vec![[1.0, 1.0, 1.0, 1.0].into()],
                )
                .unwrap();
        let command_buffer = glyph_brush
            .draw(
                command_buffer,
                &section1,
                [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
                dimensions,
            )
            .unwrap();
        let command_buffer = glyph_brush
            .draw(
                command_buffer,
                &section2,
                [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
                dimensions,
            )
            .unwrap();

        let command_buffer = command_buffer.end_render_pass().unwrap().build().unwrap();

        let future = previous_frame_end
            .join(copy_future)
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                previous_frame_end = Box::new(future) as Box<_>;
            }
            Err(vulkano::sync::FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<_>;
            }
            Err(e) => {
                println!("{:?}", e);
                previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<_>;
            }
        }

        let mut done = false;
        events_loop.poll_events(|ev| match ev {
            winit::Event::WindowEvent {
                event: winit::WindowEvent::CloseRequested,
                ..
            } => done = true,
            _ => (),
        });
        if done {
            return;
        }
    }
}
