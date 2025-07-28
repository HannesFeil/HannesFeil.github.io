//! Simulating compute shaders with webgl

use web_sys::{
    WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlRenderingContext as GL, WebGlTexture,
    js_sys::Float32Array,
};

use crate::webgl::{Uniform, UniformData, compile_shader, create_program};

// TODO: write docs

pub trait UniformConstAccess<const INDEX: u32> {
    type UniformDataType;

    fn access(&mut self) -> &mut Uniform<Self::UniformDataType>;
}

pub trait UniformSet {
    fn initialize(gl: &GL, program: &WebGlProgram) -> Self;

    fn apply_all(&self, gl: &GL);
}

/// # Example
/// ```
/// uniform_set! {
///     pub TestSet {
///         u_position: (f32, f32), // Uses default implemenation for initialization
///         u_aspect: (f32,) = (1.0,), // Initializes with value (1.0,)
///     }
/// }
/// ```
#[macro_export]
macro_rules! uniform_set {
    (
        $set_visibility:vis $set_name:ident {
            $(
                $location:ident: $type:ty $(= $val:expr)?
            ),*
            $(,)?
        }

    ) => {
        #[derive(Debug)]
        $set_visibility struct $set_name {
            $(
                pub $location: Uniform<$type>
            ),*
        }

        #[allow(non_upper_case_globals, dead_code)]
        impl $set_name {
            uniform_set!(@count_constants | $($location),*);

            pub fn access<const UNIFORM_LOCATION: u32>(&mut self) -> &mut Uniform<<Self as $crate::webgl::UniformConstAccess<UNIFORM_LOCATION>>::UniformDataType>
            where
                Self: $crate::webgl::UniformConstAccess<UNIFORM_LOCATION>
            {
                <Self as $crate::webgl::UniformConstAccess<UNIFORM_LOCATION>>::access(self)
            }
        }

        #[allow(unused_variables)]
        impl $crate::webgl::UniformSet for $set_name {
            fn initialize(gl: &GL, program: &WebGlProgram) -> Self {
                Self {
                    $(
                        $location: Uniform::new(gl, program, stringify!($location), uniform_set!(@val_or_default $($val)?))
                    ),*
                }
            }

            fn apply_all(&self, gl: &GL) {
                $(
                    self.$location.apply(gl)
                );*
            }
        }

        $(
            impl $crate::webgl::UniformConstAccess<{ $set_name::$location }> for $set_name {
                type UniformDataType = $type;

                fn access(&mut self) -> &mut Uniform<Self::UniformDataType> {
                    &mut self.$location
                }
            }
        )*
    };
    (@count_constants | ) => {};
    (@count_constants $($counted:ident),* | $first:ident, $($rest:ident),*) => {
        pub const $first: u32 = uniform_set!(@to_number $($counted),*);
        uniform_set!(@count_constants $($counted,)* $first | $($rest),*);
    };
    (@count_constants $($counted:ident),* | $last:ident) => {
        pub const $last: u32 = uniform_set!(@to_number $($counted),*);
    };
    (@to_number $($id:ident),*) => {
        $(
            uniform_set!(@to_one $id) +
        )*
        0
    };
    (@to_one $id:ident) => {
        1
    };
    (@val_or_default $val:expr) => {
        $val
    };
    (@val_or_default) => {
        Default::default()
    };
}

/// A compute program, consisting of multiple input textures and an output textures.
///
/// All textures must have the same sizes.
/// The actual computation is done using a fragment shader.
#[derive(Debug)]
pub struct ComputeProgram<Set: UniformSet> {
    /// The width of the textures
    width: u32,
    /// The height of the textures
    height: u32,
    /// The input textures
    inputs: Vec<(WebGlTexture, Uniform<(i32,)>)>,
    /// The output texture
    output_texture: WebGlTexture,
    /// The program used to compute the actual data
    program: WebGlProgram,
    /// The output framebuffer
    frame_buffer: WebGlFramebuffer,
    /// The vertex buffer
    vertex_buffer: WebGlBuffer,
    /// The dimension uniform
    dimensions_uniform: Uniform<(f32, f32)>,
    /// Any additional uniforms for the fragment shader
    uniforms: Set,
}

impl<Set: UniformSet> ComputeProgram<Set> {
    /// Vertex shader for drawing the space filling quad
    const VERTEX_SOURCE: &'static str = "
        attribute vec2 a_position;

        void main() {
            gl_Position = vec4(a_position, 0.0, 1.0);
        }
    ";

    /// Vertex coordinates for a space filling quad
    const VERTICES: [f32; 12] = [
        -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0,
    ];

    /// Creates a new compute shader with the given dimensions and uniforms and fragment shader source.
    pub fn new(
        width: u32,
        height: u32,
        inputs: usize,
        gl: &GL,
        fragment_source: impl AsRef<str>,
    ) -> Self {
        let output_texture = Self::create_texture(gl, width as i32, height as i32);

        let vertex_shader = compile_shader(gl, GL::VERTEX_SHADER, Self::VERTEX_SOURCE).unwrap();
        let fragment_shader = compile_shader(gl, GL::FRAGMENT_SHADER, fragment_source).unwrap();
        let program = create_program(gl, &vertex_shader, &fragment_shader).unwrap();

        let inputs = (0..inputs)
            .map(|i| {
                (
                    Self::create_texture(gl, width as i32, height as i32),
                    Uniform::new(gl, &program, format!("u_input_{i}"), (i as i32,)),
                )
            })
            .collect();

        let frame_buffer = gl.create_framebuffer().unwrap();
        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&frame_buffer));
        gl.framebuffer_texture_2d(
            GL::FRAMEBUFFER,
            GL::COLOR_ATTACHMENT0,
            GL::TEXTURE_2D,
            Some(&output_texture),
            0,
        );
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);

        let vertex_buffer = gl.create_buffer().unwrap();
        let verts = web_sys::js_sys::Float32Array::from(Self::VERTICES.as_slice());
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &verts, GL::STATIC_DRAW);
        gl.bind_buffer(GL::ARRAY_BUFFER, None);

        let dimensions_uniform =
            Uniform::new(gl, &program, "u_dimensions", (width as f32, height as f32));

        let uniforms = Set::initialize(gl, &program);

        Self {
            width,
            height,
            inputs,
            output_texture,
            program,
            frame_buffer,
            vertex_buffer,
            dimensions_uniform,
            uniforms,
        }
    }

    /// Convenient function for creating a floating point texture of the given size
    fn create_texture(gl: &GL, width: i32, height: i32) -> WebGlTexture {
        let texture = gl.create_texture().unwrap();

        gl.get_extension("OES_texture_float").unwrap().unwrap();
        gl.get_extension("WEBGL_color_buffer_float")
            .unwrap()
            .unwrap();

        gl.bind_texture(GL::TEXTURE_2D, Some(&texture));
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            GL::TEXTURE_2D,
            0,
            GL::RGBA as i32,
            width,
            height,
            0,
            GL::RGBA,
            GL::FLOAT,
            None,
        )
        .unwrap();
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);

        gl.bind_texture(GL::TEXTURE_2D, None);

        texture
    }

    /// Write the given data to the given input texture
    ///
    /// # Panics
    /// If the data dimension does not match the texture dimension
    pub fn write_input(&self, gl: &GL, index: usize, data: &[f32]) {
        assert_eq!(data.len() as u32, self.width * self.height * 4);
        gl.bind_texture(GL::TEXTURE_2D, Some(&self.inputs[index].0));
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
            GL::TEXTURE_2D,
            0,
            GL::RGBA as i32,
            self.width as i32,
            self.height as i32,
            0,
            GL::RGBA,
            GL::FLOAT,
            Some(&Float32Array::from(data)),
        )
        .unwrap();
        gl.bind_texture(GL::TEXTURE_2D, None);
    }

    /// Apply the compute shader and render to the output texture
    pub fn compute(&self, gl: &GL) {
        gl.use_program(Some(&self.program));
        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&self.frame_buffer));

        for (index, (texture, uniform)) in self.inputs.iter().enumerate() {
            gl.active_texture(GL::TEXTURE0 + index as u32);
            gl.bind_texture(GL::TEXTURE_2D, Some(texture));
            uniform.apply(gl);
        }

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vertex_buffer));

        let position = gl.get_attrib_location(&self.program, "a_position") as u32;
        gl.vertex_attrib_pointer_with_i32(position, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(position);

        self.dimensions_uniform.apply(gl);
        self.uniforms.apply_all(gl);

        gl.viewport(0, 0, self.width as i32, self.height as i32);
        gl.draw_arrays(GL::TRIANGLES, 0, 6);

        gl.disable_vertex_attrib_array(position);
        gl.bind_buffer(GL::ARRAY_BUFFER, None);
        gl.bind_texture(GL::TEXTURE_2D, None);
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);
        gl.use_program(None);
    }

    /// Copy the output texture to the given texture
    pub fn copy_output(&self, gl: &GL, texture: &WebGlTexture) {
        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&self.frame_buffer));
        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, Some(texture));
        gl.copy_tex_image_2d(
            GL::TEXTURE_2D,
            0,
            GL::RGBA,
            0,
            0,
            self.width as i32,
            self.height as i32,
            0,
        );
        gl.bind_texture(GL::TEXTURE_2D, None);
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);
    }

    /// Copy the output texture to the given input texture
    pub fn copy_output_to_input(&self, gl: &GL, input_index: usize) {
        self.copy_output(gl, &self.inputs[input_index].0);
    }

    /// Read the output texture into an array
    pub fn read_output(&self, gl: &GL) -> Float32Array {
        let output = Float32Array::new_with_length(self.width * self.height * 4);

        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&self.frame_buffer));
        gl.read_pixels_with_opt_array_buffer_view(
            0,
            0,
            self.width as i32,
            self.height as i32,
            GL::RGBA,
            GL::FLOAT,
            Some(&output),
        )
        .unwrap();
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);

        output
    }

    /// Return the input texture handle at the given index
    pub fn input_texture(&self, index: usize) -> &WebGlTexture {
        &self.inputs[index].0
    }

    /// Return an iterator of the input textures
    pub fn input_textures(&self) -> impl Iterator<Item = &WebGlTexture> {
        self.inputs.iter().map(|(texture, _)| texture)
    }

    /// Return the output texture
    pub fn output_texture(&self) -> &WebGlTexture {
        &self.output_texture
    }

    /// Set a given uniform
    ///
    /// # Panics
    /// If the uniform was not given to the constructor
    pub fn set_uniform<const UNIFORM_LOCATION: u32>(
        &mut self,
        data: <Set as UniformConstAccess<UNIFORM_LOCATION>>::UniformDataType,
    ) where
        Set: UniformConstAccess<UNIFORM_LOCATION>,
        <Set as UniformConstAccess<UNIFORM_LOCATION>>::UniformDataType: UniformData,
    {
        self.uniforms.access().set_data(data);
    }
}
