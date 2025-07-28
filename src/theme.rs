//! Website theming

use gloo_storage::Storage;
use std::{
    collections::HashSet,
    ops::{Deref, Index},
    rc::Rc,
};
use strum::IntoEnumIterator;
use stylist::{css, yew::use_style};
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};
use web_sys::HtmlSelectElement;
use yew::{
    Callback, Children, ContextProvider, Html, InputEvent, Properties, TargetCast, UseStateHandle,
    function_component, hook, html, platform::spawn_local, use_state,
};
use yew_agent::prelude::*;

const THEME_STORAGE_KEY: &str = "Theme";

/// An enum which can be used to index [`Theme`] colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeColor {
    Base00,
    Base01,
    Base02,
    Base03,
    Base04,
    Base05,
    Base06,
    Base07,
    Base08,
    Base09,
    Base0A,
    Base0B,
    Base0C,
    Base0D,
    Base0E,
    Base0F,
}

/// A general base 16 theme in combination with a syntax theme
#[allow(non_snake_case)]
#[derive(Debug)]
pub struct Theme {
    name: &'static str,
    author: &'static str,
    link: &'static str,
    /// base00 color css code
    pub base00: &'static str,
    /// base01 color css code
    pub base01: &'static str,
    /// base02 color css code
    pub base02: &'static str,
    /// base03 color css code
    pub base03: &'static str,
    /// base04 color css code
    pub base04: &'static str,
    /// base05 color css code
    pub base05: &'static str,
    /// base06 color css code
    pub base06: &'static str,
    /// base07 color css code
    pub base07: &'static str,
    /// base08 color css code
    pub base08: &'static str,
    /// base09 color css code
    pub base09: &'static str,
    /// base0A color css code
    pub base0A: &'static str,
    /// base0B color css code
    pub base0B: &'static str,
    /// base0C color css code
    pub base0C: &'static str,
    /// base0D color css code
    pub base0D: &'static str,
    /// base0E color css code
    pub base0E: &'static str,
    /// base0F color css code
    pub base0F: &'static str,
    /// The syntax theme name
    pub syntax_theme: &'static str,
}

impl Index<ThemeColor> for Theme {
    type Output = &'static str;

    fn index(&self, index: ThemeColor) -> &Self::Output {
        match index {
            ThemeColor::Base00 => &self.base00,
            ThemeColor::Base01 => &self.base01,
            ThemeColor::Base02 => &self.base02,
            ThemeColor::Base03 => &self.base03,
            ThemeColor::Base04 => &self.base04,
            ThemeColor::Base05 => &self.base05,
            ThemeColor::Base06 => &self.base06,
            ThemeColor::Base07 => &self.base07,
            ThemeColor::Base08 => &self.base08,
            ThemeColor::Base09 => &self.base09,
            ThemeColor::Base0A => &self.base0A,
            ThemeColor::Base0B => &self.base0B,
            ThemeColor::Base0C => &self.base0C,
            ThemeColor::Base0D => &self.base0D,
            ThemeColor::Base0E => &self.base0E,
            ThemeColor::Base0F => &self.base0F,
        }
    }
}

/// The collection of available themes
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumIter,
    strum::Display,
)]
pub enum ThemeKind {
    #[default]
    Dark,
    Light,
}

impl ThemeKind {
    /// Returns the current [`Theme`] for this kind
    pub fn current(self) -> &'static Theme {
        #[allow(non_snake_case)]
        const DARK: Theme = Theme {
            name: "Twilight",
            author: "David Hart",
            link: "https://github.com/hartbit/base16-twilight-scheme",
            base00: "#1e1e1e",
            base01: "#323537",
            base02: "#464b50",
            base03: "#5f5a60",
            base04: "#838184",
            base05: "#a7a7a7",
            base06: "#c3c3c3",
            base07: "#ffffff",
            base08: "#cf6a4c",
            base09: "#cda869",
            base0A: "#f9ee98",
            base0B: "#8f9d6a",
            base0C: "#afc4db",
            base0D: "#7587a6",
            base0E: "#9b859d",
            base0F: "#9b703f",
            syntax_theme: "base16-eighties.dark",
        };

        #[allow(non_snake_case)]
        const LIGHT: Theme = Theme {
            name: "Classic Light",
            author: "Jason Heeris",
            link: "https://github.com/detly/base16-classic-scheme",
            base00: "#f5f5f5",
            base01: "#e0e0e0",
            base02: "#d0d0d0",
            base03: "#b0b0b0",
            base04: "#505050",
            base05: "#303030",
            base06: "#202020",
            base07: "#151515",
            base08: "#ac4142",
            base09: "#d28445",
            base0A: "#f4bf75",
            base0B: "#90a959",
            base0C: "#75b5aa",
            base0D: "#6a9fb5",
            base0E: "#aa759f",
            base0F: "#8f5536",
            syntax_theme: "InspiredGitHub",
        };

        match self {
            ThemeKind::Dark => &DARK,
            ThemeKind::Light => &LIGHT,
        }
    }

    pub fn credits(self) -> Html {
        html! {
            <p>
                <h4 style="display: inline;">{self.to_string()}{": "}</h4>
                <a href={self.current().link}>{self.current().name}</a>
                {" by "}
                {self.current().author}
            </p>
        }
    }
}

/// A context used to relay theme information through the website
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeContext {
    /// Current theme handle
    inner: UseStateHandle<ThemeKind>,
    /// Global highlight set
    highlight: UseStateHandle<Option<Rc<HighlightSet>>>,
}

impl ThemeContext {
    /// Create a new theme context from a theme state handle
    pub fn new(
        inner: UseStateHandle<ThemeKind>,
        highlight: UseStateHandle<Option<Rc<HighlightSet>>>,
    ) -> Self {
        Self { inner, highlight }
    }

    /// Set the current theme
    pub fn set(&self, theme: ThemeKind) {
        gloo_storage::LocalStorage::set(THEME_STORAGE_KEY, theme).unwrap();
        self.inner.set(theme)
    }

    /// Retreive the current theme
    pub fn kind(&self) -> ThemeKind {
        *self.inner
    }
}

impl Deref for ThemeContext {
    type Target = Theme;

    fn deref(&self) -> &Self::Target {
        self.inner.current()
    }
}

/// Properties for the [`ThemeProvider`]
#[derive(Debug, PartialEq, Properties)]
pub(crate) struct ThemeProviderProps {
    pub children: Children,
}

/// A context provider for the [`ThemeContext`]
#[function_component(ThemeProvider)]
pub(crate) fn theme_provider(props: &ThemeProviderProps) -> Html {
    let theme_kind =
        use_state(|| gloo_storage::LocalStorage::get(THEME_STORAGE_KEY).unwrap_or_default());
    let highlight = use_state(|| None);
    let theme_ctx = ThemeContext::new(theme_kind, highlight);

    html! {
        <ContextProvider<ThemeContext> context={theme_ctx}>
            {props.children.clone()}
        </ContextProvider<ThemeContext>>
    }
}

/// A convenient hook for accessing the current theme context.
///
/// # Panics
/// If [`ThemeContext`] has not been provided.
#[hook]
pub(crate) fn use_theme() -> ThemeContext {
    use yew::use_context;

    use_context::<ThemeContext>().unwrap()
}

#[function_component(ThemeSelector)]
pub fn theme_selector() -> Html {
    let theme = use_theme();
    let style = use_style!(
        r#"
            height: 100%;
            font-size: 20px;
            font-weight: bold;
            display: flex;
            flex-direction: row;

            select {
                height: 100%;
                padding: 0px 5px;
                color: ${fg};
                background-color: ${bg};
                border: none;
                font-size: inherit;
                font-weight: inherit;
            }

            select:hover {
                background-color: ${bg_hover};
            }

            select option {
                background-color: ${bg};
                border: none;
            }

            select option:hover {
                background-color: ${bg_hover};
            }

            i {
                color: ${fg};
            }

            .icon-container {
                font-size: 30px;
                display: flex;
                align-items: center;
                margin: 0px 5px;
            }
        "#,
        fg = theme.base00,
        bg = theme.base0D,
        bg_hover = theme.base0C,
    );
    let themes = ThemeKind::iter().map(|kind| {
        html! {
            <option selected={kind == theme.kind()}>
                {kind.to_string()}
            </option>
        }
    });
    let on_input = Callback::from({
        let theme = theme.clone();
        move |event: InputEvent| {
            theme.set(
                ThemeKind::iter()
                    .nth(
                        usize::try_from(
                            event
                                .target_dyn_into::<HtmlSelectElement>()
                                .unwrap()
                                .selected_index(),
                        )
                        .unwrap(),
                    )
                    .unwrap(),
            );
        }
    });
    html! {
        <div class={style}>
            <div class="icon-container">
                <i class="iconoir-brightness"/>
            </div>
            <select oninput={on_input}>
                {for themes}
            </select>
        </div>
    }
}

/// The syntax theme
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HighlightSet {
    /// Available syntaxes for highlighting
    syntaxes: SyntaxSet,
    /// Available themes for highlighting
    themes: ThemeSet,
}

impl PartialEq for HighlightSet {
    fn eq(&self, other: &Self) -> bool {
        let syntax_names: HashSet<_> = self.syntaxes.syntaxes().iter().map(|s| &s.name).collect();
        let theme_names: HashSet<_> = self.themes.themes.iter().map(|t| t.0).collect();
        let other_syntax_names: HashSet<_> =
            other.syntaxes.syntaxes().iter().map(|s| &s.name).collect();
        let other_theme_names: HashSet<_> = other.themes.themes.iter().map(|t| t.0).collect();

        syntax_names == other_syntax_names && theme_names == other_theme_names
    }
}

impl HighlightSet {
    /// Returns the available syntaxes
    pub fn syntaxes(&self) -> &SyntaxSet {
        &self.syntaxes
    }

    /// Returns the available themes
    pub fn themes(&self) -> &ThemeSet {
        &self.themes
    }
}

/// A hook for accessing the current SyntaxTheme.
///
/// This hook leverages a webworker to load the available syntaxes asynchronously. This load time is
/// only expected when first calling this hook.
#[hook]
pub fn use_highlight_set() -> yew::suspense::SuspensionResult<Rc<HighlightSet>> {
    let theme = use_theme();
    let load_task = use_oneshot_runner::<LoadSyntaxTheme>();

    // If the theme is already loaded, return a reference
    if let Some(syntax_theme) = theme.highlight.as_ref() {
        return Ok(syntax_theme.clone());
    }

    let (s, handle) = yew::suspense::Suspension::new();

    // Otherwise load the theme
    spawn_local(async move {
        let loaded = load_task.run(()).await;
        theme.highlight.set(Some(Rc::new(loaded)));
        handle.resume();
    });

    Err(s)
}

/// The webworker function for loading the [`SyntaxTheme`]
#[oneshot(LoadSyntaxTheme)]
pub async fn load_syntax_theme(_: ()) -> HighlightSet {
    let syntaxes = SyntaxSet::load_defaults_newlines();
    let themes = ThemeSet::load_defaults();

    HighlightSet { syntaxes, themes }
}
