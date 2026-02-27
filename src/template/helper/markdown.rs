use std::collections::HashMap;

use comrak::Options;
use comrak::options::Plugins;
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use lol_html::html_content::{ContentType, Element};
use lol_html::{RewriteStrSettings, element, rewrite_str};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use tera::{Error, Filter, Value};

use crate::config::ThemeConfig;

pub struct MarkdownToHtml {
    syntect: SyntectAdapter,
}

impl MarkdownToHtml {
    pub fn from_config(config: &ThemeConfig) -> anyhow::Result<Self> {
        let mut builder = SyntectAdapterBuilder::new();

        if !config.enable_default_code_syntax {
            builder = builder.syntax_set(SyntaxSet::load_from_folder(
                &config.current().code_syntax_dir,
            )?);
        }
        if !config.enable_default_code_themes {
            builder = builder.theme_set(ThemeSet::load_from_folder(
                &config.current().code_themes_dir,
            )?);
        }

        let syntect = builder.theme(&config.current_code_theme).build();

        Ok(Self { syntect })
    }
}

impl Filter for MarkdownToHtml {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        let Some(markdown) = value.as_str() else {
            return Err(Error::msg(format!(
                "invalid value: {value}, expected string"
            )));
        };

        let options = markdown_options();
        let plugins = markdown_plugins(&self.syntect);

        let html = comrak::markdown_to_html_with_plugins(markdown, &options, &plugins);

        Ok(rewrite_html(&html).map_err(|e| format!("{e:?}"))?.into())
    }
}

fn markdown_options() -> Options<'static> {
    let mut options = Options::default();

    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.spoiler = true;
    options.extension.underline = true;
    options.extension.footnotes = true;
    options.extension.math_code = true;
    options.extension.shortcodes = true;
    options.extension.header_ids = Some(String::new());

    options.render.r#unsafe = true;

    options
}

fn markdown_plugins(syntect: &SyntectAdapter) -> Plugins<'_> {
    let mut plugins = Plugins::default();

    plugins.render.codefence_syntax_highlighter = Some(syntect);

    plugins
}

fn rewrite_html(html: &str) -> anyhow::Result<String> {
    let handler = |el: &mut Element| {
        el.before("<div class=\"table-box\">", ContentType::Html);
        el.after("</div>", ContentType::Html);
        Ok(())
    };

    let html = rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("table", handler)],
            ..RewriteStrSettings::new()
        },
    )?;

    Ok(html)
}
