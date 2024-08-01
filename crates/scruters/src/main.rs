//! Scruters is a TUI with various tools for Rust
//! development.

use color_eyre::{
    config::HookBuilder,
    eyre::{self, Context},
    Result,
};
use crossterm::event::{Event, EventStream};
use message::Message;
use state::State;
use std::panic;
use tokio_stream::StreamExt as _;
use tracing::{debug, log::LevelFilter, trace};

mod message;
mod state;
mod tui;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    install_hooks()?;

    tui_logger::init_logger(LevelFilter::Trace)
        .wrap_err("Error initializing logger")?;

    // TODO: Make these configurable via CLI arguments
    tui_logger::set_default_level(LevelFilter::Info);
    tui_logger::set_level_for_target(
        "scruters",
        LevelFilter::Trace,
    );

    trace!("Starting Scruters");

    let mut terminal = tui::init()?;

    let mut state = initialize_state()
        .await
        .wrap_err("Error initializing state")?;

    let mut crossterm_events = EventStream::new();

    while state.current_screen.is_some() {
        _ = terminal
            .draw(|frame| ui::draw(&mut state, frame))
            .wrap_err("Error drawing UI")?;

        #[allow(clippy::integer_division_remainder_used)]
        let mut maybe_message = tokio::select! {
            event = crossterm_events.next() => event.map_or_else(
                || {
                    debug!("Crossterm event stream ended");
                    Some(Message::Quit)
                },
                |event| match event {
                    Ok(Event::Key(event)) => Some(event.into()),
                    Ok(_) => None,
                    Err(error) => {
                        tracing::error!(?error, "Error reading crossterm event");
                        None
                    },
                }
            ),
        };

        while let Some(message) = maybe_message {
            maybe_message =
                state.handle_message(message).await?;
        }
    }

    tui::restore()?;

    Ok(())
}

/// This replaces the standard `color_eyre` panic and error
/// hooks with hooks that restore the terminal before
/// printing the panic or error.
///
/// Source: <https://ratatui.rs/recipes/apps/color-eyre/>
fn install_hooks() -> Result<()> {
    // add any extra configuration you need to the hook
    // builder
    let hook_builder = HookBuilder::default();
    let (panic_hook, eyre_hook) = hook_builder.into_hooks();

    // convert from a color_eyre PanicHook to a standard
    // panic hook
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        drop(tui::restore()); // ignore any errors as we are already failing
        panic_hook(panic_info);
    }));

    // convert from a color_eyre EyreHook to a eyre
    // ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(move |error| {
        drop(tui::restore()); // ignore any errors as we are already failing
        eyre_hook(error)
    }))?;

    Ok(())
}

async fn initialize_state() -> Result<State> {
    let state = if let Some(state) = State::load_from_file()
        .await
        .wrap_err("Error loading state")?
    {
        debug!("Loaded state from file");
        state
    } else {
        trace!("Creating new state");

        let state = State::default();

        state
            .save_to_file()
            .await
            .wrap_err("Error saving state")?;

        debug!("Saved initial state to file");

        state
    };

    Ok(state)
}
