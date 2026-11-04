mod args;
mod build;
mod bundle;
mod new_project;
mod util;

use args::{BuildArgs, BundleArgs, Cli, Commands, NewArgs, RunArgs};
use clap::{
    ColorChoice, CommandFactory, FromArgMatches, builder::Styles, builder::styling::AnsiColor,
};
use std::error::Error;

fn main() {
    if let Err(error) = run() {
        Cli::command()
            .styles(cli_styles())
            .color(ColorChoice::Auto)
            .error(clap::error::ErrorKind::Io, error)
            .exit();
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut command = Cli::command().styles(cli_styles()).color(ColorChoice::Auto);
    if std::env::args_os().len() == 1 {
        command.print_help()?;
        println!();
        return Ok(());
    }

    let matches = command.get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    match cli.command {
        Some(Commands::New(args)) => create_project(args),
        Some(Commands::Build(args)) => run_build(args),
        Some(Commands::Run(args)) => run_app(args),
        Some(Commands::Bundle(args)) => run_bundle(args),
        None => {
            Cli::command()
                .styles(cli_styles())
                .color(ColorChoice::Auto)
                .print_help()?;
            println!();
            Ok(())
        }
    }
}

fn cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Cyan.on_default().bold())
        .usage(AnsiColor::Magenta.on_default().bold())
        .literal(AnsiColor::Green.on_default().bold())
        .placeholder(AnsiColor::Yellow.on_default())
        .error(AnsiColor::Red.on_default().bold())
        .valid(AnsiColor::Green.on_default().bold())
        .invalid(AnsiColor::Yellow.on_default().bold())
}

fn create_project(args: NewArgs) -> Result<(), Box<dyn Error>> {
    let options = new_project::options(args)?;
    new_project::scaffold(&options)?;
    println!("Created {}", options.output.display());
    println!(
        "  Desktop: akari run --project {}",
        options.output.display()
    );
    println!(
        "  Android: akari run --target android --project {}",
        options.output.display()
    );
    println!("  iOS:     see {}/ios/README.md", options.output.display());
    Ok(())
}

fn run_build(args: BuildArgs) -> Result<(), Box<dyn Error>> {
    build::run_project_command(args.project, args.target)
}

fn run_app(args: RunArgs) -> Result<(), Box<dyn Error>> {
    build::run_project(args.project, args.target)
}

fn run_bundle(args: BundleArgs) -> Result<(), Box<dyn Error>> {
    bundle::bundle(args.project, args.output)
}
