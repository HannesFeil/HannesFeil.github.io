//! Components for creating interactive interfaces

use std::{cell::LazyCell, rc::Rc, sync::Mutex};

use gloo::{events::EventListener, utils::window};
use stylist::yew::use_style;
use web_sys::{HtmlCanvasElement, HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use crate::{
    use_theme,
    webgl::{Canvas, CanvasRenderer, RenderLoopState},
};

/// A scroll event listener, notifying a list of callbacks
struct ScrollEventListener {
    _listener: EventListener,
    callbacks: Rc<Mutex<Vec<Callback<Event>>>>,
}

impl ScrollEventListener {
    /// Create a new `ScrollEventListener`, registering it for the window
    fn new() -> Self {
        let callbacks: Rc<Mutex<Vec<Callback<Event>>>> = Rc::new(Mutex::new(Vec::default()));

        ScrollEventListener {
            _listener: EventListener::new(&window(), "scroll", {
                let callbacks = callbacks.clone();
                move |event| {
                    for cb in callbacks.lock().unwrap().iter() {
                        cb.emit(event.clone());
                    }
                }
            }),
            callbacks,
        }
    }
}

thread_local! {
    /// The unique scroll listener used throughout this website
    static SCROLL_EVENT_LISTENER: LazyCell<ScrollEventListener> = LazyCell::new(ScrollEventListener::new);
}

/// Register a callback to the unique [`ScrollEventListener`].
///
/// The callback gets unregistered automatically when the hook is no more in use.
#[hook]
pub fn use_scroll_event_listener(callback: impl Fn(Event) + 'static) {
    let callback = Callback::from(callback);

    use_effect_with(callback, |callback| {
        let callback = callback.clone();
        let callback_clone = callback.clone();

        SCROLL_EVENT_LISTENER.with(move |listener| {
            let mut callbacks = listener.callbacks.lock().unwrap();
            callbacks.push(callback_clone);
        });

        move || {
            SCROLL_EVENT_LISTENER.with(|listener| {
                listener
                    .callbacks
                    .lock()
                    .unwrap()
                    .retain(|cb| *cb != callback)
            });
        }
    });
}

/// Properties for the [`InteractiveExample`] component
#[derive(Properties, PartialEq)]
pub struct InteractiveExampleProperties<R: CanvasRenderer> {
    /// The renderer used on this [Canvas]
    pub renderer: R,
    /// Input to the renderer
    pub render_input: R::RenderInput,
    #[prop_or_default]
    /// Whether this example is initially active
    pub initially_active: bool,
    /// Settings for this example, components and their labels
    pub settings: Vec<(String, Html)>,
}

/// An interactive example.
///
/// This is mostly a wrapper around a [`Canvas`]
#[function_component(InteractiveExample)]
pub fn interactive_example<R: CanvasRenderer>(props: &InteractiveExampleProperties<R>) -> Html {
    let canvas_node_ref = use_node_ref();
    let visible = use_state(|| props.initially_active);

    use_scroll_event_listener({
        let visible = visible.clone();
        let canvas_node_ref = canvas_node_ref.clone();

        move |_| {
            if let Some(canvas) = canvas_node_ref.cast::<HtmlCanvasElement>() {
                let bounding_rect = canvas.get_bounding_client_rect();
                let window_height = window().inner_height().unwrap().as_f64().unwrap();

                let on_screen = bounding_rect.top() >= -bounding_rect.height()
                    && bounding_rect.bottom() <= window_height + bounding_rect.height();
                visible.set(on_screen);
            } else {
                panic!("Canvas should exist");
            }
        }
    });

    let full_screen_canvas = Callback::from({
        let canvas_node_ref = canvas_node_ref.clone();

        move |_| {
            if let Some(canvas) = canvas_node_ref.cast::<HtmlCanvasElement>() {
                canvas.request_fullscreen().unwrap();
            } else {
                panic!("Canvas should exist");
            }
        }
    });

    let render_loop_state = if *visible {
        RenderLoopState::Rendering
    } else {
        RenderLoopState::Finished
    };

    let theme = use_theme();
    let style = use_style!(
        r#"
            display: grid;
            row-gap: 0;
            position: relative;
        
            .settings {
                display: grid;
                grid-template-columns: max-content auto max-content auto;
                column-gap: 20px;
                background-color: ${bg};
                padding: 10px 20px;
                align-items: center;
            }

            .settings * {
                font-size: 13px;
            }

            .full-screen-button {
                position: absolute;
                top: 10px;
                right: 10px;
                color: ${full_screen_button_fg};
                background-color: transparent;
                border: none;
            }

            .full-screen-button:hover {
                color: ${full_screen_button_fg_hover};
            }

            .full-screen-button i {
                font-size: 32px;
            }
        "#,
        bg = theme.base00,
        full_screen_button_fg = theme.base04,
        full_screen_button_fg_hover = theme.base07,
    );
    let settings = props.settings.iter().map(|(key, html)| {
        html! {
            <>
                <label>{key}</label>
                {html.clone()}
            </>
        }
    });
    html! {
        <div class={style}>
            <button class="full-screen-button" onclick={full_screen_canvas}>
                <i class="iconoir-plus-square"/>
            </button>
            <Canvas<R>
                canvas_node_ref={canvas_node_ref.clone()}
                renderer={props.renderer.clone()}
                render_input={props.render_input.clone()}
                width="100%"
                height="500px"
                {render_loop_state}
            />
            <div class="settings">
                {for settings}
            </div>
        </div>
    }
}

/// Allows a type to be used with [`Slider`]
pub trait SliderValue
where
    Self: PartialEq + PartialOrd + 'static,
{
    /// The value one
    const ONE: Self;

    /// Converts self to a js number
    fn to_js_number_string(&self) -> String;

    /// Converts to self from a js number
    fn from_js_number_string(value: String) -> Self;
}

impl SliderValue for u32 {
    const ONE: Self = 1;

    fn to_js_number_string(&self) -> String {
        self.to_string()
    }

    fn from_js_number_string(value: String) -> Self {
        value.parse().unwrap()
    }
}

impl SliderValue for f32 {
    const ONE: Self = 1.0;

    fn to_js_number_string(&self) -> String {
        self.to_string()
    }

    fn from_js_number_string(value: String) -> Self {
        value.parse().unwrap()
    }
}

/// Properties for the [`Slider`] component
#[derive(Debug, PartialEq, Properties)]
pub struct SliderProperties<T: SliderValue> {
    /// Whether the component is active
    #[prop_or(true)]
    pub active: bool,
    /// The minimum value
    pub min: T,
    /// The maximum value
    pub max: T,
    /// The step value
    #[prop_or(T::ONE)]
    pub step: T,
    /// The selected value
    pub value: UseStateHandle<T>,
}

/// A slider component used to select a value in a range
#[function_component(Slider)]
pub fn slider<T: SliderValue>(
    SliderProperties {
        active,
        min,
        max,
        value,
        step,
    }: &SliderProperties<T>,
) -> Html {
    let theme = use_theme();
    let style = use_style!(
        r#"
            width: 100%;
            display: grid;
            grid-template-columns: max-content auto max-content;
            column-gap: 10px;
            align-items: center;

            p {
                color: ${fg};
            }
        "#,
        fg = theme.base04,
    );
    let on_input = Callback::from({
        let value = value.clone();

        move |event: InputEvent| {
            value.set(T::from_js_number_string(
                event.target_dyn_into::<HtmlInputElement>().unwrap().value(),
            ));
        }
    });
    html! {
        <div class={style}>
            <p>{min.to_js_number_string()}</p>
            <input
                type="range"
                disabled={!active}
                min={min.to_js_number_string()}
                max={max.to_js_number_string()}
                step={step.to_js_number_string()}
                value={value.to_js_number_string()}
                oninput={on_input}
            />
            <p>{max.to_js_number_string()}</p>
        </div>
    }
}

/// Properties for the [`Checkbox`] component
#[derive(Debug, Properties, PartialEq)]
pub struct CheckboxProperties {
    /// Whether the checkbox is active
    #[prop_or(true)]
    pub active: bool,
    /// The checked state of the checkbox
    pub value: UseStateHandle<bool>,
}

/// A checkbox component to enable or disable stuff
#[function_component(Checkbox)]
pub fn checkbox(CheckboxProperties { active, value }: &CheckboxProperties) -> Html {
    let style = use_style!(
        r#"
            height: 20px;
            width: 20px;
        "#
    );

    let on_input = Callback::from({
        let value = value.clone();

        move |event: InputEvent| {
            value.set(
                event
                    .target_dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .checked(),
            )
        }
    });

    html! {
        <input
            class={style}
            type="checkbox"
            disabled={!active}
            value={value.to_string()}
            checked={**value}
            oninput={on_input}
        />
    }
}

/// Properties for the [`ColorPicker`] component
#[derive(Debug, Properties, PartialEq)]
pub struct ColorPickerProperties {
    /// Whether the color picker is actuve
    #[prop_or(true)]
    pub active: bool,
    /// The selected color as a css string
    pub value: UseStateHandle<String>,
}

/// A color picker component used to select a color
#[function_component(ColorPicker)]
pub fn color_picker(ColorPickerProperties { active, value }: &ColorPickerProperties) -> Html {
    let style = use_style!(
        r#"
        "#
    );

    let on_input = Callback::from({
        let value = value.clone();

        move |event: InputEvent| {
            value.set(event.target_dyn_into::<HtmlInputElement>().unwrap().value())
        }
    });

    html! {
        <input
            class={style}
            type="color"
            disabled={!active}
            value={value.to_string()}
            oninput={on_input}
        />
    }
}

/// Properties for the [`Selection`] component
#[derive(Debug, Properties, PartialEq)]
pub struct SelectionProperties<T: ToString + PartialEq + Clone + 'static> {
    /// Whether the component is active
    #[prop_or(true)]
    pub active: bool,
    /// The currently selected value
    pub value: UseStateHandle<T>,
    /// The possible values
    pub values: Box<[T]>,
}

/// A component used for selecting values from a list of possible values
#[function_component(Selection)]
pub fn selection<T: ToString + PartialEq + Clone + 'static>(
    SelectionProperties {
        active,
        value,
        values,
    }: &SelectionProperties<T>,
) -> Html {
    let options = values.iter().map(|v| {
        html! { <option selected={*v == **value}>{ v.to_string() }</option> }
    });
    let style = use_style!(
        r#"
            height: 30px;
            margin: 10px 0px;
        "#
    );

    let on_input = Callback::from({
        let value = value.clone();
        let values = values.clone();

        move |event: InputEvent| {
            value.set(
                values[usize::try_from(
                    event
                        .target_dyn_into::<HtmlSelectElement>()
                        .unwrap()
                        .selected_index(),
                )
                .unwrap()]
                .clone(),
            );
        }
    });

    html! {
        <select disabled={!active} oninput={on_input} class={style}>
            {for options}
        </select>
    }
}
