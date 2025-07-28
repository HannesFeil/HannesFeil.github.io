//! Individual project pages

use stylist::{css, yew::use_style};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, Style},
    util::LinesWithEndings,
};
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    about::Author,
    navigation::Route,
    projects::{boids::BoidsPage, fractal_clock::FractalClockPage},
    theme::use_theme,
    theme::{HighlightSet, use_highlight_set},
};

pub mod boids;
pub mod fractal_clock;
mod interactive;

/// An enum of all projects
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Project {
    /// Fractal clock
    FractalClock,
    /// Boids
    Boids,
}

/// Project metadata
#[derive(Clone, Copy)]
pub struct ProjectMeta {
    /// The title
    pub title: &'static str,
    /// A short description
    pub description: &'static str,
    /// The authors
    pub authors: &'static [Author],
}

impl Project {
    /// Returns the projects metadata
    pub const fn meta(self) -> ProjectMeta {
        match self {
            Project::FractalClock => ProjectMeta {
                title: "Fractal Clock",
                description: indoc::indoc! {"
                    When drawing an analogue clock recursively at each pointer tip, beautiful
                    patterns emerge. We will explore how to optimize and render this efficiently
                    using webgl rendering.
                "},
                authors: &[Author::Ciklon],
            },
            Project::Boids => ProjectMeta {
                title: "Boids",
                description: indoc::indoc! {"
                    This interactive tutorial guides you through implementing the Boids algorithm,
                    originally developed by Craig Reynolds in 1986, using a compute shader.
                "},
                authors: &[Author::DawnFirefly],
            },
        }
    }

    /// Returns the route that leads to the project page
    pub fn route(self) -> Route {
        Route::Project { project: self }
    }

    /// Returns the path to the project preview image
    pub fn preview_image_path(self) -> String {
        format!("assets/images/preview/{self}.png")
    }

    /// Returns the project page html
    pub fn html(self) -> Html {
        match self {
            Project::FractalClock => html! { <FractalClockPage/> },
            Project::Boids => html! { <BoidsPage/> },
        }
    }
}

/// Properties for the [`ProjectPreview`] component
#[derive(Debug, PartialEq, Properties)]
pub struct ProjectPreviewProperties {
    pub project: Project,
}

/// A project preview component showing an image next to a description
#[function_component(ProjectPreview)]
pub fn project_preview(ProjectPreviewProperties { project }: &ProjectPreviewProperties) -> Html {
    let theme = use_theme();
    let style = use_style!(
        r#"
            display: flex;
            justify-content: center;
            background-color: ${container_bg};
            padding: 10px;
            height: 350px;
            width: 900px;
            margin: 0 auto;

            a {
                margin: 10px;
                color: ${heading_fg};
                overflow: hidden;
                display: flex;
                justify-content: center;
                text-align: center;
            }

            > a {
                width: 50%;
            }

            a img {
                height: 100%;
            }

            > div {
                width: 50%;
                display: flex;
                flex-direction: column;
                justify-content: center;
            }

            h3 {
                text-align: center;
                color: ${heading_fg};
            }

            .authors {
                margin: 40px 0px;
                text-align: center;
                display: flex;
                justify-content: center;
            }
        "#,
        container_bg = theme.base02,
        heading_fg = theme.base06,
    );
    let authors = project.meta().authors.iter().map(|author| {
        html! {
            <div class={css!("margin: 0px 10px;")}>
                {author.badge()}
            </div>
        }
    });
    html! {
        <div class={style}>
            <Link<Route> to={project.route()}>
                <img src={project.preview_image_path()}/>
            </Link<Route>>
            <div>
                <Link<Route> to={project.route()}>
                    <h3>{project.meta().title}</h3>
                </Link<Route>>
                <p>{project.meta().description}</p>
                <div class="authors">
                    {for authors}
                </div>
            </div>
        </div>
    }
}

/// Properties for the [`ProjectSite`] component
#[derive(Debug, Properties, PartialEq)]
pub struct ProjectSiteProperties {
    /// Site title
    title: AttrValue,
    /// Inner content
    children: Children,
}

/// Wraps project content in a page (mainly for styling)
#[function_component(ProjectSite)]
pub fn project_site(ProjectSiteProperties { title, children }: &ProjectSiteProperties) -> Html {
    let theme = use_theme();
    let style = use_style!(
        r#"
            width: 900px;
            height: 100%;
            margin: 0 auto;
            padding: 20px;
            background-color: ${bg};
            display: flex;
            flex-direction: column;

            h1 {
                text-align: center;
            }

            a {
                color: ${link_fg};
            }
        "#,
        bg = theme.base02,
        link_fg = theme.base0C,
    );
    html! {
        <div class={style}>
            <h1>{title}</h1>
            {children}
        </div>
    }
}

/// Properties for the [`CodeExample`] component
#[derive(Debug, PartialEq, Properties)]
pub struct CodeExampleProperties {
    /// the language of the example
    pub lang: AttrValue,
    /// The inner code
    pub children: &'static str,
    /// The syntax theme
    #[prop_or_default]
    pub theme: Option<AttrValue>,
}

/// A Code example displays syntax highlighted code
#[function_component(CodeExample)]
pub fn code_example(props: &CodeExampleProperties) -> Html {
    html! {
        <Suspense fallback={"Loading code..."}>
            <CodeExampleInner lang={props.lang.clone()} theme={props.theme.clone()}>
                {props.children}
            </CodeExampleInner>
        </Suspense>
    }
}

/// A helper component for the code example to allow suspense rendering
#[function_component(CodeExampleInner)]
fn code_example_inner(props: &CodeExampleProperties) -> HtmlResult {
    let theme = use_theme();
    let highlight_set = use_highlight_set()?;
    let style = use_style!(
        r#"
            background-color: ${bg};
            padding: 10px 20px;
            font-family: monospace;
            font-size: 15px;
        "#,
        bg = theme.base00,
    );
    let highlighted = highlight_code(
        &props.lang,
        props.children,
        &highlight_set,
        props
            .theme
            .as_ref()
            .map(AttrValue::as_str)
            .unwrap_or(theme.syntax_theme),
    );
    let content = match highlighted {
        Ok(highlighted) => highlight_to_html(&highlighted),
        Err(error) => html! { {error} },
    };
    Ok(html! {
        <div class={style}>
            <pre>
                {content}
            </pre>
        </div>
    })
}

/// A helper method for highlighting code with a [`SyntaxTheme`]
fn highlight_code<'code>(
    lang: &str,
    code: &'code str,
    highlight_set: &HighlightSet,
    theme_name: &str,
) -> Result<Vec<(Style, &'code str)>, syntect::Error> {
    let theme = &highlight_set.themes().themes[theme_name];
    let syntax = highlight_set.syntaxes().find_syntax_by_name(lang).unwrap();
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut result = Vec::default();
    for line in LinesWithEndings::from(code) {
        let mut highlighted = highlighter.highlight_line(line, highlight_set.syntaxes())?;
        result.append(&mut highlighted);
    }

    Ok(result)
}

/// Converts a sequence of highlighted strings to html
fn highlight_to_html(highlight: &[(Style, &str)]) -> Html {
    fn to_css_style(style: &Style) -> String {
        format!(
            "{underline}{bold}{italic}color: {color}",
            underline = if style.font_style.contains(FontStyle::UNDERLINE) {
                "text-decoration: underline;"
            } else {
                ""
            },
            bold = if style.font_style.contains(FontStyle::BOLD) {
                "font-weight: bold;"
            } else {
                ""
            },
            italic = if style.font_style.contains(FontStyle::ITALIC) {
                "font-style: italic;"
            } else {
                ""
            },
            color = {
                let col = style.foreground;
                format!(
                    "#{r:02x}{g:02x}{b:02x}{alpha}",
                    r = col.r,
                    g = col.g,
                    b = col.b,
                    alpha = if col.a != 0xFF {
                        format!("{a:02x}", a = col.a)
                    } else {
                        "".to_owned()
                    }
                )
            }
        )
    }
    let mut output: Vec<Html> = vec![];
    let mut prev_style: Option<&Style> = None;
    let mut prev_text: Vec<&str> = vec![];
    for &(ref style, text) in highlight.iter() {
        let unify_style = if let Some(ps) = prev_style {
            style == ps || (style.background == ps.background && text.trim().is_empty())
        } else {
            false
        };
        if unify_style {
            prev_text.push(text);
        } else {
            if let Some(style) = prev_style {
                let css = to_css_style(style);
                output.push(html! {
                    <span style={css}>
                        {for prev_text.iter()}
                    </span>
                });
                prev_text.clear();
            }
            prev_style = Some(style);
            prev_text.push(text);
        }
    }
    if let Some(style) = prev_style {
        let css = to_css_style(style);
        output.push(html! {
            <span style={css}>
                {for prev_text.iter()}
            </span>
        });
        prev_text.clear();
    }
    output.into_iter().collect()
}
