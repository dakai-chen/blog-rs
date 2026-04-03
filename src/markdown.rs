use std::error::Error;
use std::fmt::Write;
use std::sync::LazyLock;

use comrak::html::{ChildRendering, Context};
use comrak::nodes::{NodeCodeBlock, NodeHeading, NodeTaskItem, NodeValue};
use comrak::options::Plugins;
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{Arena, Node, Options, node_matches};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::config::{ThemeCodeSource, ThemeConfig};

type BoxStdError = Box<dyn Error + Send + Sync + 'static>;

static MARKDOWN_TO_HTML_CONFIG: LazyLock<MarkdownToHtmlConfig> =
    LazyLock::new(|| MarkdownToHtmlConfig::from_config(&crate::config::get().theme).unwrap());

pub fn version() -> &'static str {
    &crate::config::get().theme.render_version
}

pub fn render(markdown: &str) -> anyhow::Result<String> {
    let options = markdown_options();
    let plugins = markdown_plugins(&MARKDOWN_TO_HTML_CONFIG.syntect);

    comrak::create_formatter!(CustomFormatter<()>, {
        NodeValue::Heading(nh) => |context, node, entering| {
            return custom_render_heading(context, node, entering, &nh);
        },
        NodeValue::CodeBlock(ref ncb) => |context, node, entering| {
            return custom_render_code_block(context, node, entering, ncb);
        },
        NodeValue::Table(_) => |context, node, entering| {
            return custom_render_table(context, node, entering);
        },
        NodeValue::TaskItem(ref nti) => |context, node, entering| {
            return custom_render_task_item(context, node, entering, nti);
        },
    });

    let arena = Arena::new();
    let root = comrak::parse_document(&arena, markdown, &options);

    let mut html = String::new();
    CustomFormatter::format_document_with_plugins(root, &options, &mut html, &plugins, ())?;

    Ok(html)
}

struct MarkdownToHtmlConfig {
    syntect: SyntectAdapter,
}

impl MarkdownToHtmlConfig {
    fn from_config(config: &ThemeConfig) -> anyhow::Result<Self> {
        let mut builder = SyntectAdapterBuilder::new();

        builder = match config.code_syntax_source {
            ThemeCodeSource::Default => builder,
            ThemeCodeSource::Theme => builder.syntax_set(SyntaxSet::load_from_folder(
                &config.current().code_syntax_dir,
            )?),
            ThemeCodeSource::Custom => builder.syntax_set(SyntaxSet::load_from_folder(
                &config.custom().code_syntax_dir,
            )?),
        };
        builder = match config.code_themes_source {
            ThemeCodeSource::Default => builder,
            ThemeCodeSource::Theme => builder.theme_set(ThemeSet::load_from_folder(
                &config.current().code_themes_dir,
            )?),
            ThemeCodeSource::Custom => builder.theme_set(ThemeSet::load_from_folder(
                &config.custom().code_themes_dir,
            )?),
        };

        let syntect = builder.theme(&config.current_code_theme).build();

        Ok(Self { syntect })
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

fn custom_render_heading<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    nh: &NodeHeading,
) -> Result<ChildRendering, std::fmt::Error> {
    if entering {
        context.cr()?;
        write!(context, "<h{}", nh.level)?;
        comrak::html::render_sourcepos(context, node)?;
        context.write_str(">")?;

        if let Some(ref prefix) = context.options.extension.header_ids {
            let text_content = comrak::html::collect_text(node);
            let id = context.anchorizer.anchorize(&text_content);
            write!(
                context,
                "<a href=\"#{}\" class=\"anchor\" id=\"{}{}\">",
                id, prefix, id
            )?;
        }
    } else {
        write!(context, "</a></h{}>", nh.level)?;
        context.lf()?;
    }

    Ok(ChildRendering::HTML)
}

fn custom_render_code_block<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    ncb: &NodeCodeBlock,
) -> Result<ChildRendering, std::fmt::Error> {
    let child_rendering = if entering {
        context
            .write_str(r#"<div class="code-block-box"><div class="code-block-header"><span>"#)?;
        if ncb.info.is_empty() {
            comrak::html::escape(context, "")?;
        } else {
            comrak::html::escape(context, &ncb.info)?;
        }
        context.write_str(
            r#"</span><button class="code-copy-btn" title="将代码复制到剪贴板"></button></div>"#,
        )?;
        comrak::html::format_node_default(context, node, entering)?
    } else {
        let child_rendering = comrak::html::format_node_default(context, node, entering)?;
        context.write_str("</div>")?;
        child_rendering
    };

    Ok(child_rendering)
}

fn custom_render_table<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
) -> Result<ChildRendering, std::fmt::Error> {
    let child_rendering = if entering {
        context.write_str(r#"<div class="table-box">"#)?;
        comrak::html::format_node_default(context, node, entering)?
    } else {
        let child_rendering = comrak::html::format_node_default(context, node, entering)?;
        context.write_str("</div>")?;
        child_rendering
    };

    Ok(child_rendering)
}

fn custom_render_task_item<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    nti: &NodeTaskItem,
) -> Result<ChildRendering, std::fmt::Error> {
    let write_li = node
        .parent()
        .map(|p| node_matches!(p, NodeValue::List(_)))
        .unwrap_or_default();

    if entering {
        context.cr()?;
        if write_li {
            context.write_str("<li")?;
            if context.options.render.tasklist_classes {
                context.write_str(" class=\"task-list-item\"")?;
            }
            comrak::html::render_sourcepos(context, node)?;
            context.write_str(">")?;
        }
        context.write_str("<input type=\"checkbox\"")?;
        if !write_li {
            comrak::html::render_sourcepos(context, node)?;
        }
        if context.options.render.tasklist_classes {
            context.write_str(" class=\"task-list-item-checkbox\"")?;
        }
        if nti.symbol.is_some() {
            context.write_str(" checked=\"\"")?;
        }
        context.write_str(" disabled=\"\" />")?;
    } else if write_li {
        context.write_str("</li>")?;
        context.lf()?;
    }

    Ok(ChildRendering::HTML)
}
