use moon_error::MoonError;
use moon_target::TargetError;
use moon_utils::process::ArgsParseError;
use starbase_styles::{Style, Stylize};
use starbase_utils::glob::GlobError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Failed to parse env file {}: {1}", .0.style(Style::Path))]
    InvalidEnvFile(PathBuf, String),

    #[error(
        "Task outputs must be project relative and cannot be absolute. Found {} in {}.", .0.style(Style::File), .1.style(Style::Label)
    )]
    NoAbsoluteOutput(String, String),

    #[error(
        "Task outputs must be project relative and cannot traverse upwards. Found {} in {}.", .0.style(Style::File), .1.style(Style::Label)
    )]
    NoParentOutput(String, String),

    #[error("Target {} defines the output {}, but this output does not exist after being ran.", .0.style(Style::Label), .1.style(Style::File))]
    MissingOutput(String, String),

    #[error(transparent)]
    ArgsParse(#[from] ArgsParseError),

    #[error(transparent)]
    FileGroup(#[from] FileGroupError),

    #[error(transparent)]
    Moon(#[from] MoonError),

    #[error(transparent)]
    Target(#[from] TargetError),
}

#[derive(Error, Debug)]
pub enum FileGroupError {
    #[error("No globs defined in file group {}.", .0.style(Style::Id))]
    NoGlobs(String), // file group

    #[error(transparent)]
    Glob(#[from] GlobError),

    #[error(transparent)]
    Moon(#[from] MoonError),
}
