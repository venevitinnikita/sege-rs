extern crate gl;
extern crate sdl2;

use std::ffi::CString;
use gl::types::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const VERTEX_SHADER: &'static str = "\
#version 150

in vec2 position;

void main()
{
    gl_Position = vec4(position, 0.0, 1.0);
}
";

const FRAGMENT_SHADER: &'static str = "\
#version 150

out vec4 outColor;

void main()
{
    outColor = vec4(1.0, 1.0, 1.0, 1.0);
}
";

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Sege", 1366, 768)
        .fullscreen()
        .opengl()
        .build()
        .unwrap();
    let ctx = window.gl_create_context().unwrap();
    let _res = window.gl_make_current(&ctx);

    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    video_subsystem.gl_set_swap_interval(1);

    unsafe {
        let mut vao: GLuint = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let mut vertices = vec![
        0f32, 0.5f32,
        0.5f32, -0.5f32,
        -0.5f32, -0.5f32
        ];

        let mut vbo: GLuint = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertices.len() * 4) as GLsizeiptr,
                       vertices.as_mut_ptr() as *const _,
                       gl::STATIC_DRAW);

        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        // Здесь обязательно использовать переменную, иначе строка будет освобождаться
        let vs = CString::new(VERTEX_SHADER).unwrap();
        gl::ShaderSource(vertex_shader, 1, &vs.as_ptr(), std::ptr::null());
        gl::CompileShader(vertex_shader);
        check_shader_compile_status(vertex_shader);

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        // Здесь обязательно использовать переменную, иначе строка будет освобождаться
        let fs = CString::new(FRAGMENT_SHADER).unwrap();
        gl::ShaderSource(fragment_shader, 1, &fs.as_ptr(), std::ptr::null());
        gl::CompileShader(fragment_shader);
        check_shader_compile_status(fragment_shader);

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);

        let out_color_str = CString::new("outColor").unwrap();
        gl::BindFragDataLocation(shader_program, 0, out_color_str.as_ptr());

        gl::LinkProgram(shader_program);
        gl::UseProgram(shader_program);

        let position_str = CString::new("position").unwrap();
        let position_attr: GLuint = gl::GetAttribLocation(
            shader_program, position_str.as_ptr()) as GLuint;
        gl::VertexAttribPointer(position_attr, 2, gl::FLOAT, gl::FALSE, 0, 0 as *const _);
        gl::EnableVertexAttribArray(position_attr);
    }

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        unsafe { gl::DrawArrays(gl::TRIANGLES, 0, 3); }
        window.gl_swap_window();
    }
}

fn check_shader_compile_status(shader: GLuint) {
    unsafe {
        let mut status: GLint = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if (status as u8) != gl::TRUE {
            let buffer = CString::from_vec_unchecked(vec![1u8; 512]);
            let pbuf: *mut i8 /* c_char */ = buffer.into_raw();
            let mut _err_str_len: GLsizei = 0;
            gl::GetShaderInfoLog(shader, 512, &mut _err_str_len, pbuf);
            match CString::from_raw(pbuf).into_string() {
                Ok(err_str) => panic!("{}", err_str),
                Err(_) => panic!("Ошибка при компиляции шейдера")
            }
        }
    }
}