use glow::*;

use imgui::im_str;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use imgui_glow_renderer::Renderer;

use std::time::Instant;

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("A fantastic window!");

    let windowed_context = glutin::ContextBuilder::new()
        .build_windowed(wb, &event_loop)
        .unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let gl = unsafe {
        let context = glow::Context::from_loader_function(|s| {
            windowed_context.get_proc_address(s) as *const _
        });
        context
    };

    unsafe {
        use glutin::event::{Event, WindowEvent};
        use glutin::event_loop::ControlFlow;

        let mut changeable = 0.01;
        let mut last_frame = Instant::now();

        let scale_factor = windowed_context.window().scale_factor() as f32;
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            windowed_context.window(),
            HiDpiMode::Default,
        );

        let font_size = 13.0 * scale_factor;
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    size_pixels: font_size,
                    ..imgui::FontConfig::default()
                }),
            }]);

        imgui.io_mut().font_global_scale = (1.0 / scale_factor) as f32;
        let mut style = imgui.style_mut().clone();
        style.window_rounding = 0.0;
        style.window_border_size = 0.0;
        style.colors[imgui::StyleColor::TitleBg as usize] = [1.0, 1.0, 1.0, 1.0];

        let imgui_renderer = Renderer::new(&gl, &mut imgui);

        let mut modifiable_string = imgui::ImString::new("");
        let mut mod_color = [0.0, 0.0, 0.0, 0.0];

        //let window ;
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    imgui_renderer.cleanup(&gl);

                    *control_flow = ControlFlow::Exit;
                }

                Event::NewEvents(_) => {}

                Event::MainEventsCleared => {
                    platform
                        .prepare_frame(imgui.io_mut(), &windowed_context.window())
                        .expect("Failed to prepare frame");
                    windowed_context.window().request_redraw();
                }

                Event::RedrawRequested(_) => {
                    gl.clear(glow::COLOR_BUFFER_BIT);

                    // other application-specific logic
                    let delta = Instant::now().duration_since(last_frame); //  last_frame.duration_since(  )
                    imgui.io_mut().update_delta_time(delta);
                    last_frame = Instant::now();

                    platform.handle_event(imgui.io_mut(), &windowed_context.window(), &event); // step 3

                    let ui = imgui.frame();

                    imgui::Window::new(im_str!("Hello window!"))
                        .size(
                            [300.0, windowed_context.window().inner_size().height as f32],
                            imgui::Condition::Always,
                        )
                        .position([0.0, 0.0], imgui::Condition::Always)
                        .build(&ui, || {
                            imgui::Slider::new(im_str!("Slider!"))
                                .range(0.0..=1.0)
                                .build(&ui, &mut changeable);

                            ui.text("text");
                            imgui::InputText::new(
                                &ui,
                                im_str!("hey there!"),
                                &mut modifiable_string,
                            )
                            .resize_buffer(true)
                            .build();
                            imgui::ColorEdit::new(im_str!("color"), &mut mod_color)
                                .hdr(true)
                                .build(&ui);
                        });

                    ui.show_demo_window(&mut true);

                    let draw_data = ui.render();
                    imgui_renderer.render(&gl, &draw_data);
                    windowed_context.swap_buffers().unwrap();
                }
                // other application-specific event handling
                event => {
                    platform.handle_event(imgui.io_mut(), &windowed_context.window(), &event);
                    // step 3
                }
            }
        });
    } // end of unsafe
}
