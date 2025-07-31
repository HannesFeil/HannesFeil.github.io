//! Meta information about the website

use strum::IntoEnumIterator;
use stylist::yew::use_style;
use yew::prelude::*;

use crate::{
    navigation::Section,
    theme::{ThemeColor, ThemeKind, use_theme},
};

const WEBSITE_SOURCE_LINK: &str = "https://github.com/HannesFeil/Website";

#[function_component(AboutPage)]
pub fn about_page() -> Html {
    let theme = use_theme();
    let style = use_style!(
        r#"
            h1, h2, h3 {
                text-align: center;
            }

            > section {
                padding: 5px 0px;
            }

            a {
                color: ${link_fg};
            }

            .centered-p {
                padding-bottom: 50px;
            }

            .centered-p p {
                text-align: center;
            }
        "#,
        link_fg = theme.base0C,
    );
    let author_sections = Author::iter().map(|a| {
        html! {
            <Section title={a.name()} hide_title=true>
                {a.description()}
            </Section>
        }
    });
    let theme_credits = ThemeKind::iter().map(|kind| kind.credits());
    html! {
        <div class={style}>
            <h1>{"About"}</h1>
            <Section title="The Website">
                <ImageSplitDiv image_path="assets/images/cod_256.png" image_link="/">
                    <p>
                        {"
                            During our time studying, we (Dawn Firefly and Ciklon) decided to build a website
                            together for one of our modules. It should be a little personal collection
                            of small projects we worked on for other people to discover and enjoy. You
                            are currently looking at the result of this idea.
                        "}
                    </p>
                    <p>
                        {"
                            The Source of this website is publicly availble on Github to anyone who
                            is interested: 
                        "}
                        <a href={WEBSITE_SOURCE_LINK}>{"Website Source"}</a>
                    </p>
                </ImageSplitDiv>
            </Section>
            <Section title="The Authors">
                {for author_sections}
            </Section>
            <div class="centered-p">
                <Section title="The Links">
                    <p>
                        <a href={WEBSITE_SOURCE_LINK}>{"The Website Source"}</a>
                    </p>
                    <h3>{"Themes"}</h3>
                    {for theme_credits}
                    <h3>{"Icons"}</h3>
                    <p>
                        <a href="https://iconoir.com/">{"Iconoir"}</a>
                    </p>
                    <h3>{"Fonts"}</h3>
                    <p>
                        <a href="https://pcaro.es/hermit/">{"Hermit"}</a>
                    </p>
                </Section>
            </div>
        </div>
    }
}

#[derive(Debug, PartialEq, Properties)]
struct ImageSplitProperties {
    image_path: AttrValue,
    image_link: Option<AttrValue>,
    children: Children,
}

#[function_component(ImageSplitDiv)]
fn image_split_div(props: &ImageSplitProperties) -> Html {
    let theme = use_theme();
    let style = use_style!(
        r#"
            display: flex;
            flex-direction: row;
            padding: 50px 15%;
            justify-content: space-between;
            align-items: center;
            background-color: ${image_p_bg};

            > a > img,
            > img {
                width: 256px;
                height: 256px;
            }

            > div {
                margin: 0px 100px;
            }
        "#,
        image_p_bg = theme.base00,
    );
    html! {
        <div class={style}>
            if let Some(link) = props.image_link.as_ref() {
                <a href={link.clone()}>
                    <img src={props.image_path.clone()}/>
                </a>
            } else {
                <img src={props.image_path.clone()}/>
            }
            <div>
                {props.children.clone()}
            </div>
        </div>
    }
}

/// Properties for the [`AuthorBadge`] element
#[derive(Debug, PartialEq, Eq, Properties)]
struct AuthorBadgeProperties {
    /// The name to be displayed
    name: &'static str,
    /// The color of the badge
    color: ThemeColor,
}

/// The author [`Author::badge()`] element
#[function_component(AuthorBadge)]
fn author_badge(AuthorBadgeProperties { name, color }: &AuthorBadgeProperties) -> Html {
    let theme = use_theme();
    let style = use_style!(
        r#"
            color: ${fg};
            background-color: ${bg};
            padding: 5px;
            border-radius: 5px;
        "#,
        fg = theme.base00,
        bg = theme[*color],
    );
    html! {
        <span class={style}>{ name }</span>
    }
}

/// Authors of this website
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum Author {
    Ciklon,
    DawnFirefly,
}

impl Author {
    /// Returns the name of the author
    pub const fn name(self) -> &'static str {
        match self {
            Author::DawnFirefly => "Dawn Firefly",
            Author::Ciklon => "Ciklon",
        }
    }

    pub const fn profile_link(self) -> &'static str {
        match self {
            Author::DawnFirefly => "https://github.com/Dawn-Firefly",
            Author::Ciklon => "https://github.com/HannesFeil",
        }
    }

    /// Creates a html element to identify with this author
    pub fn badge(self) -> Html {
        let name = self.name();
        let color = match self {
            Author::DawnFirefly => ThemeColor::Base0D,
            Author::Ciklon => ThemeColor::Base0B,
        };

        html! {
            <AuthorBadge {name} {color}/>
        }
    }

    pub fn description(self) -> Html {
        match self {
            // TODO: Dawn write this :3
            Author::DawnFirefly => html! {
                <ImageSplitDiv image_path="assets/images/dawnfirefly_profile_256.png" image_link={self.profile_link()}>
                    <h3>{self.name()}</h3>
                    <p>
                        {"
                            I'm a third year Bachelor's student in Computer science.
                        "}
                    </p>
                    <p>
                        {"
                            < This place intentionally left empty >
                        "}
                    </p>
                    <p>
                        {"
                            Lately I got interested in compute shaders because of their ability to
                            aid in the efficient creation of fascinating visuals. These highlighy
                            complex illustrations often emerge from relatively simple rules and
                            operations. 
                        "}
                    </p>
                </ImageSplitDiv>
            },
            Author::Ciklon => html! {
                <ImageSplitDiv image_path="assets/images/ciklon_profile_256.png" image_link={self.profile_link()}>
                    <h3>{self.name()}</h3>
                    <p>
                        {"
                            I'm (at the time of writing) 22 years old and studying for my Bachelor's
                            degree in Computer science.
                        "}
                    </p>
                    <p>
                        {"
                            The first 'game' I ever got was
                        "}
                        <a href="https://www.rpgmakerweb.com/products/rpg-maker-vx-ace">{"RPG Maker VX Ace"}</a>
                        {"
                            and although I have to admit that no game ever came to life, the concept
                            of creating worlds through programming (as 'high level' as it may have
                            been) inspired me. When I started having computer science lessons in
                            school, I think it's fair to say a passion was born. At that point I
                            had been playing around with Minecraft command blocks (allowing for some
                            convoluted form of logic and computation) and the freedom that common
                            programming languages allowed was astounding.
                        "}
                    </p>
                    <p>
                        {"
                            Lately I got caught in the beauty of the Rust programming language,
                            which is the reason this Website has been written in it (Sorry Dawn Firefly).
                        "}
                    </p>
                </ImageSplitDiv>
            },
        }
    }
}
