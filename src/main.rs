extern crate gl;
extern crate sdl2;

use std::ffi::CString;
use std::time::{Instant, Duration};
use gl::types::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const VERTEX_SHADER: &'static str = "\
#version 130

in vec2 position;
in vec3 color;

out vec3 Color;

void main()
{
    Color = color;
    gl_Position = vec4(position, 0.0, 1.0);
}
";

const FRAGMENT_SHADER: &'static str = "\
#version 130

in vec3 Color;

out vec4 out_color;

void main()
{
    out_color = vec4(Color, 1.0);
}
";

const MS_PER_UPDATE: u32 = 15;

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
        0f32, 0.5f32, 1f32, 0f32, 0f32,
        0.5f32, -0.5f32, 0f32, 1f32, 0f32,
        -0.5f32, -0.5f32, 0f32, 0f32, 1f32
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

        let out_color_str = CString::new("out_color").unwrap();
        gl::BindFragDataLocation(shader_program, 0, out_color_str.as_ptr());

        gl::LinkProgram(shader_program);
        gl::UseProgram(shader_program);

        let position_str = CString::new("position").unwrap();
        let position_attr = gl::GetAttribLocation(shader_program, position_str.as_ptr()) as u32;
        gl::VertexAttribPointer(position_attr, 2, gl::FLOAT, gl::FALSE, 5 * 4, 0 as *const _);
        gl::EnableVertexAttribArray(position_attr);

        let color_str = CString::new("color").unwrap();
        let color_attr = gl::GetAttribLocation(shader_program, color_str.as_ptr()) as u32;
        gl::VertexAttribPointer(color_attr, 3, gl::FLOAT, gl::FALSE, 5 * 4, (2 * 4) as *const _);
        gl::EnableVertexAttribArray(color_attr);
    }

    let ms_per_update = Duration::from_millis(MS_PER_UPDATE as u64);
    let mut now = Instant::now();
    let mut lag = Duration::from_millis(0);
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        lag += now.elapsed();
        now = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        println!("Before update");
        while lag >= ms_per_update {
            // TODO update();
            println!("{:?}", lag);
            lag -= ms_per_update;
        }
        println!("After update");

        // TODO render(lag / ms_per_update);

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