//! Aubrieta Creative Studio Desktop Application (08.7, 08.15, 09.24).
//!
//! Interactive creative desktop suite providing vector and photo layout editing,
//! One-Tree object hierarchy, appearance stacks, multi-surface artboards,
//! variable data merge, and live interactive canvas tools.
//!
//! Usage:
//!   aubrieta-desktop                  # Launch interactive desktop window
//!   aubrieta-desktop --smoke-test     # Run headless end-to-end verification
//!   aubrieta-desktop --headless       # Run in headless mode

mod app;
mod font;
mod ui_renderer;

use app::DesktopApp;
use aubrieta_ui_gpui::tools::{PointerButton, ToolKind};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use std::env;

const DEFAULT_WIDTH: usize = 1280;
const DEFAULT_HEIGHT: usize = 800;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check for flags
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("Aubrieta Design Studio v0.1.0");
        return;
    }

    let mut app = DesktopApp::new(DEFAULT_WIDTH, DEFAULT_HEIGHT);

    if args
        .iter()
        .any(|a| a == "--smoke-test" || a == "--headless")
    {
        println!("aubrieta-desktop: Running automated smoke test...");
        if let Err(err) = app.smoke_test() {
            eprintln!("aubrieta-desktop smoke test failed: {err}");
            std::process::exit(1);
        }
        println!("aubrieta-desktop: Smoke test PASSED successfully.");
        return;
    }

    // Interactive graphical window execution
    run_interactive_window(app);
}

fn print_help() {
    println!("Aubrieta Creative Studio — Desktop Vector & Layout Suite");
    println!("Usage:");
    println!("  aubrieta-desktop [options]");
    println!();
    println!("Options:");
    println!("  --smoke-test    Run automated end-to-end headless verification");
    println!("  --headless      Run in headless mode without window creation");
    println!("  -h, --help      Display this help information");
    println!("  -V, --version   Display version string");
}

fn run_interactive_window(mut app: DesktopApp) {
    let display_available = env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok();

    if !display_available {
        println!("aubrieta-desktop: No display server detected (DISPLAY/WAYLAND_DISPLAY unset).");
        println!("aubrieta-desktop: Falling back to headless smoke test verification...");
        if let Err(err) = app.smoke_test() {
            eprintln!("aubrieta-desktop verification failed: {err}");
            std::process::exit(1);
        }
        println!("aubrieta-desktop: Headless execution verified successfully.");
        return;
    }

    let win_opts = WindowOptions {
        resize: true,
        scale: minifb::Scale::X1,
        ..WindowOptions::default()
    };

    let mut window = match Window::new(
        "Aubrieta Design Studio — Professional Vector & Layout",
        DEFAULT_WIDTH,
        DEFAULT_HEIGHT,
        win_opts,
    ) {
        Ok(win) => win,
        Err(err) => {
            eprintln!(
                "aubrieta-desktop: Failed to open window ({err}). Falling back to headless..."
            );
            if let Err(e) = app.smoke_test() {
                eprintln!("smoke test failed: {e}");
                std::process::exit(1);
            }
            return;
        }
    };

    // Target 60 FPS update rate
    window.set_target_fps(60);

    let mut prev_left_down = false;
    let mut prev_right_down = false;
    let mut prev_middle_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // 1. Handle window resize
        let (cur_w, cur_h) = window.get_size();
        if cur_w != app.fb.width || cur_h != app.fb.height {
            app.resize(cur_w, cur_h);
        }

        // 2. Keyboard modifiers
        app.shift_held = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
        app.alt_held = window.is_key_down(Key::LeftAlt) || window.is_key_down(Key::RightAlt);
        app.ctrl_held = window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl);
        app.is_panning = window.is_key_down(Key::Space);

        // 3. Single key shortcut commands
        for key in window.get_keys_pressed(KeyRepeat::No) {
            match key {
                Key::V => app.shell.set_active_tool(ToolKind::Select),
                Key::A => app.shell.set_active_tool(ToolKind::Node),
                Key::P => app.shell.set_active_tool(ToolKind::Pen),
                Key::M => app.shell.set_active_tool(ToolKind::Rectangle),
                Key::E => app.shell.set_active_tool(ToolKind::Ellipse),
                Key::G => app.shell.set_active_tool(ToolKind::Polygon),
                Key::Z if app.ctrl_held => {
                    let _ = app.shell.undo();
                }
                Key::Y if app.ctrl_held => {
                    let _ = app.shell.redo();
                }
                Key::Key1 => app.active_tab = crate::ui_renderer::ActiveDockTab::Layers,
                Key::Key2 => app.active_tab = crate::ui_renderer::ActiveDockTab::Properties,
                Key::Key3 => app.active_tab = crate::ui_renderer::ActiveDockTab::History,
                Key::Key4 => app.active_tab = crate::ui_renderer::ActiveDockTab::DataMerge,
                _ => {}
            }
        }

        // 4. Mouse position and movement
        if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Pass) {
            let ix = mx as usize;
            let iy = my as usize;
            app.on_mouse_move(ix, iy);

            // Left button
            let left_down = window.get_mouse_down(MouseButton::Left);
            if left_down && !prev_left_down {
                app.on_mouse_down(ix, iy, PointerButton::Primary);
            } else if !left_down && prev_left_down {
                app.on_mouse_up(ix, iy, PointerButton::Primary);
            }
            prev_left_down = left_down;

            // Right button
            let right_down = window.get_mouse_down(MouseButton::Right);
            if right_down && !prev_right_down {
                app.on_mouse_down(ix, iy, PointerButton::Secondary);
            } else if !right_down && prev_right_down {
                app.on_mouse_up(ix, iy, PointerButton::Secondary);
            }
            prev_right_down = right_down;

            // Middle button
            let middle_down = window.get_mouse_down(MouseButton::Middle);
            if middle_down && !prev_middle_down {
                app.on_mouse_down(ix, iy, PointerButton::Middle);
            } else if !middle_down && prev_middle_down {
                app.on_mouse_up(ix, iy, PointerButton::Middle);
            }
            prev_middle_down = middle_down;
        }

        // 5. Mouse scroll wheel zoom
        if let Some((_scroll_x, scroll_y)) = window.get_scroll_wheel() {
            if scroll_y.abs() > 0.01 {
                app.on_scroll(scroll_y as f64);
            }
        }

        // 6. Render frame buffer and present to window
        app.render();
        let width = app.fb.width;
        let height = app.fb.height;
        if let Err(e) = window.update_with_buffer(&app.fb.pixels, width, height) {
            eprintln!("aubrieta-desktop: Window update error: {e}");
            break;
        }
    }
}
