use std::{num::NonZeroU32, ffi::CString};

use cosmic_text::{FontSystem, TextBuffer, TextMetrics, Attrs, Family, TextBufferLine, AttrsList, SwashCache};
use femtovg::{Paint, RenderTarget};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, dpi::PhysicalSize, event::{WindowEvent, Event}};
use glutin::{prelude::*, display::{Display, DisplayApiPreference}, config::ConfigTemplate, context::{ContextAttributesBuilder, ContextApi}, surface::{SurfaceAttributesBuilder, WindowSurface}};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vector text")
        .with_inner_size(PhysicalSize::new(1280, 800))
        .build(&event_loop)
        .unwrap();

    let display = unsafe { Display::new(window.raw_display_handle(), DisplayApiPreference::Egl) }.unwrap();
    let config = unsafe { display.find_configs(ConfigTemplate::default()) }.unwrap().next().unwrap();
    let context = unsafe {
            display.create_context(
                &config,
                &ContextAttributesBuilder::new()
                    .with_context_api(ContextApi::Gles(None))
                    .build(Some(window.raw_window_handle()))
            )
        }
        .unwrap();
    let surface = unsafe { display.create_window_surface(
        &config,
        &SurfaceAttributesBuilder::<WindowSurface>::new()
            .build(window.raw_window_handle(), NonZeroU32::new(1280).unwrap(), NonZeroU32::new(800).unwrap())
    ) }
    .unwrap();
    let context = context.make_current(&surface).unwrap();

    let renderer = unsafe {
        femtovg::renderer::OpenGl::new_from_function(|name| {
            let str = CString::new(name).unwrap();
            display.get_proc_address(str.as_c_str())
        })
    }.unwrap();
    let mut canvas = femtovg::Canvas::new(renderer).unwrap();

    // Text setup
    let font_system = FontSystem::new();
    let mut image_buffer = TextBuffer::new(
        &font_system,
        TextMetrics::new(16, 18)
    );
    image_buffer.set_size(500, 200);

    let attrs = Attrs::new();
    let serif_attrs = attrs.family(Family::Serif);
    image_buffer.lines.clear();
    image_buffer.lines.push(TextBufferLine::new("I am an image", AttrsList::new(serif_attrs)));
    image_buffer.shape_until_scroll();

    let image = canvas.create_image_empty(
        500,
        200,
        femtovg::PixelFormat::Rgba8,
        femtovg::ImageFlags::FLIP_Y,
    )
    .unwrap();

    // For now, render to an image and upload
    let mut swash_cache = SwashCache::new(&font_system);
    canvas.set_render_target(RenderTarget::Image(image));
    image_buffer.draw(&mut swash_cache, cosmic_text::Color::rgb(128, 128, 128), |x, y, w, h, color| {
        let mut path = femtovg::Path::new();
        path.rect(x as _, y as _, w as _, h as _);
        let mut paint = femtovg::Paint::default();
        paint.set_color(femtovg::Color::rgba(color.r(), color.g(), color.b(), color.a()));
        canvas.fill_path(&mut path, paint);
    });

    canvas.set_render_target(RenderTarget::Screen);

    let mut vector_buffer = TextBuffer::new(
        &font_system,
        TextMetrics::new(16, 18)
    );
    vector_buffer.set_size(500, 200);

    let attrs = Attrs::new();
    let serif_attrs = attrs.family(Family::Serif);
    vector_buffer.lines.clear();
    vector_buffer.lines.push(TextBufferLine::new("I am a vector", AttrsList::new(serif_attrs)));
    vector_buffer.shape_until_scroll();

    event_loop.run(move |event, _, exit| {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::Resized(size) => {
                        surface.resize(
                            &context,
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap()
                        );
                        canvas.set_size(size.width, size.height, 1.0);
                    },
                    WindowEvent::CloseRequested => *exit = ControlFlow::Exit,
                    _ => (),
                }
            },

            Event::MainEventsCleared => {
                window.request_redraw();
            },

            Event::RedrawRequested(_) => {
                canvas.clear_rect(
                    0,
                    0,
                    window.inner_size().width,
                    window.inner_size().height,
                    femtovg::Color::white()
                );

                // Draw the image rendered text
                let paint = Paint::image(
                    image,
                    0.0,
                    0.0,
                    500.0,
                    200.0,
                    0.0,
                    1.0,
                );
                let mut path = femtovg::Path::new();
                path.rect(0., 0., 500., 200.);
                canvas.fill_path(&mut path, paint);


                // TODO: Draw the vector rendered text

                canvas.flush();
                surface.swap_buffers(&context).unwrap();
            },

            Event::RedrawEventsCleared => {},

            _ => ()
        }
    });
}
