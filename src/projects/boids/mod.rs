use yew::prelude::*;

mod render;

use crate::projects::{
    ProjectSite,
    boids::render::{BoidsRenderInput, BoidsRenderer},
    interactive::{InteractiveExample, Slider},
};

#[function_component(BoidsPage)]
pub fn boids_page() -> Html {
    let cohesion = use_state(|| 0.5);
    let separation = use_state(|| 0.5);
    let alignment = use_state(|| 0.5);
    let edge_avoidance = use_state(|| 0.5);
    let avoidance_radius = use_state(|| 0.1);
    let detection_radius = use_state(|| 0.2);
    let min_velocity = use_state(|| 0.005);
    let max_velocity = use_state(|| 0.005);
    let max_acceleration = use_state(|| 0.005);

    let render_input = BoidsRenderInput {
        cohesion: *cohesion,
        separation: *separation,
        alignment: *alignment,
        edge_avoidance: *edge_avoidance,
        avoidance_radius: *avoidance_radius,
        detection_radius: *detection_radius,
        min_velocity: *min_velocity,
        max_velocity: *max_velocity,
        max_acceleration: *max_acceleration,
    };

    let settings = vec![
        (
            "Cohesion".to_string(),
            html! { <Slider<f32> min={0.0} max={1.0} step={0.1} value={cohesion}/> },
        ),
        (
            "Separation".to_string(),
            html! { <Slider<f32> min={0.0} max={1.0} step={0.1} value={separation}/> },
        ),
        (
            "Alignment".to_string(),
            html! { <Slider<f32> min={0.0} max={1.0} step={0.1} value={alignment}/> },
        ),
        (
            "Edge Avoidance".to_string(),
            html! { <Slider<f32> min={0.0} max={1.0} step={0.1} value={edge_avoidance}/> },
        ),
        (
            "Detection Radius".to_string(),
            html! { <Slider<f32> min={0.0} max={1.0} step={0.1} value={detection_radius}/> },
        ),
        (
            "Avoidance Radius".to_string(),
            html! { <Slider<f32> min={0.0} max={1.0} step={0.1} value={avoidance_radius}/> },
        ),
        (
            "Minimum Velocity".to_string(),
            html! { <Slider<f32> min={0.0} max={0.1} step={0.005} value={min_velocity}/> },
        ),
        (
            "Maximum Velocity".to_string(),
            html! { <Slider<f32> min={0.0} max={0.1} step={0.005} value={max_velocity}/> },
        ),
        (
            "Maximum Acceleration".to_string(),
            html! { <Slider<f32> min={0.0} max={0.1} step={0.005} value={max_acceleration}/> },
        ),
    ];

    html! {
        <ProjectSite title="Boids">
            <InteractiveExample<BoidsRenderer>
                renderer={BoidsRenderer {}}
                render_input={render_input.clone()}
                initially_active=true
                settings={settings.clone()}
            />
        </ProjectSite>
    }
}
