use std::{collections::HashMap, rc::Rc};

use crate::{
    navigation::{Route, Section},
    projects::{
        CodeExample, ProjectSite,
        fractal_clock::render::{
            BLEND_EQUATIONS, BLEND_MULTIPLIERS, BlendConstant, FractalClockRenderInput,
            FractalClockRenderer, MAX_RECURSION_DEPTH,
        },
        interactive::{Checkbox, ColorPicker, InteractiveExample, Selection, Slider},
    },
};

use color::AlphaColor;
use yew::prelude::*;
use yew_router::prelude::Link;

mod render;

const HOUR_ANGLE_SETTING: &str = "Hour angle";
const MINUTE_ANGLE_SETTING: &str = "Minute angle";
const ANIMATE_SETTING: &str = "Animate";
const SIZE_SETTING: &str = "Size";
const HOUR_RATIO_SETTING: &str = "Hour ratio";
const RECURSION_DEPTH_SETTING: &str = "Recursion depth";
const SIZE_FACTOR_SETTING: &str = "Size factor";
const COLOR_SETTING: &str = "Color";
const ALPHA_SETTING: &str = "Alpha";
const RGB_BLEND_SETTING: &str = "RGB blend";
const ALPHA_BLEND_SETTING: &str = "Alpha blend";
const SOURCE_RGB_SETTING: &str = "Source RGB";
const SOURCE_ALPHA_SETTING: &str = "Source Alpha";
const DESTINATION_RGB_SETTING: &str = "Destination RGB";
const DESTINATION_ALPHA_SETTING: &str = "Destination Alpha";

#[function_component(FractalClockPage)]
pub fn fractal_clock_page() -> Html {
    // Define shared example settings
    let hour_angle = use_state(|| 310.0);
    let minute_angle = use_state(|| 60.0);
    let animate = use_state(|| true);
    let size = use_state(|| 1.0);
    let recursion_depth = use_state(|| 8);
    let hour_ratio = use_state(|| 0.75);
    let size_factor = use_state(|| 0.75);
    let color = use_state(|| "#40ff20".to_owned());
    let alpha = use_state(|| 0.5);
    let blend_equations: Box<[_]> = BLEND_EQUATIONS.iter().copied().collect();
    let blend_multipliers: Box<[_]> = BLEND_MULTIPLIERS.iter().copied().collect();
    let blend_equation_1 = use_state(|| BlendConstant::Addition);
    let blend_equation_2 = use_state(|| BlendConstant::Addition);
    let blend_multiplier_1 = use_state(|| BlendConstant::SourceAlpha);
    let blend_multiplier_2 = use_state(|| BlendConstant::DestinationAlpha);
    let blend_multiplier_3 = use_state(|| BlendConstant::One);
    let blend_multiplier_4 = use_state(|| BlendConstant::One);

    let settings: Rc<HashMap<_, _>> = Rc::new([
            (
                "Hour angle".to_string(),
                html! { <Slider<f32> active={!*animate} min={0.0} max={360.0} step={0.1} value={hour_angle.clone()}/> },
            ),
            (
                "Minute angle".to_string(),
                html! { <Slider<f32> active={!*animate} min={0.0} max={360.0} step={0.1} value={minute_angle.clone()}/> },
            ),
            (
                "Animate".to_string(),
                html! { <Checkbox value={animate.clone()}/> },
            ),
            (
                "Size".to_string(),
                html! { <Slider<f32> min={1.0} max={10.0} step={0.1} value={size.clone()}/> },
            ),
            (
                "Hour ratio".to_string(),
                html! { <Slider<f32> min={0.0} max={1.0} step={0.01} value={hour_ratio.clone()}/> },
            ),
            (
                "Recursion depth".to_string(),
                html! { <Slider<u32> min={1} max={MAX_RECURSION_DEPTH} step={1} value={recursion_depth.clone()}/> },
            ),
            (
                "Size factor".to_string(),
                html! { <Slider<f32> min={0.0} max={0.99} step={0.01} value={size_factor.clone()}/> },
            ),
            (
                "Color".to_string(),
                html! { <ColorPicker value={color.clone()}/> },
            ),
            (
                "Alpha".to_string(),
                html! { <Slider<f32> min={0.0} max={1.0} step={0.01} value={alpha.clone()}/> },
            ),
            (
                "RGB blend".to_string(),
                html! { <Selection<BlendConstant> value={blend_equation_1.clone()} values={blend_equations.clone()}/> },
            ),
            (
                "Alpha blend".to_string(),
                html! { <Selection<BlendConstant> value={blend_equation_2.clone()} values={blend_equations.clone()}/> },
            ),
            (
                "Source RGB".to_string(),
                html! { <Selection<BlendConstant> value={blend_multiplier_1.clone()} values={blend_multipliers.clone()}/> },
            ),
            (
                "Source Alpha".to_string(),
                html! { <Selection<BlendConstant> value={blend_multiplier_2.clone()} values={blend_multipliers.clone()}/> },
            ),
            (
                "Destination RGB".to_string(),
                html! { <Selection<BlendConstant> value={blend_multiplier_3.clone()} values={blend_multipliers.clone()}/> },
            ),
            (
                "Destination Alpha".to_string(),
                html! { <Selection<BlendConstant> value={blend_multiplier_4.clone()} values={blend_multipliers.clone()}/> },
            ),
        ].into_iter().collect());

    let col = color::parse_color(&color)
        .unwrap()
        .to_alpha_color::<color::Srgb>()
        .with_alpha(*alpha);

    let final_render_input = Rc::new(FractalClockRenderInput {
        hour_angle: *hour_angle,
        minute_angle: *minute_angle,
        animate: *animate,
        size: *size,
        recursion_depth: *recursion_depth,
        hour_ratio: *hour_ratio,
        size_factor: *size_factor,
        color: col,
        blend_equations: (*blend_equation_1, *blend_equation_2),
        blend_multipliers: (
            *blend_multiplier_1,
            *blend_multiplier_2,
            *blend_multiplier_3,
            *blend_multiplier_4,
        ),
    });

    html! {
        <ProjectSite title="Fractal Clock">
            <Section title="Introduction">
                <p>
                    {"
                        Some while ago I stumbled upon a
                    "}
                    <a href="https://www.youtube.com/watch?v=4SH_-YhN15A">{"Video"}</a>
                    {"
                        by Code Parade about a particular way to visualize a clock. Since it looked
                        cool I thought why not give implementing it a try? I took this challenge as
                        an opportunity to learn about
                    "}
                    <a href="https://wgpu.rs/">{"wgpu.rs"}</a>
                    {"
                        although I will mostly focus on the clock and not dive into detail about
                        shaders etc.
                    "}
                </p>
                <p>
                    {"
                        Conceptually the visualization recursively draws analogue clocks at the
                        end of each clock's pointer (The previously mentioned video does a good job
                        explaining the concept at the beginning). Another way to think about it is
                        imagining each pointer as a
                    "}
                    <a href="https://en.wikipedia.org/wiki/Fractal_canopy">{"Fractal canopy"}</a>
                    {"."}
                </p>
                <p>
                    {"
                        Just a quick note: The source code of this website is accessible to anyone
                        interested in the implementation used throughout the explanation, see 
                    "}
                    <Link<Route> to={Route::About}>{"About"}</Link<Route>>
                    {"."}
                </p>
            </Section>
            <Section title="Implementation Basics">
                <p>
                    {"
                        Starting off we will be working with basic vector math. We will treat each
                        base pointer as a vector of length 1 (and a shorter length for the hour
                        pointer), with an appropriate angle derived from the current time (Not
                        accurate to the actual time for demonstration purposes).
                    "}
                </p>
                <CodeExample lang="Rust">
                    {indoc::indoc! {
                        r#"
                            let (hour_angle, hour_ratio, minute_angle) = // Implementation details
                            let hour_pointer = (hour_angle.cos() * hour_ratio, hour_angle.sin() * hour_ratio);
                            let minute_pointer = (minute_angle.cos(), minute_angle.sin());
                        "#
                    }}
                </CodeExample>
                <FractalClockExample
                    version={ExampleVersion::Trivial}
                    final_render_input={final_render_input.clone()}
                    settings={settings.clone()}
                    initially_active=true
                />
                <p>
                    {"
                        So far so good, this already looks like a minimal analogue clock. Now comes
                        the interesting part: recursively computing hour and minute pointers. To
                        draw a line of course, we need two points but since the starting point
                        is always the tip of a previous pointer (or the origin), we don't need to
                        compute them again.  So how do we compute the next pointer recursively on
                        top of a previous one? Technically the following should work:
                    "}
                </p>
                <CodeExample lang="Rust">
                    {indoc::indoc! {
                        r#"
                            let (prev_pointer_x, prev_pointer_y, prev_pointer_angle) = // ...
                            let next_pointer_origin = (prev_pointer_x, prev_pointer_y);
                            let next_hour_pointer_angle =
                                prev_pointer_angle + hour_angle; // Since each recursive clock is rotated
                            let next_hour_pointer = (
                                next_pointer_origin + next_hour_pointer_angle.cos(),
                                next_pointer_origin + next_hour_pointer_angle.sin()
                            )
                            // Similar for the minute pointer ...
                        "#
                    }}
                </CodeExample>
                <p>
                    {"
                        However we need to carry the pointer angles around, and repeatedly calculate
                        sinus and cosinus functions. To avoid those hassles, we can use
                    "}
                    <a href="https://en.wikipedia.org/wiki/Complex_number">{"Complex Numbers"}</a>
                    {"
                        :)
                    "}
                </p>
            </Section>
            <Section title="Complex numbers">
                <p>
                    {"
                        I have sneakily already defined our vectors in a way that resembles a
                        complex number, derived from it's polar form. We can now use the property,
                        that multiplying two complex numbers is equivalent to adding their angles in
                        polar form and multiplying their lengths.
                    "}
                </p>
                <p>
                    {"
                        Additionally we get a property for free that I forgot to mention before:
                        Each subsequent pointer should have a smaller length. The following is an
                        example of drawing the first recursive set of pointers (Don't be confused by
                        the size factor scaling the entire clock to fit it on the canvas):
                    "}
                </p>
                <CodeExample lang="Rust">
                    {indoc::indoc! {
                        r#"
                            let (hour_x, hour_y, minute_x, minute_y) = //...
                            let (prev_pointer_x, prev_pointer_y) = // ...
                            let next_pointer_origin = (prev_pointer_x, prev_pointer_y);
                            // We cannot add the origin offset with this method, but
                            // since we need the previous point for rendering anyway
                            // we can simply add it during rendering
                            let next_hour_pointer = (
                                prev_pointer_x * hour_x - prev_pointer_y * hour_y,
                                prev_pointer_x * hour_y + prev_pointer_y * hour_x
                            );
                            // Similar for the minute pointer ...
                        "#
                    }}
                </CodeExample>
                <FractalClockExample
                    version={ExampleVersion::TrivialRecursive(false)}
                    final_render_input={final_render_input.clone()}
                    settings={settings.clone()}
                />
            </Section>
            <Section title="Recursion">
                <p>
                    {"
                        Not that we know how to calculate deeper pointers, it's time to do it
                        recursively right? I mentioned in the beginning, that I used WebGPU to
                        compute the fractal clock on a graphics card. This allows a lot of vertices
                        to be efficiently computed, however shader code does not support recursion.
                        To get rid of the recursion, we can iteratively compute each recursion-layer
                        where each time the number of vertices (pointers) computed is doubled.
                    "}
                </p>
                <p>
                    {"
                        Note: Since the first layers contain only a few vertices, it is faster to
                        compute them on the CPU before sending them to the GPU, since each layer
                        needs a seperate pass to the GPU. Also I've scaled the clock depending on
                        the recursion depth to completely fit on screen.
                    "}
                </p>
                <FractalClockExample
                    version={ExampleVersion::TrivialRecursive(true)}
                    final_render_input={final_render_input.clone()}
                    settings={settings.clone()}
                />
            </Section>
            <Section title="Colors">
                <p>
                    {"
                        The previous example already looks functionally correct, now it's time to
                        make it pretty :3 The rendering pipeline offers a few screws we can turn,
                        starting with the actual drawing color.
                    "}
                </p>
                <p>
                    {"
                        Additionally I've added a size slider to zoom into the clock in case someone
                        want's to inspect some clock states in more detail.
                    "}
                </p>
                <FractalClockExample
                    version={ExampleVersion::CompleteWithoutBlending}
                    final_render_input={final_render_input.clone()}
                    settings={settings.clone()}
                />
            </Section>
            <Section title="Blending">
                <p>
                    {"
                        Finally the render pipeline allows us to play with the blending of colors.
                        The color and alpha part of the final result is calculated seperately: Both
                        use a function (like addition or subtraction) and some factors with which
                        to multiply the source value (from the data that is being drawn) and the
                        destination value (from the data that's already there).
                    "}
                </p>
                <p>
                    {"
                        The default blend settings I chose could be written as follows:
                    "}
                </p>
                <CodeExample lang="Rust">
                    {indoc::indoc! {
                        r#"
                            // Function is addition so we have formulas of the form
                            // FACTOR * src_component + FACTOR * dst_component
                            let final_rgb = src_alpha * src_rgb + 1 * dst_rgb;
                            let final_alpha = dst_alpha * src_alpha + 1 * dst_alpha;
                        "#
                    }}
                </CodeExample>
                <FractalClockExample
                    version={ExampleVersion::Complete}
                    final_render_input={final_render_input.clone()}
                    settings={settings.clone()}
                />
            </Section>
            <Section title="Conclusion">
                <p>
                    {"
                        And that's it for this little Codling :) I hope maybe this inspires you to expand
                        on the idea of a fractal clock, since I only implemented the basic functionality.
                        I also want to thank
                    "}
                    <a href="https://www.youtube.com/c/codeparade">{"Code Parade"}</a>
                    {"
                        again for introducing me to this idea and also many of his other videos, which I
                        would recommend you to watch if you found this interesting.
                    "}
                </p>
            </Section>
        </ProjectSite>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExampleVersion {
    Trivial,
    TrivialRecursive(bool),
    CompleteWithoutBlending,
    Complete,
}

#[derive(Debug, PartialEq, Properties)]
struct FractalClockExampleProperties {
    version: ExampleVersion,
    final_render_input: Rc<FractalClockRenderInput>,
    settings: Rc<HashMap<String, Html>>,
    #[prop_or_default]
    initially_active: bool,
}

#[function_component(FractalClockExample)]
fn fractal_clock_example(props: &FractalClockExampleProperties) -> Html {
    let render_input = match props.version {
        ExampleVersion::Trivial => FractalClockRenderInput {
            size: 1.0,
            recursion_depth: 1,
            size_factor: 0.75,
            color: AlphaColor::from_rgba8(255, 255, 255, 255),
            blend_equations: (BlendConstant::Addition, BlendConstant::Addition),
            blend_multipliers: (
                BlendConstant::One,
                BlendConstant::Zero,
                BlendConstant::One,
                BlendConstant::Zero,
            ),
            ..*props.final_render_input
        },
        ExampleVersion::TrivialRecursive(custom_recursion) => FractalClockRenderInput {
            size: 1.0,
            recursion_depth: if custom_recursion {
                props.final_render_input.recursion_depth
            } else {
                2
            },
            color: AlphaColor::from_rgba8(255, 255, 255, 255),
            blend_equations: (BlendConstant::Addition, BlendConstant::Addition),
            blend_multipliers: (
                BlendConstant::One,
                BlendConstant::Zero,
                BlendConstant::One,
                BlendConstant::Zero,
            ),
            ..*props.final_render_input
        },
        ExampleVersion::CompleteWithoutBlending => FractalClockRenderInput {
            blend_equations: (BlendConstant::Addition, BlendConstant::Addition),
            blend_multipliers: (
                BlendConstant::One,
                BlendConstant::Zero,
                BlendConstant::One,
                BlendConstant::Zero,
            ),
            ..*props.final_render_input
        },
        ExampleVersion::Complete => (*props.final_render_input).clone(),
    };
    const TRIVIAL_SETTINGS: &[&str] = &[
        HOUR_ANGLE_SETTING,
        MINUTE_ANGLE_SETTING,
        ANIMATE_SETTING,
        HOUR_RATIO_SETTING,
    ];
    const TRIVIAL_RECURSION_SETTINGS: &[&str] = &[
        HOUR_ANGLE_SETTING,
        MINUTE_ANGLE_SETTING,
        ANIMATE_SETTING,
        HOUR_RATIO_SETTING,
        SIZE_FACTOR_SETTING,
        RECURSION_DEPTH_SETTING,
    ];
    const COMPLETE_SETTINGS: &[&str] = &[
        HOUR_ANGLE_SETTING,
        MINUTE_ANGLE_SETTING,
        ANIMATE_SETTING,
        HOUR_RATIO_SETTING,
        SIZE_SETTING,
        SIZE_FACTOR_SETTING,
        RECURSION_DEPTH_SETTING,
        COLOR_SETTING,
        ALPHA_SETTING,
        RGB_BLEND_SETTING,
        ALPHA_BLEND_SETTING,
        SOURCE_RGB_SETTING,
        SOURCE_ALPHA_SETTING,
        DESTINATION_RGB_SETTING,
        DESTINATION_ALPHA_SETTING,
    ];
    let settings_filter: &[&str] = match props.version {
        ExampleVersion::Trivial => TRIVIAL_SETTINGS,
        ExampleVersion::TrivialRecursive(false) => &TRIVIAL_RECURSION_SETTINGS[..5],
        ExampleVersion::TrivialRecursive(true) => TRIVIAL_RECURSION_SETTINGS,
        ExampleVersion::CompleteWithoutBlending => &COMPLETE_SETTINGS[..8],
        ExampleVersion::Complete => COMPLETE_SETTINGS,
    };
    let settings: Vec<_> = settings_filter
        .iter()
        .map(|&setting| {
            (
                setting.to_owned(),
                props.settings.get(setting).unwrap().clone(),
            )
        })
        .collect();
    html! {
        <InteractiveExample<FractalClockRenderer>
            renderer={FractalClockRenderer::default()}
            {render_input}
            initially_active={props.initially_active}
            {settings}
        />
    }
}
