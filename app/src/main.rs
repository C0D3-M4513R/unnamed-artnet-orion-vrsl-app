#![forbid(
    unsafe_code,
    future_incompatible,
    clippy::unwrap_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unreachable,
    clippy::suspicious_xor_used_as_pow,
    clippy::semicolon_inside_block,
    clippy::mod_module_files,
    clippy::missing_assert_message,
    clippy::mem_forget,
    clippy::map_err_ignore,
    clippy::indexing_slicing,
    clippy::fn_to_numeric_cast_any,
    clippy::float_cmp_const,
    clippy::float_arithmetic,
    clippy::exit,
    clippy::error_impl_error,
    clippy::empty_structs_with_brackets,
    clippy::empty_drop,
    clippy::disallowed_script_idents,
    clippy::deref_by_slicing,
    clippy::default_union_representation,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason
)]
#![deny(clippy::expect_used)]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::else_if_without_else,
    clippy::dbg_macro,
    clippy::create_dir
)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![windows_subsystem = "windows"]

use std::sync::OnceLock;
use log::LevelFilter;
use tokio::runtime::{Builder, Runtime};


#[cfg(all(feature = "simple_logger", feature = "tracing_subscriber"))]
compile_error!("The features `simple_logger` and `tracing_subscriber` are not compatible");

mod app;
mod artnet;
mod u9;
mod fixturestore;
mod degree;

const APP_NAME: &str = "Unnamed ArtNet App for Club Orion/VRSL";
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

#[cfg(feature = "tracing_subscriber")]
fn init_logging(){
    use tracing_subscriber::{
        layer::SubscriberExt,
        util::SubscriberInitExt
    };
    println!("Setting up tracing_subscriber logger.");
    #[cfg(feature = "egui_tracing")]
        let collector = egui_tracing::EventCollector::new();
    let layered = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty());
    #[cfg(feature = "egui_tracing")]
        let layered = {
        layered.with(tracing_subscriber::filter::filter_fn(|event|{
            if let Some(module) = event.module_path(){
                let mut bool = *event.level() == tracing_core::Level::TRACE && (module.starts_with("egui") || module.starts_with("eframe"));
                bool |= (*event.level() == tracing_core::Level::DEBUG || *event.level() == tracing_core::Level::TRACE) && module.starts_with("globset");
                !bool
            }else{
                true
            }
        })).with(collector.clone())
    };
    layered.init();
    #[cfg(not(debug_assertions))]
    log::info!("You are running a release build. Some log statements were disabled.");
}

#[cfg(feature = "simple_logger")]
fn init_logging() {
        println!("Setting up simple_logger.");
        #[allow(clippy::expect_used)]
        simple_logger::SimpleLogger::new()
            .with_utc_timestamps()
            .with_colors(true)
            .with_level(LevelFilter::Info)
            .env()
            .init()
            .expect("unable to initialize logger");
}

#[cfg(not(any(feature = "simple_logger", feature = "tracing_subscriber")))]
fn init_logging() {
    println!("Not setting up any logger.");
}

fn main() {
    init_logging();
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
            "Error in eframe whilst trying to start the application: {err}"
        );
    }
    println!("GUI exited. Thank you for using {APP_NAME}!");
}
