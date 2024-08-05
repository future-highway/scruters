use color_eyre::{eyre::Context, Result};
use ignore_files::IgnoreFilter;
use std::path::Path;
use tokio::sync::broadcast;
use tracing::{debug, error};
use watchexec::Watchexec;
use watchexec_filterer_ignore::IgnoreFilterer;

pub fn watch(
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
