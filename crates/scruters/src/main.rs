//! Scruters is a TUI with various tools for Rust
//! development.

extern crate alloc;

use cargo_metadata::MetadataCommand;
use color_eyre::{
    config::HookBuilder,
    eyre::{self, Context},
    Result,
};
use crossterm::event::{Event, EventStream};
use ignore_files::IgnoreFilter;
use message::Message;
use std::{env::set_current_dir, panic};
use tokio::sync::mpsc;
use tokio_stream::StreamExt as _;
use tracing::{debug, log::LevelFilter, trace};

mod cargo;
mod command;
mod message;
mod state;
mod tui;
mod ui;
mod workspace;

#[tokio::main]
async fn main() -> Result<()> {
    install_hooks()?;

    tui_logger::init_logger(LevelFilter::Trace)
        .wrap_err("Error initializing logger")?;

    // TODO: Make these configurable via CLI arguments
    tui_logger::set_default_level(LevelFilter::Debug);

    tui_logger::set_level_for_target(
        "watchexec::action::worker",
        LevelFilter::Off,
    );

    trace!("Starting...");

    let mut terminal = tui::init()?;

    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .wrap_err("Failed to get cargo metadata")?;

    let root = metadata.workspace_root.as_std_path();

    // TODO: Check if this is actually needed...
    // This is important for watchexec to work correctly
    set_current_dir(root).wrap_err(
        "Failed to set current directory to workspace root",
    )?;

    let ignore_filter = {
        // The await here is !Send
        let (files, _) =
            ignore_files::from_origin(root).await;

        IgnoreFilter::new(root, &files)
            .await
            .wrap_err("Failed to create ignore filter")?
    };

    let workspace_changed_signal =
        workspace::watch(root, ignore_filter).wrap_err(
            "Failed to create watch broadcaster",
        )?;

    let (message_tx, mut message_rx) =
        mpsc::unbounded_channel();

    let mut state = state::initialize_state(
        workspace_changed_signal,
        message_tx.clone(),
    )
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
            message = message_rx.recv() => message
                .map_or_else(|| {
                    tracing::error!("Message channel closed");
                    Some(Message::Quit)
                }, Some),
        };

        while let Some(message) = maybe_message {
            maybe_message = state
                .handle_message(message, message_tx.clone())
                .await?;
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
