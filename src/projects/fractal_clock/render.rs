// TODO: restructure and cleanup pls
// TODO: Try the image rendering idea I had

use std::fmt::Display;

use color::{AlphaColor, Srgb};
use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext as GL};

use crate::{
    uniform_set,
    webgl::{
        CanvasRenderer, ComputeProgram, RenderData, Uniform, compile_shader,
        create_program,
    },
};

pub const MAX_RECURSION_DEPTH: u32 = 16;

const COMPUTE_TEXTURE_RECURSION_WIDTH: u32 = 10;
const COMPUTE_TEXTURE_RECURSION_HEIGHT: u32 =
    MAX_RECURSION_DEPTH - COMPUTE_TEXTURE_RECURSION_WIDTH + 1;
const COMPUTE_TEXTURE_WIDTH: u32 = 2_u32.pow(COMPUTE_TEXTURE_RECURSION_WIDTH);
const COMPUTE_TEXTURE_HEIGHT: u32 = 2_u32.pow(COMPUTE_TEXTURE_RECURSION_HEIGHT);

const COMPUTE_FRAGMENT_SOURCE: &str = "
    precision highp float;
    uniform sampler2D u_input_0;
    uniform vec2 u_dimensions;
    uniform vec2 u_hour_start;
    uniform vec2 u_minute_start;
    uniform vec2 u_hour;
    uniform vec2 u_minute;

    vec4 getValueFrom2DTextureAs1DArray(sampler2D tex, vec2 dimensions, float index) {
        float y = floor(index / dimensions.x);
        float x = mod(index, dimensions.x);
        vec2 texcoord = (vec2(x, y) + 0.5) / dimensions;
        return texture2D(tex, texcoord);
    }

    void main() {
        float index = floor(u_dimensions.x) * floor(gl_FragCoord.y) + floor(gl_FragCoord.x);

        if (index == 0.0) {
            gl_FragColor = vec4(u_hour_start.xy, u_hour_start.xy);
        } else if (index == 1.0) {
            gl_FragColor = vec4(u_minute_start.xy, u_minute_start.xy);
        } else {
            float parentIndex = floor(index / 2.0) - 1.0;
            vec4 data = getValueFrom2DTextureAs1DArray(u_input_0, u_dimensions, parentIndex);
            vec2 angle = data.zw;

            if (mod(index, 2.0) == 0.0) {
                angle = vec2(angle.x * u_hour.x - angle.y * u_hour.y, angle.x * u_hour.y + angle.y * u_hour.x);
            } else {
                angle = vec2(angle.x * u_minute.x - angle.y * u_minute.y, angle.x * u_minute.y + angle.y * u_minute.x);
            }

            gl_FragColor = vec4(data.x + angle.x, data.y + angle.y, angle.xy);
        }
    }
";
const VERTEX_RENDER_VERTEX_SOURCE: &str = "
    precision mediump float;

    attribute float a_index;
    uniform sampler2D u_input;
    uniform vec2 u_dimensions;
    uniform vec2 u_scale;

    vec4 getValueFrom2DTextureAs1DArray(sampler2D tex, vec2 dimensions, float index) {
        float y = floor(index / dimensions.x);
        float x = mod(index, dimensions.x);
        vec2 texcoord = (vec2(x, y) + 0.5) / dimensions;
        return texture2D(tex, texcoord);
    }

    void main() {
        float vertex_index = floor(a_index / 2.0);
        if (mod(a_index, 2.0) == 0.0) {
            vertex_index = floor(vertex_index / 2.0) - 1.0;
        }
        if (vertex_index == -1.0) {
            gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        } else {
            vec2 position = getValueFrom2DTextureAs1DArray(u_input, u_dimensions, vertex_index).xy;
            gl_Position = vec4(position.y * u_scale.x, position.x * u_scale.y, 0.0, 1.0);
        }
    }
";
const VERTEX_RENDER_FRAGMENT_SOURCE: &str = "
    precision mediump float;

    uniform vec4 u_color;

    void main() {
        gl_FragColor = u_color;
    }
";

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum BlendConstant {
    Addition = GL::FUNC_ADD,
    Subtraction = GL::FUNC_SUBTRACT,
    ReverseSubtraction = GL::FUNC_REVERSE_SUBTRACT,
    Zero = GL::ZERO,
    One = GL::ONE,
    SourceColor = GL::SRC_COLOR,
    OneMinusSourceColor = GL::ONE_MINUS_SRC_COLOR,
    DestinationColor = GL::DST_COLOR,
    OneMinusDestinationColor = GL::ONE_MINUS_DST_COLOR,
    SourceAlpha = GL::SRC_ALPHA,
    OneMinusSourceAlpha = GL::ONE_MINUS_SRC_ALPHA,
    DestinationAlpha = GL::DST_ALPHA,
    OneMinusDestinationAlpha = GL::ONE_MINUS_DST_ALPHA,
    SourceAlphaSaturate = GL::SRC_ALPHA_SATURATE,
}

impl Display for BlendConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BlendConstant::Addition => "Addition",
                BlendConstant::Subtraction => "Subtraction",
                BlendConstant::ReverseSubtraction => "Reverse Subtraction",
                BlendConstant::Zero => "Zero",
                BlendConstant::One => "One",
                BlendConstant::SourceColor => "Source Color",
                BlendConstant::OneMinusSourceColor => "One Minus Source Color",
                BlendConstant::DestinationColor => "Destination Color",
                BlendConstant::OneMinusDestinationColor => "One Minus Destination Color",
                BlendConstant::SourceAlpha => "Source Alpha",
                BlendConstant::OneMinusSourceAlpha => "One Minus Source Alpha",
                BlendConstant::DestinationAlpha => "Destination Alpha",
                BlendConstant::OneMinusDestinationAlpha => "One Minus Destination Alpha",
                BlendConstant::SourceAlphaSaturate => "Source Alpha Saturate",
            }
        )
    }
}

impl BlendConstant {
    fn value(self) -> u32 {
        self as u32
    }
}

pub const BLEND_EQUATIONS: &[BlendConstant] = &[
    BlendConstant::Addition,
    BlendConstant::Subtraction,
    BlendConstant::ReverseSubtraction,
];
pub const BLEND_MULTIPLIERS: &[BlendConstant] = &[
    BlendConstant::Zero,
    BlendConstant::One,
    BlendConstant::SourceColor,
    BlendConstant::OneMinusSourceColor,
    BlendConstant::DestinationColor,
    BlendConstant::OneMinusDestinationColor,
    BlendConstant::SourceAlpha,
    BlendConstant::OneMinusSourceAlpha,
    BlendConstant::DestinationAlpha,
    BlendConstant::OneMinusDestinationAlpha,
    BlendConstant::SourceAlphaSaturate,
];

uniform_set! {
    ComputeUniformSet {
        u_hour_start: (f32, f32),
        u_minute_start: (f32, f32),
        u_hour: (f32, f32),
        u_minute: (f32, f32),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FractalClockRenderer {}

#[derive(Debug)]
pub struct FractalClockRenderState {
    vertex_compute_input_buffer: Vec<f32>,
    vertex_compute_program: ComputeProgram<ComputeUniformSet>,
    vertex_render_program: WebGlProgram,
    vertex_render_dimensions_uniform: Uniform<(f32, f32)>,
    vertex_render_input_uniform: Uniform<(i32,)>,
    vertex_render_scale_uniform: Uniform<(f32, f32)>,
    vertex_render_color_uniform: Uniform<(f32, f32, f32, f32)>,
    vertex_render_vertex_buffer: WebGlBuffer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FractalClockRenderInput {
    pub hour_angle: f32,
    pub minute_angle: f32,
    pub animate: bool,
    pub size: f32,
    pub recursion_depth: u32,
    pub hour_ratio: f32,
    pub size_factor: f32,
    pub color: AlphaColor<Srgb>,
    pub blend_equations: (BlendConstant, BlendConstant),
    pub blend_multipliers: (BlendConstant, BlendConstant, BlendConstant, BlendConstant),
}

impl CanvasRenderer for FractalClockRenderer {
    type RenderState = FractalClockRenderState;

    type RenderInput = FractalClockRenderInput;

    fn render(
        &self,
        state: &mut Self::RenderState,
        input: &Self::RenderInput,
        gl: &GL,
        RenderData {
            initial_render,
            width,
            height,
            input_changed,
            time,
            ..
        }: RenderData,
    ) {
        if input_changed || initial_render || input.animate {
            let (hour_angle, minute_angle) = if input.animate {
                const COMPLETE_TIME_ROTATION: u32 = 12 * 60 * 60 * 10;
                const ONE_HOUR_TIME_ROTATION: u32 = COMPLETE_TIME_ROTATION / 12;
                (
                    (time % COMPLETE_TIME_ROTATION) as f32 / COMPLETE_TIME_ROTATION as f32 * 360.0,
                    (time % ONE_HOUR_TIME_ROTATION) as f32 / ONE_HOUR_TIME_ROTATION as f32 * 360.0,
                )
            } else {
                (input.hour_angle, input.minute_angle)
            };

            let (hour_y, hour_x) = hour_angle.to_radians().sin_cos();
            let (minute_y, minute_x) = minute_angle.to_radians().sin_cos();
            let (hour_start, minute_start) = (
                (hour_x * input.hour_ratio, hour_y * input.hour_ratio),
                (minute_x, minute_y),
            );
            let (hour, minute) = (
                (
                    hour_start.0 * input.size_factor,
                    hour_start.1 * input.size_factor,
                ),
                (minute_x * input.size_factor, minute_y * input.size_factor),
            );
            state
                .vertex_compute_program
                .set_uniform::<{ ComputeUniformSet::u_hour_start }>((hour_start.0, hour_start.1));
            state
                .vertex_compute_program
                .set_uniform::<{ ComputeUniformSet::u_minute_start }>((
                    minute_start.0,
                    minute_start.1,
                ));
            state
                .vertex_compute_program
                .set_uniform::<{ ComputeUniformSet::u_hour }>((hour.0, hour.1));
            state
                .vertex_compute_program
                .set_uniform::<{ ComputeUniformSet::u_minute }>((minute.0, minute.1));
            state
                .vertex_compute_program
                .write_input(gl, 0, &state.vertex_compute_input_buffer);

            state.vertex_compute_input_buffer[0] = hour_start.0;
            state.vertex_compute_input_buffer[1] = hour_start.1;
            state.vertex_compute_input_buffer[2] = hour_start.0;
            state.vertex_compute_input_buffer[3] = hour_start.1;
            state.vertex_compute_input_buffer[4] = minute_start.0;
            state.vertex_compute_input_buffer[5] = minute_start.1;
            state.vertex_compute_input_buffer[6] = minute_start.0;
            state.vertex_compute_input_buffer[7] = minute_start.1;

            for i in 2..1024 {
                let parent = i / 2 - 1;
                let position = (
                    state.vertex_compute_input_buffer[parent * 4],
                    state.vertex_compute_input_buffer[parent * 4 + 1],
                );
                let angle = (
                    state.vertex_compute_input_buffer[parent * 4 + 2],
                    state.vertex_compute_input_buffer[parent * 4 + 3],
                );
                let mut new_angle = if i % 2 == 0 { hour } else { minute };
                new_angle = (
                    angle.0 * new_angle.0 - angle.1 * new_angle.1,
                    angle.0 * new_angle.1 + angle.1 * new_angle.0,
                );
                state.vertex_compute_input_buffer[i * 4] = position.0 + new_angle.0;
                state.vertex_compute_input_buffer[i * 4 + 1] = position.1 + new_angle.1;
                state.vertex_compute_input_buffer[i * 4 + 2] = new_angle.0;
                state.vertex_compute_input_buffer[i * 4 + 3] = new_angle.1;
            }
            state
                .vertex_compute_program
                .write_input(gl, 0, &state.vertex_compute_input_buffer);

            for _ in 0..(input
                .recursion_depth
                .saturating_sub(COMPUTE_TEXTURE_RECURSION_WIDTH)
                + 1)
            {
                state.vertex_compute_program.compute(gl);
                state.vertex_compute_program.copy_output_to_input(gl, 0);
            }
        }

        gl.use_program(Some(&state.vertex_render_program));
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&state.vertex_render_vertex_buffer));
        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(
            GL::TEXTURE_2D,
            Some(state.vertex_compute_program.output_texture()),
        );

        let position = gl
            .get_attrib_location(&state.vertex_render_program, "a_index")
            .try_into()
            .unwrap();
        gl.vertex_attrib_pointer_with_i32(position, 1, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(position);

        state.vertex_render_dimensions_uniform.apply(gl);
        state.vertex_render_input_uniform.apply(gl);
        let scale = input.size
            / ((1.0
                - input
                    .size_factor
                    .powi(input.recursion_depth.try_into().unwrap()))
                / (1.0 - input.size_factor));
        state
            .vertex_render_scale_uniform
            .apply_data(gl, (height as f32 / width as f32 * scale, scale));
        let [r, g, b, a] = input.color.components;
        state
            .vertex_render_color_uniform
            .apply_data(gl, (r, g, b, a));

        gl.viewport(0, 0, width.try_into().unwrap(), height.try_into().unwrap());
        gl.clear_color(0.0, 0.0, 0.0, 0.0);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        gl.get_extension("EXT_float_blend").unwrap();
        gl.enable(GL::BLEND);
        gl.blend_equation_separate(
            input.blend_equations.0.value(),
            input.blend_equations.1.value(),
        );
        gl.blend_func_separate(
            input.blend_multipliers.0.value(),
            input.blend_multipliers.1.value(),
            input.blend_multipliers.2.value(),
            input.blend_multipliers.3.value(),
        );

        let x = 2 * 2 * (2_i32.pow(input.recursion_depth) - 1);
        gl.draw_arrays(GL::LINES, 0, x);
        gl.disable(GL::BLEND);
    }

    fn initial_render_state(
        &self,
        _: &Self::RenderInput,
        gl: &GL,
        _: RenderData,
    ) -> FractalClockRenderState {
        let max_texture_size = gl
            .get_parameter(GL::MAX_TEXTURE_SIZE)
            .unwrap()
            .as_f64()
            .unwrap() as u32;
        assert!(max_texture_size >= std::cmp::max(COMPUTE_TEXTURE_WIDTH, COMPUTE_TEXTURE_HEIGHT));

        let vertex_compute_program = ComputeProgram::new(
            COMPUTE_TEXTURE_WIDTH,
            COMPUTE_TEXTURE_HEIGHT,
            1,
            gl,
            COMPUTE_FRAGMENT_SOURCE,
        );
        let vertex_compute_input_buffer = vec![
            0.0;
            (4 * COMPUTE_TEXTURE_WIDTH * COMPUTE_TEXTURE_HEIGHT)
                .try_into()
                .unwrap()
        ];

        let vertex_render_vertex_shader =
            compile_shader(gl, GL::VERTEX_SHADER, VERTEX_RENDER_VERTEX_SOURCE).unwrap();
        let vertex_render_fragment_shader =
            compile_shader(gl, GL::FRAGMENT_SHADER, VERTEX_RENDER_FRAGMENT_SOURCE).unwrap();
        let vertex_render_program = create_program(
            gl,
            &vertex_render_vertex_shader,
            &vertex_render_fragment_shader,
        )
        .unwrap();

        let vertex_render_dimensions_uniform = Uniform::new(
            gl,
            &vertex_render_program,
            "u_dimensions",
            (COMPUTE_TEXTURE_WIDTH as f32, COMPUTE_TEXTURE_HEIGHT as f32),
        );
        let vertex_render_input_uniform = Uniform::new(gl, &vertex_render_program, "u_input", (0,));
        let vertex_render_scale_uniform =
            Uniform::new(gl, &vertex_render_program, "u_scale", (1.0, 1.0));
        let vertex_render_color_uniform =
            Uniform::new(gl, &vertex_render_program, "u_color", (1.0, 1.0, 1.0, 1.0));

        let vertices: Vec<f32> = (0..2_u32.pow(MAX_RECURSION_DEPTH + 2))
            .map(|i| i as f32)
            .collect();
        let verts = web_sys::js_sys::Float32Array::from(vertices.as_slice());
        let vertex_render_vertex_buffer = gl.create_buffer().unwrap();

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_render_vertex_buffer));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &verts, GL::STATIC_DRAW);

        FractalClockRenderState {
            vertex_compute_program,
            vertex_compute_input_buffer,
            vertex_render_program,
            vertex_render_dimensions_uniform,
            vertex_render_input_uniform,
            vertex_render_scale_uniform,
            vertex_render_color_uniform,
            vertex_render_vertex_buffer,
        }
    }
}
