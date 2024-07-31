//! Scruters is a TUI with various tools for Rust
//! development.

use state::State;

mod state;

#[tokio::main]
async fn main() {
    let state = if let Some(state) =
        State::load_from_file().await
    {
        state
    } else {
        let state = State::default();
        state.save_to_file().await;
        state
    };

    println!("{state:?}");
}
