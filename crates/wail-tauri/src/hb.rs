use std::sync::OnceLock;

use futures::compat::Future01CompatExt;
use honeybadger::{ConfigBuilder, Honeybadger};

const API_KEY: &str = "hbp_0GYAg4zTkkp5dnhFf4k3Ke9rvmfvA62vKf8O";

static HB_INIT: OnceLock<()> = OnceLock::new();

pub fn init() {
    HB_INIT.get_or_init(|| ());
}

fn make_client() -> Option<Honeybadger> {
    if HB_INIT.get().is_none() {
        return None;
    }
    let config = ConfigBuilder::new(API_KEY).with_env("production").build();
    Honeybadger::new(config).ok()
}

fn make_notice(message: &str) -> honeybadger::notice::Error {
    honeybadger::notice::Error {
        class: message.to_string(),
        message: Some(message.to_string()),
        causes: None,
    }
}

/// Report an error string to Honeybadger from an async context.
pub async fn report(message: &str) {
    let Some(hb) = make_client() else { return };
    let _ = hb.notify(make_notice(message), None).compat().await;
}

/// Install a global panic hook that fires Honeybadger in a background thread.
pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        std::thread::spawn(move || {
            let Some(hb) = make_client() else { return };
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            if let Ok(rt) = rt {
                let _ = rt.block_on(hb.notify(make_notice(&msg), None).compat());
            }
        });
    }));
}
