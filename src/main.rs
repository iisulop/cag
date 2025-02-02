use cag::app::run_app;
use cag::error::Error;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tracing::{error, trace, Level};

const ENVIRONMENT_VARIABLE_ENABLE_TRACING: &str = "ENABLE_TRACING";

fn main() -> Result<(), Error> {
    if let Ok(enable_tracing) = std::env::var(ENVIRONMENT_VARIABLE_ENABLE_TRACING) {
        if enable_tracing == "1" || &enable_tracing.to_lowercase() == "true" {
            let file_appender = tracing_appender::rolling::hourly("./.logs/", "runlog");
            tracing_subscriber::fmt()
                .with_max_level(Level::TRACE)
                .with_writer(file_appender)
                .init();
        }
    }
    trace!("Enabling raw mode");
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    trace!("Disabling raw mode");

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        error!("{:?}", err);
        eprintln!("{err}");
    }

    Ok(())
}
