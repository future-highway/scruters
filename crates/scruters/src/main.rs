//! Scruters is a TUI with various tools for Rust
//! development.

use color_eyre::{config::HookBuilder, eyre, Result};
use state::State;
use std::panic;

mod state;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    install_hooks()?;

    let state = if let Some(state) =
        State::load_from_file().await
    {
        state
    } else {
        let state = State::default();
        state.save_to_file().await;
        state
    };

    let _tui = tui::init()?;

    tui::restore()?;

    todo!("{state:?}");
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
