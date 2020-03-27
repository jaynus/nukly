use glow::*;
use nukly;

#[cfg(all(target_arch = "wasm32"))]
use wasm_bindgen::prelude::*;

const MAX_VERTEX_MEMORY: i32 = 512 * 1024;
const MAX_ELEMENT_MEMORY: i32 = 128 * 1024;

#[cfg_attr(all(target_arch = "wasm32"), wasm_bindgen(start))]
pub fn wasm_main() {
    main();
}

fn setup_nk_vertex_attrib<C: glow::HasContext>(gl: &C) {
    unsafe {
        gl.vertex_attrib_pointer_f32(
            0,
            2,
            glow::FLOAT,
            false,
            std::mem::size_of::<nukly::draw::Vertex>() as i32,
            0,
        );
        gl.vertex_attrib_pointer_f32(
            1,
            2,
            glow::FLOAT,
            false,
            std::mem::size_of::<nukly::draw::Vertex>() as i32,
            (std::mem::size_of::<f32>() * 2) as i32,
        );
        gl.vertex_attrib_pointer_f32(
            2,
            4,
            glow::UNSIGNED_BYTE,
            true,
            std::mem::size_of::<nukly::draw::Vertex>() as i32,
            (std::mem::size_of::<f32>() * 4) as i32,
        );

        gl.enable_vertex_attrib_array(0);
        gl.enable_vertex_attrib_array(1);
        gl.enable_vertex_attrib_array(2);
    }
}

fn setup_nk_vertex_buffers<C: glow::HasContext>(
    gl: &C,
) -> (
    <C as glow::HasContext>::VertexArray,
    <C as glow::HasContext>::Buffer,
    <C as glow::HasContext>::Buffer,
) {
    unsafe {
        let array = gl
            .create_vertex_array()
            .expect("Cannot create vertex array");
        gl.bind_vertex_array(Some(array));

        let vbo = gl.create_buffer().expect("Failed to create vbo");
        let veo = gl.create_buffer().expect("Failed to create veo");
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(veo));

        setup_nk_vertex_attrib(gl);

        (array, vbo, veo)
    }
}

fn main() {
    unsafe {
        #[cfg(target_arch = "wasm32")]
        let (_window, gl, _events_loop, render_loop, shader_version) = {
            use wasm_bindgen::JsCast;
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();
            let webgl2_context = canvas
                .get_context("webgl2")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::WebGl2RenderingContext>()
                .unwrap();
            (
                (),
                glow::Context::from_webgl2_context(webgl2_context),
                (),
                glow::RenderLoop::from_request_animation_frame(),
                "#version 300 es",
            )
        };

        // Create a context from a glutin window on non-wasm32 targets
        #[cfg(not(target_arch = "wasm32"))]
        let (gl, event_loop, windowed_context, shader_version) = {
            let el = glutin::event_loop::EventLoop::new();
            let wb = glutin::window::WindowBuilder::new()
                .with_title("Hello triangle!")
                .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
            let windowed_context = glutin::ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(wb, &el)
                .unwrap();
            let windowed_context = windowed_context.make_current().unwrap();
            let context = glow::Context::from_loader_function(|s| {
                windowed_context.get_proc_address(s) as *const _
            });
            (context, el, windowed_context, "#version 410")
        };

        let program = gl.create_program().expect("Cannot create program");

        let (vertex_shader_source, fragment_shader_source) = (
            r#" uniform mat4 ProjMtx;
                in vec2 Position;
                in vec2 TexCoord;
                in vec4 Color;
                out vec2 Frag_UV;
                out vec4 Frag_Color;
                void main() {
                   Frag_UV = TexCoord;
                   Frag_Color = Color;
                   gl_Position = ProjMtx * vec4(Position.xy, 0, 1);
                };
                "#,
            r#" precision mediump float;
                uniform sampler2D Texture;
                in vec2 Frag_UV;
                in vec4 Frag_Color;
                out vec4 Out_Color;
                void main(){
                   Out_Color = Frag_Color * texture(Texture, Frag_UV.st);
                }
                "#,
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

        let (vertex_array, vbo, veo) = setup_nk_vertex_buffers(&gl);

        gl.buffer_data_size(vbo, MAX_VERTEX_MEMORY, glow::STREAM_DRAW);
        gl.buffer_data_size(vbo, MAX_ELEMENT_MEMORY, glow::STREAM_DRAW);

        #[cfg(not(target_arch = "wasm32"))]
        {
            use glutin::event::{Event, WindowEvent};
            use glutin::event_loop::ControlFlow;

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
                        gl.use_program(Some(program));
                        gl.clear_color(0.1, 0.2, 0.3, 1.0);

                        gl.enable(glow::BLEND);
                        gl.blend_equation(glow::FUNC_ADD);
                        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

                        gl.disable(glow::CULL_FACE);
                        gl.disable(glow::DEPTH_TEST);
                        gl.enable(glow::SCISSOR_TEST);

                        gl.active_texture(glow::TEXTURE0);

                        let texture_loc = gl.get_uniform_location(program, "Texture");
                        let proj_loc = gl.get_uniform_location(program, "ProjMtx");

                        gl.uniform_1_i32(texture_loc, 0);
                        #[rustfmt::skip]
                            let ortho = [
                            2.0 / 1024.0, 0.0, 0.0, 0.0,
                            0.0, -2.0 / 768.0, 0.0, 0.0,
                            0.0, 0.0, -1.0, 0.0,
                            -1.0, 1.0, 0.0, 1.0,
                        ];
                        gl.uniform_matrix_4_f32_slice(proj_loc, false, &ortho);
                        //https://github.com/Immediate-Mode-UI/Nuklear/blob/master/example/canvas.c
                        gl.enable_vertex_attrib_array(0);
                        gl.enable_vertex_attrib_array(1);
                        gl.enable_vertex_attrib_array(2);
                        gl.bind_vertex_array(Some(vertex_array));
                        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(veo));

                        let vertices =
                            gl.map_buffer_range(vbo, 0, MAX_VERTEX_MEMORY, glow::WRITE_ONLY);
                        let elements =
                            gl.map_buffer_range(veo, 0, MAX_VERTEX_MEMORY, glow::WRITE_ONLY);

                        gl.unmap_buffer(vbo);
                        gl.unmap_buffer(veo);

                        windowed_context.swap_buffers().unwrap();
                    }
                    Event::WindowEvent { ref event, .. } => match event {
                        WindowEvent::Resized(physical_size) => {
                            windowed_context.resize(*physical_size);
                        }
                        WindowEvent::CloseRequested => {
                            gl.delete_program(program);
                            *control_flow = ControlFlow::Exit
                        }
                        _ => (),
                    },
                    _ => (),
                }
            });
        }

        #[cfg(target_arch = "wasm32")]
        render_loop.run(move |running: &mut bool| {
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.draw_arrays(glow::TRIANGLES, 0, 3);

            if !*running {
                gl.delete_program(program);
            }
        });
    }
}