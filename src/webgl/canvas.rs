//! Canvas webgl rendering framework

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use gloo::utils::window;
use stylist::css;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use web_sys::WebGlRenderingContext as GL;
use yew::html;
use yew::prelude::*;

/// The state of the rendering loop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderLoopState {
    /// Currently rendering each frame
    Rendering,
    /// Not rendering
    Paused,
    /// About to terminate the loop
    Finished,
}

/// Data about the last mouse state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MouseData {
    /// Whether mouse button 1 is down
    pub primary_button: bool,
    /// Whether mouse button 2 is down
    pub secondary_button: bool,
    /// The mouse position relative to this canvas (None if not on the canvas)
    pub position: Option<(u32, u32)>,
}

/// Some additional rendering data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderData {
    /// Whether it's the initial render
    pub initial_render: bool,
    /// The width of the canvas
    pub width: u32,
    /// The height of the canvas
    pub height: u32,
    /// If the canvas was resized before this frame
    pub resized: bool,
    /// Whether any render input changed
    pub input_changed: bool,
    /// The amount of milliseconds that passed since the beginning of rendering
    pub time: u32,
    /// The amount of milliseconds that passed since the last frame
    pub delta_time: u32,
    /// Info about the mouse
    pub mouse_data: MouseData,
}

/// A trait for rendering on a [Canvas]
pub trait CanvasRenderer: Clone + PartialEq + 'static {
    /// Internal state that can be modified each render
    type RenderState: 'static;
    /// External input that can not be modified from within the renderer
    type RenderInput: Clone + PartialEq + 'static;

    /// Called every frame to render to the [Canvas]
    fn render(
        &self,
        state: &mut Self::RenderState,
        input: &Self::RenderInput,
        gl: &GL,
        render_data: RenderData,
    );

    /// Create the initial render state
    fn initial_render_state(
        &self,
        input: &Self::RenderInput,
        gl: &GL,
        render_data: RenderData,
    ) -> Self::RenderState;
}

/// Properties for use in [Html]
#[derive(Debug, Properties, PartialEq)]
pub struct CanvasProperties<R>
where
    R: CanvasRenderer + PartialEq,
    R::RenderInput: PartialEq,
{
    /// The node ref used to hold on to the canvas
    #[prop_or_default]
    pub canvas_node_ref: NodeRef,
    /// The renderer used on this [Canvas]
    pub renderer: R,
    /// Input to the renderer
    pub render_input: R::RenderInput,
    /// The width of the [Canvas], valid css
    #[prop_or(AttrValue::from("100%"))]
    pub width: AttrValue,
    /// The height of the [Canvas], valid css
    #[prop_or(AttrValue::from("100%"))]
    pub height: AttrValue,
    /// The render loop state
    #[prop_or(RenderLoopState::Rendering)]
    pub render_loop_state: RenderLoopState,
}

/// A Canvas used for rendering with WebGL
pub struct Canvas<R>
where
    R: CanvasRenderer,
{
    /// The canvas node
    canvas_node_ref: NodeRef,
    /// Internal state for the renderer
    canvas_render_state: Arc<Mutex<CanvasRenderState<R>>>,
    /// Whether to initiate the gl render loop on the next render
    initiate_render_loop: bool,
}

/// Internal rendering state
struct CanvasRenderState<R>
where
    R: CanvasRenderer,
{
    /// The renderer
    renderer: R,
    /// The render state
    render_state: Option<R::RenderState>,
    /// The render input
    render_input: R::RenderInput,
    /// Whether the render input changed last frame
    render_input_changed: bool,
    /// The render loop state
    render_loop_state: RenderLoopState,
    /// Mouse data
    mouse_data: MouseData,
}

impl<R> CanvasRenderState<R>
where
    R: CanvasRenderer,
{
    /// Create a new [CanvasRenderState]
    fn new(
        renderer: R,
        canvas_render_input: R::RenderInput,
        render_loop_state: RenderLoopState,
    ) -> Self {
        Self {
            renderer,
            render_state: None,
            render_input: canvas_render_input,
            render_input_changed: false,
            render_loop_state,
            mouse_data: MouseData::default(),
        }
    }
}

impl<R> Component for Canvas<R>
where
    R: CanvasRenderer + PartialEq + Clone + 'static,
    R::RenderInput: PartialEq + Clone + 'static,
    R::RenderState: 'static,
{
    type Message = ();
    type Properties = CanvasProperties<R>;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            canvas_node_ref: ctx.props().canvas_node_ref.clone(),
            canvas_render_state: Arc::new(Mutex::new(CanvasRenderState::new(
                ctx.props().renderer.clone(),
                ctx.props().render_input.clone(),
                ctx.props().render_loop_state,
            ))),
            initiate_render_loop: matches!(
                ctx.props().render_loop_state,
                RenderLoopState::Rendering | RenderLoopState::Paused
            ),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let css = css!(
            r#"
                background-color: #000000;
                width: ${w};
                height: ${h};
                user-select: none;
            "#,
            w = ctx.props().width,
            h = ctx.props().height,
        );

        let onmousedown = Callback::from({
            let state: Arc<_> = self.canvas_render_state.clone();
            move |event: MouseEvent| {
                let buttons = event.buttons();
                if buttons & 0b1 == 0b1 {
                    state.lock().unwrap().mouse_data.primary_button = true;
                }
                if buttons & 0b10 == 0b10 {
                    state.lock().unwrap().mouse_data.secondary_button = true;
                }
            }
        });
        let onmouseup = Callback::from({
            let state: Arc<_> = self.canvas_render_state.clone();
            move |event: MouseEvent| {
                let buttons = event.buttons();
                if buttons & 0b1 == 0 {
                    state.lock().unwrap().mouse_data.primary_button = false;
                }
                if buttons & 0b10 == 0 {
                    state.lock().unwrap().mouse_data.secondary_button = false;
                }
            }
        });
        let onmousemove = Callback::from({
            let state: Arc<_> = self.canvas_render_state.clone();
            move |event: MouseEvent| {
                state.lock().unwrap().mouse_data.position =
                    Some((event.offset_x() as u32, event.offset_y() as u32));
            }
        });
        let onmouseleave = Callback::from({
            let state: Arc<_> = self.canvas_render_state.clone();
            move |_: MouseEvent| {
                state.lock().unwrap().mouse_data.position = None;
            }
        });
        let oncontextmenu = Callback::from(|e: MouseEvent| e.prevent_default());

        html! {
            <canvas
                class={css}
                ref={self.canvas_node_ref.clone()}
                {onmousedown}
                {onmouseup}
                {onmousemove}
                {onmouseleave}
                {oncontextmenu}
            />
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if !self.initiate_render_loop {
            return;
        }

        let canvas = self.canvas_node_ref.cast::<HtmlCanvasElement>().unwrap();
        let gl: GL = canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        Self::init_render_loop(gl, self.canvas_render_state.clone());

        self.initiate_render_loop = false;
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let new_props = ctx.props();
        let mut changed = false;

        if old_props.renderer != new_props.renderer {
            let mut render_state = self.canvas_render_state.lock().unwrap();

            render_state.render_state = None;
            render_state.renderer = new_props.renderer.clone();

            drop(render_state);
        }
        if old_props.render_input != new_props.render_input {
            let mut render_state = self.canvas_render_state.lock().unwrap();

            render_state.render_input = new_props.render_input.clone();
            render_state.render_input_changed = true;

            drop(render_state);
        }
        if old_props.render_loop_state != new_props.render_loop_state {
            self.canvas_render_state.lock().unwrap().render_loop_state =
                new_props.render_loop_state;

            if let RenderLoopState::Finished = old_props.render_loop_state {
                self.initiate_render_loop = true;
                changed = true;
            }
        }
        if old_props.width != new_props.width || old_props.height != new_props.height {
            changed = true;
        }

        changed
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.canvas_render_state.lock().unwrap().render_loop_state = RenderLoopState::Finished;
    }
}

impl<R> Canvas<R>
where
    R: CanvasRenderer + 'static,
    R::RenderState: 'static,
{
    /// Resize the canvas size to fir it's actual size (not 100% accurate but good enough?)
    fn resize_to_display_size(gl: &GL) -> (u32, u32, bool) {
        let canvas: HtmlCanvasElement = gl
            .canvas()
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        let (client_width, client_height): (u32, u32) = (
            canvas.client_width().try_into().unwrap(),
            canvas.client_height().try_into().unwrap(),
        );

        let resized = if client_width != canvas.width() || client_height != canvas.height() {
            canvas.set_width(client_width);
            canvas.set_height(client_height);

            true
        } else {
            false
        };

        (canvas.width(), canvas.height(), resized)
    }

    /// Initiate the rendering loop to render each frame
    fn init_render_loop(gl: GL, rendering_state: Arc<Mutex<CanvasRenderState<R>>>) {
        type SelfOwnedSharedFunction<T> = Rc<RefCell<Option<Closure<dyn FnMut(T)>>>>;
        let cb: SelfOwnedSharedFunction<u32> = Rc::new(RefCell::new(None));

        *cb.borrow_mut() = Some(Closure::wrap(Box::new({
            let cb = cb.clone();
            let mut last_time = 0;
            move |time: u32| {
                match &mut *rendering_state.lock().unwrap() {
                    CanvasRenderState {
                        renderer,
                        render_state,
                        render_input: canvas_render_input,
                        render_input_changed,
                        render_loop_state: RenderLoopState::Rendering,
                        mouse_data,
                    } => {
                        let (width, height, resized) = Self::resize_to_display_size(&gl);
                        let render_data = RenderData {
                            initial_render: render_state.is_none(),
                            width,
                            height,
                            resized,
                            input_changed: *render_input_changed,
                            time,
                            delta_time: time - last_time,
                            mouse_data: *mouse_data,
                        };

                        let render_state = render_state.get_or_insert_with(|| {
                            renderer.initial_render_state(canvas_render_input, &gl, render_data)
                        });

                        renderer.render(render_state, canvas_render_input, &gl, render_data);

                        *render_input_changed = false;
                        last_time = time;
                    }
                    CanvasRenderState {
                        render_loop_state: RenderLoopState::Finished,
                        ..
                    } => {
                        *cb.borrow_mut() = None;
                        return;
                    }
                    CanvasRenderState {
                        render_loop_state: RenderLoopState::Paused,
                        ..
                    } => {}
                }

                Self::render_loop(cb.borrow().as_ref().unwrap());
            }
        }) as Box<dyn FnMut(u32)>));

        Self::render_loop(cb.borrow().as_ref().unwrap());
    }

    /// Helper method for the rendering loop
    fn render_loop(render_function: &Closure<dyn FnMut(u32)>) {
        window()
            .request_animation_frame(render_function.as_ref().unchecked_ref())
            .unwrap();
    }
}
