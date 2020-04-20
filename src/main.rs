extern crate glfw;
use self::glfw::{Action, Context, Key};

extern crate gl;
use self::gl::types::*;

use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::str;
use std::sync::mpsc::Receiver;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    void main() {
       gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    out vec4 FragColor;
    uniform vec4 ourColor;
    void main() {
       FragColor = ourColor;
    }
"#;

unsafe fn check_shader_compile_error(label: &str, shader: GLuint) {
    let mut success = gl::FALSE as GLint;
    let mut info_log = Vec::with_capacity(512);
    info_log.set_len(512 - 1); // subtract 1 to skip the trailing null character
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success != gl::TRUE as GLint {
        gl::GetShaderInfoLog(
            shader,
            512,
            ptr::null_mut(),
            info_log.as_mut_ptr() as *mut GLchar,
        );
        println!(
            "ERROR: {} shader compile failed\n{}",
            label,
            str::from_utf8(&info_log).unwrap()
        );
    }
}

unsafe fn check_linking_error(label: &str, shader: GLuint) {
    let mut success = gl::FALSE as GLint;
    let mut info_log = Vec::with_capacity(512);
    info_log.set_len(512 - 1); // subtract 1 to skip the trailing null character
    gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
    if success != gl::TRUE as GLint {
        gl::GetProgramInfoLog(
            shader,
            512,
            ptr::null_mut(),
            info_log.as_mut_ptr() as *mut GLchar,
        );
        println!(
            "ERROR: {} linkng failed\n{}",
            label,
            str::from_utf8(&info_log).unwrap()
        );
    }
}

fn main() {
    // glfw: initialize and configure
    // ------------------------------
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // glfw window creation
    // --------------------
    let (mut window, events) = glfw
        .create_window(
            SCR_WIDTH,
            SCR_HEIGHT,
            "LearnOpenGL",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let shader_program = unsafe {
        // build and compile our shader program
        // ------------------------------------
        // vertex shader
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);
        check_shader_compile_error("vertex", vertex_shader);

        // fragment shader
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);
        check_shader_compile_error("fragment", fragment_shader);

        // link shaders
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        check_linking_error("link", shader_program);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        shader_program
    };

    let vao = unsafe {
        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        // HINT: type annotation is crucial since default for float literals is f64
        let vertices: [f32; 9] = [
            -0.5, -0.5, 0.0, // left
            0.5, -0.5, 0.0, // right
            0.0, 0.5, 0.0, // top
        ];
        let (mut vbo, mut vao) = (0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &vertices[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        // note that this is allowed, the call to gl::VertexAttribPointer registered VBO as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        // You can unbind the VAO afterwards so other VAO calls won't accidentally modify this VAO, but this rarely happens. Modifying other
        // VAOs requires a call to glBindVertexArray anyways so we generally don't unbind VAOs (nor VBOs) when it's not directly necessary.
        gl::BindVertexArray(0);

        // uncomment this call to draw in wireframe polygons.
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        vao
    };

    // render loop
    // -----------
    while !window.should_close() {
        // events
        // -----
        process_events(&mut window, &events);

        // render
        // ------
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // be sure to activate the shader before any calls to glUniform
            gl::UseProgram(shader_program);

            gl::BindVertexArray(vao);

            // update shader uniform
            let time_value = glfw.get_time() as f32;
            let green_value = time_value.sin() / 2.0 + 0.5;
            let our_color = CString::new("ourColor").unwrap();
            let vertex_color_location = gl::GetUniformLocation(shader_program, our_color.as_ptr());
            gl::Uniform4f(vertex_color_location, 0.0, green_value, 0.0, 1.0);

            // render the triangle
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {}
        }
    }
}
