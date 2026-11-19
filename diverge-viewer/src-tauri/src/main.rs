//! Diverge Viewer.
//!
//! Empty up-front. Rust holds the daemon's address, the identity it is
//! dialled as, and the one connection to it; the page speaks the
//! daemon's protocol through the TypeScript SDK over the SDK's
//! `tauri_proxy` runner, and holds nothing else. See
//! `diverge-viewer/CLAUDE.md`.
//!
//! The root — `--config <dir>`, else `DIVERGE_CONFIG`, else `~/.diverge`
//! — is the one directory every Diverge program on the host shares, and
//! the app reads the viewer's settings in it ([`config`]): the daemons
//! it may dial, each with the accounts it may dial it as, as the CLI
//! keeps its own. A start names one of each with `--daemon <name>` and
//! `--account <name>`, or takes the defaults, and the window's title
//! says which daemon it is on. The app keeps no files of its own; it
//! moves the person's between their disk and an entity's when asked
//! (`transfer`).
//!
//! One viewer runs on a root, as one daemon and one provider do: the
//! app holds the root's `viewer.lock` for its life, with its process
//! id inside, and hears `POST /stop` on a loopback port of its own
//! ([`stop`]) — which is how `diverge viewer start` knows it is up,
//! how `diverge viewer stop` ends it, and why a setting is not
//! written while it runs.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use diverge_sdk::identities::{Identities, Name, Presenter};
use tauri::Manager;

mod config;
mod frontend;
mod stop;
mod transfer;

fn main() {
    tauri::Builder::default()
        .manage(transfer::Transfers::default())
        .manage(frontend::Frontends::default())
        .setup(|app| {
            // The root, as the daemon and the provider find theirs;
            // its lock, taken before anything else and kept for the
            // app's life, so a second viewer on the root is refused
            // here; the settings in it, and the daemon and the account
            // this start is; the root's identities; and the loopback
            // port the stop is heard on. Any of them failing stops the
            // start, saying why.
            let (selected, redial, identities, held, listener) = tauri::async_runtime::block_on(async {
                let arguments = config::arguments(std::env::args_os().skip(1))?;
                let root = config::root(arguments.config).await?;
                let lock = root.join(config::LOCK_FILE);
                let Some(held) = diverge_sdk::file_lock::try_exclusive(&lock).await? else {
                    let holder = diverge_sdk::file_lock::holder(&lock).await?.unwrap_or_default();
                    return Err(format!("a viewer is already running on this root, as process {holder}: stop it first, with `diverge viewer stop`").into());
                };
                let config = config::load(&root).await?;
                let selected = config::select(&config, arguments.daemon.as_deref(), arguments.account.as_deref())?;
                let identities = Identities::open(&root).await?;
                let listener = tokio::net::TcpListener::bind(("127.0.0.1", config.port))
                    .await
                    .map_err(|error| format!("the viewer's port, {}, could not be listened on: {error}", config.port))?;
                // The id last, once everything that can refuse the
                // start has not: a start that reads it knows the app
                // is up.
                held.write(std::process::id().to_string().into_bytes()).await?;
                Ok::<_, Box<dyn std::error::Error>>((selected, config.redial, identities, held, listener))
            })?;
            app.manage(held);
            stop::serve(app.handle().clone(), listener);
            // The daemon's connection, held for the page: the SDK's
            // runner, presenting the account's identity and its key,
            // holding the daemon to the key its own token names — and
            // to the identity the settings state for it, where they
            // state one — and dialling again as `redial` says.
            // What the page keeps the Home board's places under: the
            // ID a daemon's record of this daemon, reached as this
            // account by any route, would have — computed as the daemon
            // computes a record's — where the settings state who the
            // daemon is; else its name in the settings, which is all
            // there is to tell it by. A different daemon, or a
            // different account on it, is a different board.
            let home = match &selected.daemon.identity {
                Some(daemon) => {
                    let account = diverge_sdk::shared::Identity { issuer: selected.account.issuer.clone(), subject: selected.account.subject.clone() };
                    diverge_sdk::daemon::endpoints::daemons::DaemonId::of(daemon, &account, &Default::default()).to_string()
                }
                None => format!("home:{}", selected.daemon_name),
            };
            let name = Name { issuer: selected.account.issuer, subject: selected.account.subject };
            let presenter = Arc::new(Presenter::new(identities, name).expecting(selected.daemon.identity));
            app.manage(diverge_sdk::tauri_proxy::TauriProxy::new(selected.daemon.address, presenter, redial));
            // The window, made here and not by the configuration, so a
            // link that opens a new window can be answered: the page or
            // a frontend's frame asking for one — a link with a target,
            // `window.open` — is never given a window of the app's. An
            // `http` or `https` address opens in the person's own
            // browser; anything else opens nothing. Its title names
            // the daemon this start is on.
            tauri::WebviewWindowBuilder::from_config(app.handle(), &app.config().app.windows[0])?
                .title(format!("Diverge Viewer — {}", selected.daemon_name))
                // Read by the page before any of its own code runs:
                // see `home` above.
                .initialization_script(format!("window.__DIVERGE_HOME__ = {};", serde_json::Value::String(home)))
                .on_new_window(|url, _| {
                    if matches!(url.scheme(), "http" | "https") {
                        let _ = webbrowser::open(url.as_str());
                    }
                    tauri::webview::NewWindowResponse::Deny
                })
                .build()?;
            Ok(())
        })
        // A page starting to load — a reload above all — leaves the old
        // page's streams with nobody to take them: the proxy lets them
        // go and resets the connection, so the daemon ends what the old
        // page had open.
        .on_page_load(|webview, payload| {
            if payload.event() == tauri::webview::PageLoadEvent::Started {
                let app = webview.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    frontend::page_loading(&app).await;
                    app.state::<diverge_sdk::tauri_proxy::TauriProxy>().page_loading().await;
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            diverge_sdk::tauri_proxy::wire_request,
            diverge_sdk::tauri_proxy::wire_channel_request,
            diverge_sdk::tauri_proxy::wire_channel_response,
            diverge_sdk::tauri_proxy::wire_channel_response_finish,
            diverge_sdk::tauri_proxy::wire_took,
            diverge_sdk::tauri_proxy::wire_dropped,
            diverge_sdk::tauri_proxy::wire_status,
            diverge_sdk::tauri_proxy::wire_reset,
            transfer::files_download,
            transfer::files_upload,
            transfer::files_cancel,
            frontend::frontend_open,
            frontend::frontend_close,
        ])
        .run(tauri::generate_context!())
        .expect("the app could not start");
}
