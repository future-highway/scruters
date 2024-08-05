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
use state::State;
use std::{env::set_current_dir, panic, path::Path};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::{
        broadcast::{self, error::RecvError},
        mpsc::{self, UnboundedSender},
        watch,
    },
};
use tokio_stream::StreamExt as _;
use tracing::{debug, error, log::LevelFilter, trace};
use watchexec::Watchexec;
use watchexec_filterer_ignore::IgnoreFilterer;

mod cargo;
mod command;
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
    tui_logger::set_default_level(LevelFilter::Debug);

    tui_logger::set_level_for_target(
        "watchexec::action::worker",
        LevelFilter::Off,
    );

    trace!("Starting...");

    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .wrap_err("Failed to get cargo metadata")?;

    let root = metadata.workspace_root.as_std_path();

    // This is important for watchexec to work correctly
    set_current_dir(root).wrap_err(
        "Failed to set current directory to workspace root",
    )?;

    let (files, _) = ignore_files::from_origin(root).await;

    let mut ignore_filter = IgnoreFilter::empty(root);
    for file in &files {
        ignore_filter.add_file(file).await.wrap_err(
            "Failed to add file to ignore filter",
        )?;
    }

    let watch_broadcaster =
        watchexec_broadcast(root, ignore_filter).wrap_err(
            "Failed to create watch broadcaster",
        )?;

    let mut terminal = tui::init()?;

    let (message_tx, mut message_rx) =
        mpsc::unbounded_channel();

    let mut state = initialize_state(
        watch_broadcaster.subscribe(),
        message_tx.clone(),
    )
    .await
    .wrap_err("Error initializing state")?;

    let mut crossterm_events = EventStream::new();

    let mut sig_int_events =
        signal(SignalKind::interrupt()).wrap_err(
            "Error creating SIGINT signal stream",
        )?;

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
            _ = sig_int_events.recv() => {
                trace!("Received SIGINT");
                Some(Message::Quit)
            },
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

fn watchexec_broadcast(
    root: &Path,
    ignore_filter: IgnoreFilter,
) -> Result<broadcast::Sender<()>> {
    let (tx, _rx) = broadcast::channel(1);
    let tx_clone = tx.clone();

    let watch = Watchexec::new(move |handler| {
        _ = tx_clone.send(());
        handler
    })
    .wrap_err("Failed to create watch handler")?;

    _ = watch
        .config
        .filterer(IgnoreFilterer(ignore_filter))
        .pathset(vec![root]);

    drop(tokio::spawn(async move {
        match watch.main().await {
            Ok(inner) => match inner {
                Ok(()) => {
                    debug!("Watch handler exited");
                }
                Err(error) => {
                    error!(
                        ?error,
                        "Watch handler exited with error"
                    );
                }
            },
            Err(error) => {
                error!(
                    ?error,
                    "Watch handler exited with error"
                );
            }
        }
    }));

    Ok(tx)
}

async fn initialize_state(
    mut watch_rx: broadcast::Receiver<()>,
    message_tx: UnboundedSender<Message>,
) -> Result<State> {
    let mut state = if let Some(state) =
        State::load_from_file()
            .await
            .wrap_err("Error loading state")?
    {
        debug!("Loaded state from file");
        state
    } else {
        trace!("Creating new state");

        let state = State::new();

        state
            .save_to_file()
            .await
            .wrap_err("Error saving state")?;

        debug!("Saved initial state to file");

        state
    };

    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .wrap_err("Failed to get cargo metadata")?;

    let (metadata_tx, metadata_rx) =
        watch::channel(metadata);

    drop(tokio::spawn(async move {
        loop {
            match watch_rx.recv().await {
                Ok(()) => {}
                Err(RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }

            match MetadataCommand::new().no_deps().exec() {
                Ok(metadata) => {
                    if let Err(error) =
                        metadata_tx.send(metadata)
                    {
                        error!(
                            ?error,
                            "Failed to send metadata to watch handler",
                        );
                    }
                }
                Err(error) => {
                    error!(
                        ?error,
                        "Failed to get cargo metadata",
                    );
                }
            };
        }
    }));

    state.init(metadata_rx, message_tx);

    Ok(state)
}
