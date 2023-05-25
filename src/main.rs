#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    init_logs(log::LevelFilter::Debug);

    // Log to stdout (if you run with `RUST_LOG=debug`).
    // tracing_subscriber::fmt::init();


    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(wave_gen::App::new(cc))),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(wave_gen::App::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}


fn init_logs(log_level: log::LevelFilter) {
    let colors = fern::colors::ColoredLevelConfig::default()
        .info(fern::colors::Color::Blue)
        .debug(fern::colors::Color::Yellow)
        .trace(fern::colors::Color::Magenta);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}{}[{}][{}{color_line}]\x1B[0m {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors.color(record.level()),
                message,
                color_line =
                    format_args!("\x1B[{}m", colors.get_color(&record.level()).to_fg_str())
            ))
        })
        .level(log_level)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}