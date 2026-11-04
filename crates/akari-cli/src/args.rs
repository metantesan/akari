use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "akari",
    version,
    about = "Create, build, and package Akari applications",
    subcommand_required = false
)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub(super) enum Commands {
    /// Create a separate Akari application project.
    New(NewArgs),
    /// Build an application for the host or a selected target.
    Build(BuildArgs),
    /// Bundle the current host build as an AppImage or macOS package.
    Bundle(BundleArgs),
}

#[derive(Debug, Args)]
pub(super) struct NewArgs {
    /// App display name. If omitted, Akari asks for one.
    #[arg(value_name = "APP_NAME")]
    pub(super) name: Option<String>,
    /// Rust package name. Derived from the display name when omitted.
    #[arg(long, value_name = "SLUG")]
    pub(super) package: Option<String>,
    /// Reverse-DNS application ID for platform bundles, for example `com.akari.dev`.
    #[arg(long = "app-id", alias = "identifier", value_name = "APP_ID")]
    pub(super) identifier: Option<String>,
    /// Path to an Akari checkout for local development.
    #[arg(long = "path", value_name = "CHECKOUT")]
    pub(super) akari_path: Option<PathBuf>,
    /// Output project directory.
    #[arg(short, long, value_name = "DIR")]
    pub(super) output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub(super) struct BuildArgs {
    /// Application project directory. Defaults to the current directory.
    #[arg(long, value_name = "DIR")]
    pub(super) project: Option<PathBuf>,
    /// `android` or a Rust compilation target triple.
    #[arg(long, value_name = "TARGET")]
    pub(super) target: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct BundleArgs {
    /// Application project directory. Defaults to the current directory.
    #[arg(long, value_name = "DIR")]
    pub(super) project: Option<PathBuf>,
    /// Output package file path.
    #[arg(long, value_name = "FILE")]
    pub(super) output: Option<PathBuf>,
}
