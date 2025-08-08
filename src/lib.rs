#![warn(rustdoc::broken_intra_doc_links)]

use projects::ProjectPreview;
use strum::IntoEnumIterator as _;
use stylist::{
    css,
    yew::{Global, use_style},
};
use theme::{ThemeProvider, use_theme};
use yew::prelude::*;
use yew_agent::oneshot::OneshotProvider;

use crate::{
    navigation::{PageSwitch, Section},
    projects::{CodeExample, Project},
    theme::{LoadSyntaxTheme, use_highlight_set},
};

pub mod about;
pub mod navigation;
pub mod projects;
pub mod theme;
pub mod webgl;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <OneshotProvider<LoadSyntaxTheme> path="/worker.js">
            <ThemeProvider>
                <GlobalStyle/>
                <PageSwitch/>
            </ThemeProvider>
        </OneshotProvider<LoadSyntaxTheme>>
    }
}

#[function_component(GlobalStyle)]
fn global_style() -> Html {
    let theme = use_theme();
    html! {
        <Global css={
            css!(r#"
                    body {
                        background: ${bg};
                        color: ${fg};
                        margin: 0px;
                        font-family: "hermit";
                    }

                    :where(h1) {
                      margin-block: 0.67em;
                      font-size: 2em;
                    }
                "#,
                bg = theme.base01,
                fg = theme.base06,
            )
        }/>
    }
}

#[function_component(HomePage)]
fn home_page() -> Html {
    let style = use_style!(
        r#"
            h1, h2 {
                text-align: center;
            }

            p {
                text-align: center;
                margin: 2px 15%;
            }

            ul {
                list-style-type: none;
                padding: 10px;
            }

            li {
                margin: 20px 0;
            }

            section {
                margin-bottom: 50px;
            }
        "#
    );
    let projects = Project::iter().map(|project| html! { <li><ProjectPreview {project}/></li> });
    html! {
        <div class={style}>
            <Section title="Welcome" hide_title=true>
                <h1>{"Cute Codlings"}</h1>
                <p>{"So you found your way to our little website?"}</p>
                <p>{"Lucky you :)"}</p>
                <p>{"Feel free to wander around and enjoy our little codlings." }</p>
            </Section>
            <Section title="Projects">
                <ul>
                    {for projects}
                </ul>
            </Section>
        </div>
    }
}

#[cfg(debug_assertions)]
#[function_component(TestPage)]
fn test_page() -> Html {
    let theme = use_theme();
    let colors: Html = [
        &theme.base00,
        &theme.base01,
        &theme.base02,
        &theme.base03,
        &theme.base04,
        &theme.base05,
        &theme.base06,
        &theme.base07,
        &theme.base08,
        &theme.base09,
        &theme.base0A,
        &theme.base0B,
        &theme.base0C,
        &theme.base0D,
        &theme.base0E,
        &theme.base0F,
    ]
    .into_iter()
    .enumerate()
    .map(|(i, color)| {
        html! {
            <p style={format!("color:{color};")}>{format!("⏹ Base 0{i:X?}")}</p>
        }
    })
    .collect();
    html! {
        <>
            <Section title="Colors">
                {colors}
            </Section>
            <Section title="Syntax Themes">
                <Suspense fallback="Loading syntax themes">
                    <SyntaxThemesTest/>
                </Suspense>
            </Section>
        </>
    }
}

#[function_component(SyntaxThemesTest)]
fn test_syntax_themes() -> HtmlResult {
    let highlight_set = use_highlight_set()?;
    let names = highlight_set.themes().themes.iter().map(|(key, theme)| {
        html! {
            <li>
                {theme.name.as_ref()}{" : "}{key.clone()}
                <CodeExample lang="Rust" theme={Some(key.clone())}>
                    {indoc::indoc! {r#"
                        pub fn main() {
                            println!("Hello World");
                        }
                    "#}}
                </CodeExample>
            </li>
        }
    });
    Ok(html! {
        <ul>
            {for names}
        </ul>
    })
}
