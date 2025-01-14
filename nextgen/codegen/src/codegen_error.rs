use miette::Diagnostic;
use moon_common::{Id, Style, Stylize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum CodegenError {
    #[diagnostic(code(codegen::template::exists))]
    #[error(
        "A template with the name {} already exists at {}.",
        .0.style(Style::Id),
        .1.style(Style::Path),
    )]
    ExistingTemplate(Id, PathBuf),

    #[diagnostic(code(codegen::var::parse_failed))]
    #[error("Failed to parse variable argument --{0}: {1}")]
    FailedToParseArgVar(String, String),

    #[diagnostic(code(codegen::template::missing))]
    #[error(
        "No template with the name {} could not be found at any of the configured template paths.",
        .0.style(Style::Id),
    )]
    MissingTemplate(Id),

    #[diagnostic(code(codegen::template_file::load_failed))]
    #[error(
        "Failed to load template file {}.",
        .path.style(Style::Path),
    )]
    LoadTemplateFileFailed {
        path: PathBuf,
        #[source]
        error: tera::Error,
    },

    #[diagnostic(code(codegen::template_file::render_failed))]
    #[error(
        "Failed to render template file {}.",
        .path.style(Style::Path),
    )]
    RenderTemplateFileFailed {
        path: PathBuf,
        #[source]
        error: tera::Error,
    },

    #[diagnostic(code(codegen::template_file::interpolate_path))]
    #[error(
        "Failed to interpolate variables into template file path {}.",
        .path.style(Style::File),
    )]
    InterpolateTemplateFileFailed {
        path: String,
        #[source]
        error: tera::Error,
    },
}
