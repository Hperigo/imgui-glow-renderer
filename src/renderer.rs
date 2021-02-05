pub extern crate imgui;



use std::borrow::Borrow;

use glow::*;

use imgui::internal::RawWrapper;
use imgui::{BackendFlags, DrawCmd, DrawCmdParams, DrawData, ImString, TextureId, Textures};
#[derive(Debug)]
pub struct Renderer{ 
    program : u32,
    font_texture : u32, 

    ebo : u32,
    vao  : u32,
    vbo : u32, 
}

impl Renderer{
    pub fn new( gl : &glow::Context, imgui : &mut imgui::Context ) -> Self{
        
        let (program, vao, vbo, ebo)  = unsafe {
            
            let shader_version = "#version 410";
            gl.create_program().expect("Error creating ImGui Shader");

            let program= gl.create_program().expect("Cannot create program");

            let (vertex_shader_source, fragment_shader_source) = (
                //Vertex Shader ---- 
                r#"
                
                uniform mat4 matrix;
                
                in vec2 pos;
                in vec2 uv;
                in vec4 col;
                
                out vec2 f_uv;
                out vec4 f_color;
                
                // Built-in:
                // vec4 gl_Position
                
                void main() {
                   f_uv = uv;
                   f_color = col;
                  gl_Position = matrix * vec4(pos.xy, 0, 1);
                }"#,

                //Frag Shader ---- 
                r#"
                uniform sampler2D tex;
                in vec2 f_uv;
                in vec4 f_color;
                
                out vec4 out_color;
                
                void main() {
                  vec4 col =  texture(tex, f_uv.st);
                  out_color = f_color * col;
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

            let vao = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            let ebo = gl.create_buffer().expect("failed to create ebo");
            let vbo = gl.create_buffer().expect("failed to create vertex buffer");
          
            
            (program, vao, vbo, ebo)
        };

        let texture = unsafe {// Build fonts atlas

            let font_texture = gl.create_texture().expect("could not create texture");   //return_param(|x| gl.GenTextures(1, x));
            gl.bind_texture(glow::TEXTURE_2D, Some(font_texture));
            
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as _);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as _);
            
            let mut fonts =  imgui.fonts();
            let texture_atlas= fonts.build_rgba32_texture();
            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA as _, texture_atlas.width as _, texture_atlas.height as _, 0, glow::RGBA, glow::UNSIGNED_BYTE ,  Some(&texture_atlas.data) );
            gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, 0);
            font_texture
        };

        Renderer { 
            program,
            font_texture : texture,
            ebo : ebo,
            vao : vao,
            vbo : vbo,
        }
    }   


    pub fn render(&self, gl : &glow::Context, draw_data : &imgui::DrawData){
            
        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];
        if !(fb_width > 0.0 && fb_height > 0.0) {
            return;
        }

        let left = draw_data.display_pos[0];
        let right = draw_data.display_pos[0] + draw_data.display_size[0];
        let top = draw_data.display_pos[1];
        let bottom = draw_data.display_pos[1] + draw_data.display_size[1];

        unsafe{
            gl.viewport(0, 0, fb_width as _, fb_height as _);
        }
        
        let matrix = [
            (2.0 / (right - left)), 0.0, 0.0, 0.0,
            0.0, (2.0 / (top - bottom)), 0.0, 0.0,
            0.0, 0.0, -1.0, 0.0,
            
                (right + left) / (left - right),
                (top + bottom) / (bottom - top),
                0.0,
                1.0,
        ];

        let clip_off = draw_data.display_pos;
        let clip_scale = draw_data.framebuffer_scale;

    unsafe{
        

        for draw_list in draw_data.draw_lists() {
            let vtx_buffer = draw_list.vtx_buffer();

            gl.bind_vertex_array( Some(self.vao) );
            gl.bind_buffer( glow::ARRAY_BUFFER, Some(self.vbo));

            let buffer = std::slice::from_raw_parts(
                                vtx_buffer.as_ptr() as *const u8,
                                vtx_buffer.len() * std::mem::size_of::<imgui::DrawVert>());
            
            let mut indices : Vec<u16> = Vec::new();

            for data in draw_list.idx_buffer(){
                indices.push(*data);
            }


            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER,
                      buffer , glow::STREAM_DRAW
            );
            
            
            gl.bind_buffer( glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER,                
                std::slice::from_raw_parts(
                    indices.as_ptr() as *const u8,
                    indices.len() * std::mem::size_of::<u16>()) , glow::STREAM_DRAW
            );

            let pos_attrib_loc = gl.get_attrib_location(self.program, "pos").expect("could not find pos attrib");
            let uv_attrib_loc = gl.get_attrib_location(self.program, "uv").expect("could not find uv attrib");
            let color_attrib_loc = gl.get_attrib_location(self.program, "col").expect("could not find color attrib");

            gl.enable_vertex_attrib_array(pos_attrib_loc);
            gl.vertex_attrib_pointer_f32(
                pos_attrib_loc,
                2,
                glow::FLOAT,
                false,
                std::mem::size_of::<imgui::DrawVert>() as i32,
                offset_of!(imgui::DrawVert, pos)  as i32,
            );

            gl.enable_vertex_attrib_array(uv_attrib_loc);
            gl.vertex_attrib_pointer_f32(
                uv_attrib_loc,
                2,
                glow::FLOAT,
                false,
                std::mem::size_of::<imgui::DrawVert>() as i32,
                offset_of!(imgui::DrawVert, uv)  as i32,
            );

            gl.enable_vertex_attrib_array(color_attrib_loc);
            gl.vertex_attrib_pointer_f32(
                color_attrib_loc,
                4,
                glow::UNSIGNED_BYTE,
                true,
                std::mem::size_of::<imgui::DrawVert>() as i32,
                offset_of!(imgui::DrawVert, col) as i32,
            );
            

            gl.bind_vertex_array(None);
            gl.bind_buffer( glow::ELEMENT_ARRAY_BUFFER, None);
            gl.bind_buffer( glow::ARRAY_BUFFER, None);



            gl.enable( glow::BLEND );
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.use_program(Some(self.program));
            
            let shader_loc = gl.get_uniform_location(self.program, "matrix").expect("error finding matrix uniform");
            gl.uniform_matrix_4_f32_slice(
                Some(&shader_loc), 
                false, 
                &matrix,
                );

            let texture_loc = gl.get_uniform_location(self.program, "tex").expect("error finding texture sampler uniform");
            gl.bind_texture(glow::TEXTURE_2D, Some(self.font_texture));
            gl.uniform_1_i32(Some(&texture_loc), 0);

            

            gl.bind_vertex_array(Some(self.vao));
             for cmd in draw_list.commands(){

                match cmd {
                    DrawCmd::Elements {
                        count,
                        cmd_params:
                            DrawCmdParams {
                                clip_rect,
                                texture_id,
                                vtx_offset,
                                idx_offset,
                                ..
                            },
                    } =>  {

                        gl.scissor(clip_rect[0] as i32, clip_rect[1] as i32, clip_rect[2] as i32,clip_rect[3] as i32);
                        gl.draw_elements(glow::TRIANGLES, count as i32, glow::UNSIGNED_SHORT, (idx_offset * std::mem::size_of::<u16>()) as _ );
                        
                    },
                    
                    _=> (),
                }

            }

             gl.bind_vertex_array(None);
             gl.bind_texture(glow::TEXTURE_2D, None);
           }
        
        }

    }
}
