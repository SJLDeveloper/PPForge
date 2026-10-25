//! PP Forge — character builder and manager for Prowlers & Paragons Ultimate Edition.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod engine;
mod model;
mod pdf;
mod rules;
mod storage;
mod ui;

fn options(renderer: eframe::Renderer) -> eframe::NativeOptions {
    eframe::NativeOptions {
        renderer,
        viewport: egui::ViewportBuilder::default()
            .with_title("PP Forge — Prowlers & Paragons Character Builder")
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([960.0, 600.0])
            .with_drag_and_drop(true),
        ..Default::default()
    }
}

fn main() -> eframe::Result<()> {
    // Command-line export for scripting: pp_forge --pdf <in.ppchar> <out.pdf>
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 4 && args[1] == "--pdf" {
        let c = storage::load_character(std::path::Path::new(&args[2])).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1)
        });
        if let Err(e) = pdf::export(&c, std::path::Path::new(&args[3])) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return Ok(());
    }
    // Prefer wgpu; fall back to OpenGL on machines without a working wgpu backend.
    let first = eframe::run_native("PP Forge", options(eframe::Renderer::Wgpu), Box::new(|cc| Ok(Box::new(app::App::new(cc)))));
    match first {
        Err(e) => {
            eprintln!("wgpu renderer unavailable ({e}); falling back to OpenGL");
            eframe::run_native("PP Forge", options(eframe::Renderer::Glow), Box::new(|cc| Ok(Box::new(app::App::new(cc)))))
        }
        ok => ok,
    }
}
