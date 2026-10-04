mod alerts;
mod app;
mod backend;
mod cli;
mod config;
mod dbus;
mod features;
mod logging;
mod runtime;
mod ui;

fn main() -> std::process::ExitCode {
    let options = match cli::Options::parse(std::env::args_os().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}\nRun topbar --help for usage.");
            return std::process::ExitCode::from(2);
        }
    };
    if options.help {
        print!("{}", cli::HELP);
        return std::process::ExitCode::SUCCESS;
    }
    let (settings, watcher) = match config::load(options.config) {
        Ok(loaded) => loaded,
        Err(error) => {
            eprintln!("{error}");
            if !options.validate {
                runtime::block_on(alerts::config::startup_error(&error));
            }
            return std::process::ExitCode::FAILURE;
        }
    };
    if options.validate {
        println!("Configuration is valid: {}", watcher.path().display());
        return std::process::ExitCode::SUCCESS;
    }
    logging::load();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting topbar");

    match app::run(settings, watcher) {
        Ok(()) => {
            tracing::info!("topbar stopped");

            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            tracing::error!(%error, "topbar failed");
            runtime::block_on(alerts::config::startup_error(&error));

            std::process::ExitCode::FAILURE
        }
    }
}
