use web_sys::js_sys::Math::random;
use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext as GL};

use crate::uniform_set;
use crate::webgl::{CanvasRenderer, RenderData, Uniform, create_program};
use crate::webgl::{ComputeProgram, compile_shader};

uniform_set! {
    ComputeUniformSet {
        u_space: (f32, f32),
        u_cohesion: (f32,),
        u_separation: (f32,),
        u_alignment: (f32,),
        u_edge_avoidance: (f32,),
        u_avoidance_radius: (f32,),
        u_detection_radius: (f32,),
        u_min_velocity: (f32,),
        u_max_velocity: (f32,),
        u_max_acceleration: (f32,),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoidsRenderer {}

#[derive(Debug)]
pub struct BoidsRenderState {
    compute_program: ComputeProgram<ComputeUniformSet>,
    render_program: WebGlProgram,
    render_vertex_buffer: WebGlBuffer,
    render_dimensions_uniform: Uniform<(f32, f32)>,
    render_input_uniform: Uniform<(i32,)>,
    render_aspect_uniform: Uniform<(f32,)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoidsRenderInput {
    /// Weight for boids being attracted to the group center of mass
    pub cohesion: f32,
    /// Weight for boids being repelled by each other
    pub separation: f32,
    /// Weight for boids aligning to the same direction
    pub alignment: f32,
    /// Weight for boids avoiding edges
    pub edge_avoidance: f32,
    /// Radius for boids avoiding each other
    pub avoidance_radius: f32,
    /// Radius for boids vision
    pub detection_radius: f32,
    /// Minimum boid velocity
    pub min_velocity: f32,
    /// Maximum boid velocity
    pub max_velocity: f32,
    /// Maximum boid acceleration
    pub max_acceleration: f32,
}

impl CanvasRenderer for BoidsRenderer {
    type RenderState = BoidsRenderState;

    type RenderInput = BoidsRenderInput;

    fn render(
        &self,
        state: &mut Self::RenderState,
        _input: &Self::RenderInput,
        gl: &GL,
        RenderData {
            width,
            height,
            resized,
            input_changed,
            ..
        }: RenderData,
    ) {
        // if resized {
        //     gl.viewport(0, 0, width.try_into().unwrap(), height.try_into().unwrap());
        // }

        if input_changed {
            log::info!("Input changed");
        }

        let aspect = height as f32 / width as f32;

        state
            .compute_program
            .set_uniform::<{ ComputeUniformSet::u_space }>((1.0 / aspect, 1.0));
        state
            .compute_program
            .set_uniform::<{ ComputeUniformSet::u_cohesion }>((_input.cohesion,));
        state
            .compute_program
            .set_uniform::<{ ComputeUniformSet::u_separation }>((_input.separation,));
        state
            .compute_program
            .set_uniform::<{ ComputeUniformSet::u_alignment }>((_input.alignment,));
        state
            .compute_program
            .set_uniform::<{ ComputeUniformSet::u_edge_avoidance }>((_input.edge_avoidance,));
        state
            .compute_program
            .set_uniform::<{ ComputeUniformSet::u_detection_radius }>((_input.detection_radius,));
        state
            .compute_program
            .set_uniform::<{ ComputeUniformSet::u_avoidance_radius }>((_input.avoidance_radius,));
        state
            .compute_program
            .set_uniform::<{ ComputeUniformSet::u_min_velocity }>((_input.min_velocity,));
        state
            .compute_program
            .set_uniform::<{ ComputeUniformSet::u_max_velocity }>((_input.max_velocity,));
        state
            .compute_program
            .set_uniform::<{ ComputeUniformSet::u_max_acceleration }>((_input.max_acceleration,));

        state.compute_program.compute(gl);
        state.compute_program.copy_output_to_input(gl, 0);

        gl.use_program(Some(&state.render_program));
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&state.render_vertex_buffer));
        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, Some(state.compute_program.output_texture()));

        let position = gl
            .get_attrib_location(&state.render_program, "a_index")
            .try_into()
            .unwrap();
        gl.vertex_attrib_pointer_with_i32(position, 1, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(position);

        state.render_dimensions_uniform.apply(gl);
        state.render_input_uniform.apply(gl);
        state.render_aspect_uniform.apply_data(gl, (aspect,));

        gl.viewport(0, 0, width.try_into().unwrap(), height.try_into().unwrap());
        gl.clear_color(0.0, 0.0, 0.0, 0.0);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        gl.draw_arrays(GL::TRIANGLES, 0, 300);
    }

    fn initial_render_state(
        &self,
        _input: &Self::RenderInput,
        gl: &GL,
        _render_data: RenderData,
    ) -> Self::RenderState {
        const COMPUTE_FRAG_SOURCE: &str = include_str!("./compute.frag");
        const RENDER_VERT_SOURCE: &str = include_str!("./render.vert");
        const RENDER_FRAG_SOURCE: &str = include_str!("./render.frag");

        log::info!("Starting initial setup");

        let compute_program = ComputeProgram::new(10, 10, 1, gl, COMPUTE_FRAG_SOURCE);
        let initial_data: Vec<_> = (0..100)
            .flat_map(|_| {
                [
                    (2.0 * random() - 1.0) as f32,
                    (2.0 * random() - 1.0) as f32,
                    (2.0 * random() - 1.0) as f32,
                    (2.0 * random() - 1.0) as f32,
                ]
            })
            .collect();
        compute_program.write_input(gl, 0, initial_data.as_slice());

        let render_vertex_shader =
            compile_shader(gl, GL::VERTEX_SHADER, RENDER_VERT_SOURCE).unwrap();
        let render_fragment_shader =
            compile_shader(gl, GL::FRAGMENT_SHADER, RENDER_FRAG_SOURCE).unwrap();
        let render_program =
            create_program(gl, &render_vertex_shader, &render_fragment_shader).unwrap();

        let render_dimensions_uniform =
            Uniform::new(gl, &render_program, "u_dimensions", (10.0, 10.0));
        let render_input_uniform = Uniform::new(gl, &render_program, "u_input", (0,));
        let render_aspect_uniform = Uniform::new(gl, &render_program, "u_aspect", (0.0,));

        let vertices: Vec<f32> = (0..300).map(|i| i as f32).collect();
        let verts = web_sys::js_sys::Float32Array::from(vertices.as_slice());
        let render_vertex_buffer = gl.create_buffer().unwrap();

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&render_vertex_buffer));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &verts, GL::STATIC_DRAW);

        log::info!("Initial setup complete");

        BoidsRenderState {
            compute_program,
            render_program,
            render_vertex_buffer,
            render_dimensions_uniform,
            render_input_uniform,
            render_aspect_uniform,
        }
    }
}
