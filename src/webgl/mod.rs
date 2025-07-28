//! General webgl primitives

use web_sys::WebGlProgram;
use web_sys::WebGlRenderingContext as GL;
use web_sys::WebGlShader;

mod canvas;
mod compute;

pub use canvas::{Canvas, CanvasProperties, CanvasRenderer, RenderData, RenderLoopState};
pub use compute::{ComputeProgram, UniformConstAccess, UniformSet};
use web_sys::WebGlUniformLocation;

/// Wrapper around a uniform location and data
#[derive(Debug)]
pub struct Uniform<Data> {
    /// The uniform location as a string
    name: String,
    /// The uniform location handle for webgl
    location: Option<WebGlUniformLocation>,
    /// The data that will be applied to the uniform
    data: Data,
}

impl<Data: UniformData> Uniform<Data> {
    /// Create a new uniform wrapper around a uniform in the given program.
    ///
    /// # Panics
    /// If the uniform name can not be resolved.
    pub fn new(gl: &GL, program: &WebGlProgram, name: impl Into<String>, data: Data) -> Self {
        let name: String = name.into();

        let location = gl.get_uniform_location(program, &name);
        if location.is_none() {
            let max_uniforms = gl
                .get_program_parameter(program, GL::ACTIVE_UNIFORMS)
                .as_f64()
                .unwrap() as u32;
            let valid_locations: String = (0..max_uniforms)
                .map(|i| gl.get_active_uniform(program, i).unwrap().name())
                .reduce(|mut a, b| {
                    a.push_str(", ");
                    a.push_str(&b);
                    a
                })
                .unwrap_or("No options found".to_owned());
            log::error!("Expected valid uniform location\nGot: {name}\nOptions: {valid_locations}");
        };

        Self {
            name,
            location,
            data,
        }
    }

    /// Returns the name of the uniform
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the data of the uniform
    pub fn set_data(&mut self, data: Data) {
        self.data = data;
    }

    /// Applies this uniform by sending the data to the graphics card
    pub fn apply(&self, gl: &GL) {
        if let Some(location) = self.location.as_ref() {
            self.data.apply(gl, location);
        } else {
            log::warn!(
                "Tried to set uniform without valid location: {name}",
                name = self.name()
            );
        }
    }

    /// A convenience wrapper for setting and then applying uniform data
    pub fn apply_data(&mut self, gl: &GL, data: Data) {
        self.set_data(data);
        self.apply(gl);
    }
}

/// A trait for types that can be used in uniforms
pub trait UniformData: std::fmt::Debug {
    /// Applies (writes) this data to the given uniform location
    fn apply(&self, gl: &GL, location: &WebGlUniformLocation);
}

/// Implement uniform data for tuples
macro_rules! impl_uniform_data {
    (
        $(
            $type:ty: {
                $(
                    $func:ident(
                        $(
                            $arg:ident
                        ),*
                    )
                ),*
                $(,)?
            }
        )*
        $(,)?
    ) => {
        $(
            $(
                impl UniformData for (
                    $(
                        impl_uniform_data!(@expand_to_first $type: $arg)
                    ),*
                    ,
                ) {
                    fn apply(&self, gl: &GL, location: &WebGlUniformLocation) {
                        let ($($arg),*,) = self;
                        gl.$func(Some(location), $($arg.clone()),*);
                    }
                }
            )*
        )*
    };
    (@expand_to_first $type:ty: $x:ident) => {
        $type
    };
}

impl_uniform_data! {
    f32: {
        uniform1f(a),
        uniform2f(a, b),
        uniform3f(a, b, c),
        uniform4f(a, b, c, d),
    }
    i32: {
        uniform1i(a),
        uniform2i(a, b),
        uniform3i(a, b, c),
        uniform4i(a, b, c, d),
    }
}

/// Compile a [`WebGlShader`] and log any errors to the console
pub fn compile_shader(
    gl: &GL,
    shader_type: u32,
    shader_source: impl AsRef<str>,
) -> Option<WebGlShader> {
    let shader = gl.create_shader(shader_type).unwrap();

    gl.shader_source(&shader, shader_source.as_ref());
    gl.compile_shader(&shader);
    let success = gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap();

    if success {
        Some(shader)
    } else {
        let log = gl.get_shader_info_log(&shader).unwrap();
        log::error!("{log}");
        None
    }
}

/// Compile a [`WebGlProgram`] and log any errors to the console
pub fn create_program(
    gl: &GL,
    vertex_shader: &WebGlShader,
    fragment_shader: &WebGlShader,
) -> Option<WebGlProgram> {
    let program = gl.create_program().unwrap();

    gl.attach_shader(&program, vertex_shader);
    gl.attach_shader(&program, fragment_shader);
    gl.link_program(&program);

    let success = gl
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap();

    if success {
        Some(program)
    } else {
        let log = gl.get_program_info_log(&program).unwrap();
        log::error!("{log}");
        None
    }
}
