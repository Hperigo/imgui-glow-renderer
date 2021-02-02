use glow::*;

use imgui::im_str;
use nanovg::{
    Alignment, BasicCompositeOperation, Clip, Color, CompositeOperation, Font, Gradient, Image, ImagePattern,
    PathOptions, Scissor, StrokeOptions, TextOptions, Transform,
};

use imgui_winit_support::{HiDpiMode, WinitPlatform};

mod renderer;

fn main() {



    let (gl, event_loop, windowed_context, shader_version) = {
        let el = glutin::event_loop::EventLoop::new();
        let wb = glutin::window::WindowBuilder::new()
            .with_title("Hello NanoVG and Imgui!")
            .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
        let windowed_context = glutin::ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(wb, &el)
            .unwrap();
        unsafe{     
            let windowed_context = windowed_context.make_current().unwrap();
            let context = glow::Context::from_loader_function(|s| {
                windowed_context.get_proc_address(s) as *const _
            });
            (context, el, windowed_context, "#version 410")
        }
    };
    let scale_factor = windowed_context.window().scale_factor() as f32;

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut imgui);
    {
        platform.attach_window(imgui.io_mut(), windowed_context.window(),   HiDpiMode::Rounded);
    }    


    let font_size = 13.0 * scale_factor;
    imgui.fonts().add_font(&[
        imgui::FontSource::DefaultFontData{
            config : Some(imgui::FontConfig{
                    size_pixels : font_size,
                    .. imgui::FontConfig::default()
            }),
        }
    ]);

    imgui.io_mut().font_global_scale = (1.0 / scale_factor) as f32;
    let _ = imgui.fonts().build_rgba32_texture();

    let imgui_renderer = renderer::Renderer::new(&gl);
    println!("Hello there! {:?}", imgui_renderer);

    unsafe {

        // NANO VG 

        let vg_context = nanovg::ContextBuilder::new()
        .stencil_strokes()
        .build()
        .expect("Initialization of NanoVG failed!");


        let vertex_array = gl
        .create_vertex_array()
        .expect("Cannot create vertex array");
        gl.bind_vertex_array(Some(vertex_array));

        let program = gl.create_program().expect("Cannot create program");

        let (vertex_shader_source, fragment_shader_source) = (
            r#"const vec2 verts[3] = vec2[3](
                vec2(0.5f, 1.0f),
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#,
            r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#,
        );

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!(gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!(gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        gl.use_program(Some(program));
        gl.clear_color(0.1, 0.2, 0.3, 1.0);

        
        let width = 1024.0;
        let height =768.0;

        use glutin::event::{Event, WindowEvent};
        use glutin::event_loop::ControlFlow;

        let mut changeable = 0.01;
        event_loop.run(move |event, _, control_flow| {

            *control_flow = ControlFlow::Wait;
            match event {
                Event::LoopDestroyed => {
                    return;
                }
                Event::MainEventsCleared => {
                    windowed_context.window().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    gl.clear(glow::COLOR_BUFFER_BIT);
                   //gl.draw_arrays(glow::TRIANGLES, 0, 3);


                    //gl.viewport(0, 0, (width * scale_factor) as i32, (height * scale_factor) as i32 );

                    /*
                    vg_context.frame((width, height), scale_factor, | frame | {

                        frame.path(
                            |path| {
                                path.rect((0.0, 0.0), (300.0, 300.0));
                                path.fill(
                                    Gradient::Linear {
                                        start: (100.0, 100.0),
                                        end: (400.0, 400.0),
                                        start_color: Color::from_rgb(0xAA, 0x6C, 0x39),
                                        end_color: Color::from_rgb(0x88, 0x2D, 0x60),
                                    },
                                    Default::default(),
                                );
                            },
                            Default::default(),
                        );
                        
                        
                    });// end of nanovg context ----
                    */


                    let mut ui = imgui.frame();
                    
                    imgui::Window::new( im_str!("Hello window!") )
                        .size([300.0, 110.0], imgui::Condition::FirstUseEver)
                        .build( &ui, || {
                            imgui::Slider::new(im_str!("Slider!"))
                            .range(0.0 ..= 1.0)
                            .build(&ui, &mut changeable);
                        });

                    let draw_data = ui.render();
                    imgui_renderer.render(&gl, &draw_data);

                    windowed_context.swap_buffers().unwrap();
                }
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {
                        windowed_context.resize(*physical_size);
                    }
                    WindowEvent::CloseRequested => {
                        gl.delete_program(program);
                        gl.delete_vertex_array(vertex_array);
                        *control_flow = ControlFlow::Exit
                    }
                    _ => (),
                },
                event => {
                    platform.handle_event(imgui.io_mut(), windowed_context.window(), &event);
                },
            }
        });
    }
}