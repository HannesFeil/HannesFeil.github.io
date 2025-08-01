//! Utilities to handle website navigation

use std::{fmt::Display, rc::Rc};

use convert_case::Casing;
use strum::IntoEnumIterator as _;
use stylist::{css, yew::use_style};
use yew::prelude::*;
use yew_router::{BrowserRouter, Routable, Switch, prelude::Link};

use crate::{HomePage, about::AboutPage, projects::Project, theme::ThemeSelector, use_theme};

#[cfg(debug_assertions)]
use crate::TestPage;

type Context = UseReducerHandle<NavigationContext>;

/// The available routes for the website
#[derive(Debug, Clone, Copy, PartialEq, Eq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/project/*project")]
    Project { project: Project },
    #[at("/about")]
    About,
    #[cfg(debug_assertions)]
    #[at("/test")]
    Test,
    #[not_found]
    #[at("/404")]
    NotFound,
}

impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Route::Home => "Home",
                Route::About => "About",
                #[cfg(debug_assertions)]
                Route::Test => "Test",
                Route::NotFound => "Not Found",
                Route::Project { project } => project.meta().title,
            }
        )
    }
}

/// A section on a single page
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SectionData {
    /// The title of the section
    title: AttrValue,
    /// The id of the section
    id: AttrValue,
    /// The level of the section
    level: usize,
}

impl SectionData {
    /// Create a new section. The id will be generated by converting the title to kebab-case.
    pub fn new(title: AttrValue, level: usize) -> Self {
        let id = title.as_str().to_case(convert_case::Case::Kebab).into();

        Self::new_with_id(title, id, level)
    }

    /// Creates a new section.
    pub fn new_with_id(title: AttrValue, id: AttrValue, level: usize) -> Self {
        Self { title, id, level }
    }

    /// Returns the section title
    pub fn title(&self) -> &AttrValue {
        &self.title
    }

    /// Returns the section id
    pub fn id(&self) -> &AttrValue {
        &self.id
    }
}

/// Actions for modifying the [`NavigationContext`]
pub enum NavigationContextAction {
    /// Add a [`Section`] to the context
    AddSection {
        /// The section data
        data: SectionData,
        /// Optional parent section id
        parent_id: Option<AttrValue>,
    },
    /// Set the current [`Route`] for the context
    SetRoute(Route),
}

/// The navigation context, keeping track of currently relevant [`Sections`](Section)
#[derive(Debug, Default, Clone, PartialEq)]
pub struct NavigationContext {
    /// The current [`Route`]
    route: Route,
    /// The flat [`Section`] tree (Optional parent id, data)
    sections: Vec<(Option<AttrValue>, SectionData)>,
}

impl Reducible for NavigationContext {
    type Action = NavigationContextAction;

    fn reduce(mut self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let this = Rc::make_mut(&mut self);

        match action {
            NavigationContextAction::AddSection { data, parent_id } => {
                let val = (parent_id, data);
                if !(this.sections.contains(&val)) {
                    this.sections.push(val);
                }
            }
            NavigationContextAction::SetRoute(route) => {
                if this.route != route {
                    this.route = route;
                    this.sections.clear();
                }
            }
        }

        self
    }
}

/// The switch used to render different pages depending on the route
#[function_component(PageSwitch)]
pub fn page_switch() -> Html {
    let context = use_reducer_eq(NavigationContext::default);
    html! {
        <BrowserRouter>
            <ContextProvider<Context> {context}>
                <Switch<Route> render={switch} />
            </ContextProvider<Context>>
        </BrowserRouter>
    }
}

/// A convenient hook for accessing the [`NavigationContext`].
///
/// # Panics
/// If the [`NavigationContext`] has not been provided.
#[hook]
pub fn use_navigation_context() -> Context {
    use_context::<Context>().unwrap()
}

/// Generate page html depending on the route
fn switch(route: Route) -> Html {
    const NAV_BAR_HEIGHT: &str = "40px";
    const NAV_BAR_WIDTH: &str = "300px";

    let content = match route {
        Route::Home => html! { <HomePage/> },
        Route::About => html! { <AboutPage/> },
        Route::NotFound => html! { <p>{"NotFount" }</p> },
        #[cfg(debug_assertions)]
        Route::Test => html! { <TestPage/> },
        Route::Project { project } => project.html(),
    };

    html! {
        <>
            <NavBar route={route} height={NAV_BAR_HEIGHT} sidebar_width={NAV_BAR_WIDTH}/>
            <div class={css!("translate: 0px ${height};", height = NAV_BAR_HEIGHT)}>
                <SwitchInner {route}>
                    {content}
                </SwitchInner>
            </div>
        </>
    }
}

/// Properties for the [`SwitchInner`] component
#[derive(Debug, PartialEq, Properties)]
struct SwitchInnerProperties {
    /// The current route
    route: Route,
    /// children
    children: Children,
}

/// A wrapper component to update the [`NavigationContext`] route
#[function_component(SwitchInner)]
fn switch_inner(props: &SwitchInnerProperties) -> Html {
    let context = use_navigation_context();
    context.dispatch(NavigationContextAction::SetRoute(props.route));

    html! {
        {props.children.clone()}
    }
}

/// Properties for the [`NavBar`] component
#[derive(Debug, Properties, PartialEq, Eq)]
struct NavigationBarProperties {
    /// Height of the bar
    height: AttrValue,
    /// Width of the expandable sidebar
    sidebar_width: AttrValue,
    /// The current route
    route: Route,
}

/// Creates a navigation bar component used to link petween pages.
///
/// Additionally an expandable sidebar is also created for listing relevant sections.
#[function_component(NavBar)]
fn navigation_bar(props: &NavigationBarProperties) -> Html {
    let theme = use_theme();
    let nav_bar_style = use_style!(
        r#"
            position: fixed;
            top: 0px;
            left: 0px;
            z-index: 1;
            width: 100%;
            height: ${height};
            display: flex;
            justify-content: space-between;
            background-color: ${bg};

            ul {
                margin: 0px;
                padding: 0px;
                list-style-type: none;
                overflow: hidden;
                height: 100%;
            }

            li {
                float: left;
            }

            .sidebar-button {
                display: block;
                height: ${height};
                width: ${height};
                border: none;
                color: ${fg};
                background-color: ${bg};
                font-size: 20px;
                line-height: ${height};
                padding-top: 3px;
            }

            .sidebar-button:hover {
                background-color: ${bg_hover};
            }
        "#,
        fg = theme.base00,
        bg = theme.base0D,
        bg_hover = theme.base0C,
        height = props.height,
    );
    let sidebar_visible = use_state(|| false);
    let toggle_sidebar = Callback::from({
        let sidebar_visible = sidebar_visible.clone();
        move |_| {
            sidebar_visible.set(!*sidebar_visible);
        }
    });
    let buttons: Html = [
        Route::Home,
        #[cfg(debug_assertions)]
        Route::Test,
        Route::About,
    ]
    .into_iter()
    .chain(Project::iter().map(|project| project.route()))
    .map(|route| {
        html! {
            <NavigationButton
                 height={props.height.clone()}
                 route={route}
                 text={route.to_string()}
                 active={route == props.route}
            />
        }
    })
    .collect();
    html! {
        <>
            <div class={nav_bar_style}>
                <nav>
                    <ul>
                        <li>
                            <button class="sidebar-button" onclick={toggle_sidebar}>
                                if *sidebar_visible {
                                    <i class="iconoir-xmark"/>
                                } else {
                                    <i class="iconoir-menu"/>
                                }
                            </button>
                        </li>
                        {buttons}
                    </ul>
                </nav>
                <div class={css!("margin: 0px 10px;")}>
                    <ThemeSelector/>
                </div>
            </div>
            <NavigationSidebar
                route={props.route}
                visible={*sidebar_visible}
                width={props.sidebar_width.clone()}
                navigation_bar_height={props.height.clone()}
            />
        </>
    }
}

/// Properties for the [`NavigationButton`] component
#[derive(Properties, PartialEq)]
struct NavigationButtonProperties {
    /// The route the button links to
    pub route: Route,
    /// The display value of the button
    #[prop_or(AttrValue::Static("Button"))]
    pub text: AttrValue,
    /// Whether the button is currently highlighted
    #[prop_or_default]
    pub active: bool,
    /// The height of the button
    pub height: AttrValue,
}

/// Creates a navigation button component for switching pages
#[function_component(NavigationButton)]
fn navigation_button(props: &NavigationButtonProperties) -> Html {
    let route = props.route;
    let theme = use_theme();
    let style = use_style!(
        r#"
            float: left;
            padding: 0px;

            a {
                display: block;
                margin: 0px;
                padding: 0px 10px;
                background-color: ${bg};
                color: ${fg};
                text-decoration: none;
                text-align: center;
                font-weight: bold;
                font-size: 20px;
                line-height: ${height};
            }

            a:hover {
                background-color: ${bg_hover};
            }
        "#,
        height = props.height,
        fg = theme.base00,
        bg = if props.active {
            &theme.base0B
        } else {
            &theme.base0D
        },
        bg_hover = theme.base0C,
    );
    html! {
        <li class={style}>
            <Link<Route> to={route}>{props.text.clone()}</Link<Route>>
        </li>
    }
}

/// Properties for the [`NavigationSidebar`] component
#[derive(Debug, PartialEq, Properties)]
struct NavigationSidebarProperties {
    /// The current route
    route: Route,
    /// Whether the sidebar is visible
    visible: bool,
    /// The width of the expanded sidebar
    width: AttrValue,
    /// The height of the [`NavigationBar`] used to calculate screen offsets
    navigation_bar_height: AttrValue,
}

/// Creates an expandable sidebar for listing available sections
#[function_component(NavigationSidebar)]
fn navigation_sidebar(props: &NavigationSidebarProperties) -> Html {
    let theme = use_theme();
    let style = use_style!(
        r#"
            .sidebar {
                position: fixed;
                z-index: 1;
                height: 100%;
                width: ${width};
                top: ${height};
                left: 0;
                background-color: ${bg};
                transition: left 0.5s;
                overflow: hidden;
            }

            .sidebar-hidden {
                left: -${width};
            }

            .sidebar h1 {
                text-align: center;
            }
        "#,
        width = props.width,
        bg = theme.base03,
        height = props.navigation_bar_height,
    );
    let classes = if props.visible {
        classes!("sidebar")
    } else {
        classes!("sidebar", "sidebar-hidden")
    };
    html! {
        <div class={style}>
            <div class={classes}>
                <h1>{props.route.to_string()}</h1>
                <SectionLinks/>
            </div>
        </div>
    }
}

// TODO: styling
/// Component for displaying section links recursively
#[function_component(SectionLinks)]
fn section_links() -> Html {
    let context = use_navigation_context();
    let theme = use_theme();
    let style = use_style!(
        r#"
            ol {
                counter-reset: item;
                padding-left: 10px;
            }

            li {
                display: block;
                margin: 10px 0px;
            }

            li:before {
                content: counters(item, ".") ". ";
                counter-increment: item;
            }

            a {
                text-decoration: none;
                color: ${fg};
            }

            a:hover {
                color: ${fg_hover};
            }
        "#,
        fg = theme.base06,
        fg_hover = theme.base07,
    );
    let sections = context.sections.as_slice();

    fn html_link_tree<'a>(
        section_id: Option<&'a AttrValue>,
        sections: &'a [(Option<AttrValue>, SectionData)],
    ) -> Option<Html> {
        let active_sections = sections
            .iter()
            .filter(move |(id, _)| id.as_ref() == section_id);

        let inner_link_trees: Vec<_> = active_sections
            .map(|(_, data)| {
                let sub_tree = html_link_tree(Some(data.id()), sections);
                html! {
                    <li>
                        <a href={format!("#{id}", id = data.id)}>
                            {data.title.clone()}
                        </a>
                        {sub_tree}
                    </li>
                }
            })
            .collect();

        if inner_link_trees.is_empty() {
            return None;
        }

        Some(html! {
            <ol>
                {inner_link_trees}
            </ol>
        })
    }

    html! {
        <div class={style}>
            {html_link_tree(None, sections)}
        </div>
    }
}

/// Properties for the [`Section`] component
#[derive(Debug, PartialEq, Properties)]
pub struct SectionProperties {
    /// The optional section id
    #[prop_or_default]
    pub id: AttrValue,
    /// Optionally hide the title
    #[prop_or_default]
    pub hide_title: bool,
    /// The section title
    pub title: AttrValue,
    /// The inner content
    pub children: Children,
}

/// Context for the current section
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct SectionContext {
    /// Id of the current section
    id: AttrValue,
    /// Level of the current section
    level: usize,
}

/// Creates a html section and updates the navbar accordingly
#[function_component(Section)]
pub fn section(props: &SectionProperties) -> Html {
    let nav_context = use_navigation_context();
    let section_context = use_context::<SectionContext>();
    let (parent_id, level) = match section_context {
        Some(context) => (Some(context.id), context.level + 1),
        None => (None, 0),
    };

    let section_data = if !props.id.is_empty() {
        crate::navigation::SectionData::new_with_id(props.title.clone(), props.id.clone(), level)
    } else {
        crate::navigation::SectionData::new(props.title.clone(), level)
    };

    let id = section_data.id().clone();
    use_effect({
        let data = section_data;
        move || {
            nav_context.dispatch(NavigationContextAction::AddSection { data, parent_id });
        }
    });

    let new_context = SectionContext {
        id: id.clone(),
        level,
    };

    let title = (!props.hide_title).then(|| match level {
        0 => html! { <h2>{props.title.clone()}</h2> },
        1 => html! { <h3>{props.title.clone()}</h3> },
        2 => html! { <h4>{props.title.clone()}</h4> },
        3 => html! { <h5>{props.title.clone()}</h5> },
        4.. => html! { <h6>{props.title.clone()}</h6> },
    });

    html! {
        <ContextProvider<SectionContext> context={new_context}>
            <section {id}>
                {title}
                {props.children.clone()}
            </section>
        </ContextProvider<SectionContext>>
    }
}
