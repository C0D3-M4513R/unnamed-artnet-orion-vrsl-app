#![forbid(unsafe_code, future_incompatible, clippy::unwrap_used, clippy::panic, clippy::panic_in_result_fn, clippy::unwrap_in_result, clippy::unreachable)]
#![deny(clippy::expect_used)]
#![windows_subsystem = "windows"]

use std::sync::OnceLock;
use tokio::runtime::{Builder, Runtime};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod app;

const APP_NAME: &'static str = "Unnamed ArtNet App for Club Orion/VRSL";
static RUNTIME: OnceLock<Runtime> = OnceLock::new();
fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        #[allow(clippy::expect_used)]
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to initialize tokio runtime")
    })
}

fn main() {
    #[cfg(feature = "egui_tracing")]
    let collector = egui_tracing::EventCollector::new();
    let layered = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(tracing_subscriber::filter::filter_fn(|event|{
            if let Some(module) = event.module_path(){
                let mut bool = *event.level() == tracing_core::Level::TRACE && (module.starts_with("egui") || module.starts_with("eframe"));
                bool |= (*event.level() == tracing_core::Level::DEBUG || *event.level() == tracing_core::Level::TRACE) && module.starts_with("globset");
                !bool
            }else{
                true
            }
        }));
    #[cfg(feature = "egui_tracing")]
    let layered = {
        layered.with(collector.clone())
    };
    layered.init();
    log::info!("Logger initialized");
    let rt = get_runtime();
    let _a = rt.enter(); // "_" as a variable name immediately drops the value, causing no tokio runtime to be registered. "_a" does not.
    log::info!("Tokio Runtime initialized");
    let native_options = eframe::NativeOptions::default();
    if let Some(err) = eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| {

            #[cfg(feature = "egui_tracing")]
            return Box::new(app::App::new_collector(collector, cc));
            #[cfg(not(feature = "egui_tracing"))]
            return Box::new(app::App::new(cc));
        }),
    )
        .err()
    {
        eprintln!(
            "Error in eframe whilst trying to start the application: {}",
            err
        );
    }
    println!("GUI exited. Thank you for using DexProtectOSC-RS!");
}
