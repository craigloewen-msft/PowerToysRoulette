use eframe::egui;
use log::{debug, error, info};
use std::f32::consts::PI;
use std::process::Command;

use crate::wheel::{Wheel, WheelItem};

pub struct WheelApp {
    // Wheel configuration
    wheel: Wheel,

    // Animation state
    rotation: f32,
    spinning: bool,
    target_rotation: f32,
    animation_time: f32,

    // Result state
    winning_item: Option<WheelItem>,
    show_message: bool,

    // Auto-run mode
    auto_run: bool,
    auto_run_started: bool,
}

impl Default for WheelApp {
    fn default() -> Self {
        let wheel = Wheel::new();

        info!("Initializing WheelApp with {} items", wheel.items().len());
        for (i, item) in wheel.items().iter().enumerate() {
            debug!("Item {}: {} -> {}", i, item.name, item.executable);
        }

        Self {
            wheel,
            rotation: 0.0,
            spinning: false,
            target_rotation: 0.0,
            animation_time: 0.0,
            winning_item: None,
            show_message: false,
            auto_run: false,
            auto_run_started: false,
        }
    }
}

impl eframe::App for WheelApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Return fully transparent clear color for auto-run mode
        if self.auto_run {
            [0.0, 0.0, 0.0, 0.0] // Fully transparent
        } else {
            egui::Rgba::from_rgb(255.0, 255.0, 255.0).to_array() // Default light background
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set transparent background for auto-run mode
        if self.auto_run {
            ctx.style_mut(|style| {
                style.visuals.window_fill = egui::Color32::TRANSPARENT;
                style.visuals.panel_fill = egui::Color32::TRANSPARENT;
            });
        }

        // Auto-start spin in auto-run mode
        if self.auto_run && !self.auto_run_started && !self.spinning {
            info!("Auto-run mode: starting spin automatically");
            self.start_spin();
            self.auto_run_started = true;
        }

        // Update animation
        if self.spinning {
            self.animation_time += ctx.input(|i| i.unstable_dt);
            let duration = 4.0; // 4 seconds spin

            if self.animation_time >= duration {
                // Animation complete
                info!("Spin animation completed");
                self.spinning = false;
                self.rotation = self.target_rotation % (2.0 * PI);

                // Calculate which segment is actually at the top pointer position
                let segment_index = self.wheel.get_segment_at_rotation(self.rotation);
                debug!("Calculated winning segment index: {}", segment_index);
                let winning_item = &self.wheel.items()[segment_index];
                info!(
                    "Winner: {} ({})",
                    winning_item.name, winning_item.executable
                );

                // Launch the selected program
                self.launch_program(&winning_item.executable);

                self.winning_item = Some(winning_item.clone());

                // Close app after short delay in auto-run mode
                if self.auto_run {
                    info!("Auto-run mode: closing application after launch");
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            } else {
                // Ease out cubic for smooth deceleration
                let t = self.animation_time / duration;
                let eased = 1.0 - (1.0 - t).powi(3);
                self.rotation = self.target_rotation * eased;

                // Request continuous repaint while spinning
                ctx.request_repaint();
            }
        }

        // Configure transparent background in auto-run mode
        let panel = if self.auto_run {
            egui::CentralPanel::default().frame(
                egui::Frame::none()
                    .fill(egui::Color32::TRANSPARENT)
            )
        } else {
            egui::CentralPanel::default()
        };

        panel.show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Only show title and spacing in normal mode
                if !self.auto_run {
                    ui.add_space(20.0);
                    ui.heading("🎰 Prize Wheel Roulette 🎰");
                    ui.add_space(20.0);
                } else {
                    ui.add_space(100.0); // Center the wheel vertically
                }

                // Draw the wheel
                let wheel_size = 400.0;
                let (response, painter) =
                    ui.allocate_painter(egui::vec2(wheel_size, wheel_size), egui::Sense::hover());

                let center = response.rect.center();
                let radius = wheel_size / 2.0 - 10.0;

                // Draw wheel segments
                let num_segments = self.wheel.items().len();
                let angle_per_segment = 2.0 * PI / num_segments as f32;

                for i in 0..num_segments {
                    let start_angle = i as f32 * angle_per_segment + self.rotation - PI / 2.0;
                    let end_angle = start_angle + angle_per_segment;

                    let item = &self.wheel.items()[i];
                    // Draw segment
                    self.draw_segment(
                        &painter,
                        center,
                        radius,
                        start_angle,
                        end_angle,
                        item.color,
                        &item.name,
                    );
                }

                // Draw outer circle border
                painter.circle_stroke(
                    center,
                    radius,
                    egui::Stroke::new(4.0, egui::Color32::from_rgb(50, 50, 50)),
                );

                // Draw center circle
                painter.circle_filled(center, 20.0, egui::Color32::from_rgb(50, 50, 50));
                painter.circle_stroke(center, 20.0, egui::Stroke::new(2.0, egui::Color32::WHITE));

                // Draw pointer at top - pointing DOWN into the wheel
                let pointer_tip_y = center.y - radius + 5.0; // Just inside the wheel
                let pointer_base_y = center.y - radius - 30.0; // Above the wheel
                let pointer_points = vec![
                    egui::pos2(center.x, pointer_tip_y),         // Tip pointing down
                    egui::pos2(center.x - 15.0, pointer_base_y), // Left base
                    egui::pos2(center.x + 15.0, pointer_base_y), // Right base
                ];
                painter.add(egui::Shape::convex_polygon(
                    pointer_points,
                    egui::Color32::from_rgb(255, 50, 50),
                    egui::Stroke::new(3.0, egui::Color32::from_rgb(150, 0, 0)),
                ));

                // Only show buttons and messages in normal mode
                if !self.auto_run {
                    ui.add_space(30.0);

                    // Spin button
                    let button_text = if self.spinning {
                        "Spinning..."
                    } else {
                        "🎲 SPIN 🎲"
                    };
                    let button =
                        egui::Button::new(egui::RichText::new(button_text).size(28.0).strong());

                    if ui.add_sized([200.0, 60.0], button).clicked() && !self.spinning {
                        info!("Spin button clicked, starting new spin");
                        self.start_spin();
                    }

                    ui.add_space(20.0);

                    // Show winning message
                    if let Some(winning_item) = &self.winning_item {
                        ui.add_space(10.0);

                        ui.label(
                            egui::RichText::new(format!("🎉 Congratulations! 🎉"))
                                .size(32.0)
                                .color(egui::Color32::from_rgb(50, 200, 50))
                                .strong(),
                        );

                        ui.label(
                            egui::RichText::new(format!("Launching: {}", winning_item.name))
                                .size(28.0)
                                .color(egui::Color32::from_rgb(255, 215, 0))
                                .strong(),
                        );
                    }
                }
            });
        });
    }
}

impl WheelApp {
    fn draw_segment(
        &self,
        painter: &egui::Painter,
        center: egui::Pos2,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        color: egui::Color32,
        name: &str,
    ) {
        // Draw filled segment
        let steps = 20;
        let mut points = vec![center];

        for i in 0..=steps {
            let angle = start_angle + (end_angle - start_angle) * (i as f32 / steps as f32);
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            points.push(egui::pos2(x, y));
        }

        painter.add(egui::Shape::convex_polygon(
            points,
            color,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(40, 40, 40)),
        ));

        // Draw text with background for better visibility
        let mid_angle = (start_angle + end_angle) / 2.0;
        let text_radius = radius * 0.7;
        let text_x = center.x + text_radius * mid_angle.cos();
        let text_y = center.y + text_radius * mid_angle.sin();
        let text_pos = egui::pos2(text_x, text_y);

        // Measure text size for background
        let font_id = egui::FontId::proportional(20.0);
        let galley =
            painter.layout_no_wrap(name.to_string(), font_id.clone(), egui::Color32::WHITE);

        // Draw semi-transparent dark background behind text
        let text_rect =
            egui::Rect::from_center_size(text_pos, galley.size() + egui::vec2(12.0, 8.0));
        painter.rect_filled(
            text_rect,
            4.0, // rounded corners
            egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
        );

        // Draw text with black outline for extra contrast
        // Draw black outline by rendering text multiple times with offset
        for dx in [-1.5, 0.0, 1.5].iter() {
            for dy in [-1.5, 0.0, 1.5].iter() {
                if *dx != 0.0 || *dy != 0.0 {
                    painter.text(
                        egui::pos2(text_pos.x + dx, text_pos.y + dy),
                        egui::Align2::CENTER_CENTER,
                        name,
                        font_id.clone(),
                        egui::Color32::BLACK,
                    );
                }
            }
        }

        // Draw main white text on top
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            name,
            font_id,
            egui::Color32::WHITE,
        );
    }

    fn start_spin(&mut self) {
        debug!("Initiating spin sequence");
        self.spinning = true;
        self.show_message = false;
        self.animation_time = 0.0;
        self.winning_item = None;

        // Generate random target rotation using shared wheel logic
        self.target_rotation = self.wheel.generate_spin_rotation(self.rotation);

        debug!(
            "Spin parameters: target_rotation={:.2}",
            self.target_rotation
        );
    }

    fn is_console_app(exe_path: &str) -> bool {
        let exe_lower = exe_path.to_lowercase();
        exe_lower.contains("powershell") 
            || exe_lower.contains("pwsh") 
            || exe_lower.contains("cmd.exe") 
            || exe_lower.ends_with("cmd")
    }

    fn launch_in_new_window(exe: &str) -> std::io::Result<std::process::Child> {
        info!("Detected console application, launching in new window");
        #[cfg(target_os = "linux")]
        let cmd_name = "cmd.exe";
        #[cfg(not(target_os = "linux"))]
        let cmd_name = "cmd";
        
        Command::new(cmd_name)
            .arg("/C")
            .arg("start")
            .arg("") // Empty title
            .arg(exe)
            .spawn()
    }

    fn launch_program(&self, executable: &str) {
        // Launch the program in a separate thread so it doesn't block the UI
        info!("Attempting to launch program: {}", executable);
        let exe = executable.to_string();
        std::thread::spawn(move || {
            let is_console = Self::is_console_app(&exe);
            
            // Check if running on Linux/WSL
            #[cfg(target_os = "linux")]
            {
                debug!("Running on Linux, translating path with wslpath");
                
                let result = if is_console {
                    // Console apps: use original Windows path with cmd.exe
                    Self::launch_in_new_window(&exe)
                } else {
                    // Regular apps: translate path and launch
                    match Command::new("wslpath").arg("-u").arg(&exe).output() {
                        Ok(output) if output.status.success() => {
                            let translated_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                            debug!("Translated path: {} -> {}", exe, translated_path);
                            Command::new(&translated_path).spawn()
                        }
                        Ok(output) => {
                            error!("wslpath failed with status: {}", output.status);
                            error!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                            return;
                        }
                        Err(e) => {
                            error!("Failed to run wslpath: {}", e);
                            return;
                        }
                    }
                };
                
                match result {
                    Ok(_) => info!("Successfully launched: {}", exe),
                    Err(e) => error!("Failed to launch {}: {}", exe, e),
                }
            }

            // For Windows or other platforms
            #[cfg(not(target_os = "linux"))]
            {
                debug!("Launching executable: {}", exe);
                
                let result = if is_console {
                    Self::launch_in_new_window(&exe)
                } else {
                    Command::new(&exe).spawn()
                };
                
                match result {
                    Ok(_) => info!("Successfully launched: {}", exe),
                    Err(e) => error!("Failed to launch {}: {}", exe, e),
                }
            }
        });
    }
}

/// Launch the wheel application with the GUI
pub fn launch() -> Result<(), eframe::Error> {
    info!("Starting Prize Wheel Roulette application");
    debug!("Configuring window options");

    // Force software rendering for cloud dev box / GPU-less environments
    std::env::set_var("WGPU_BACKEND", "gl");
    std::env::set_var("WGPU_POWER_PREF", "low");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 700.0])
            .with_resizable(true),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            supported_backends: wgpu::Backends::all(),
            power_preference: wgpu::PowerPreference::LowPower,
            ..Default::default()
        },
        ..Default::default()
    };

    debug!("Launching native window with wgpu renderer (software fallback enabled)");
    eframe::run_native(
        "Prize Wheel Roulette",
        options,
        Box::new(|_cc| {
            info!("Creating WheelApp instance");
            Ok(Box::new(WheelApp::default()))
        }),
    )
}

/// Launch the wheel application in auto-run mode with transparent background
pub fn launch_auto_run() -> Result<(), eframe::Error> {
    info!("Starting Prize Wheel Roulette application in auto-run mode");
    debug!("Configuring window options for auto-run (transparent)");

    // Force software rendering for cloud dev box / GPU-less environments
    std::env::set_var("WGPU_BACKEND", "gl");
    std::env::set_var("WGPU_POWER_PREF", "low");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 700.0])
            .with_resizable(false)
            .with_transparent(true)
            .with_decorations(false),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            supported_backends: wgpu::Backends::all(),
            power_preference: wgpu::PowerPreference::LowPower,
            ..Default::default()
        },
        ..Default::default()
    };

    debug!("Launching native window in auto-run mode with wgpu renderer (software fallback enabled)");
    eframe::run_native(
        "Prize Wheel Roulette - Auto Run",
        options,
        Box::new(|cc| {
            // Configure transparent visuals
            let mut visuals = egui::Visuals::default();
            visuals.window_fill = egui::Color32::TRANSPARENT;
            visuals.panel_fill = egui::Color32::TRANSPARENT;
            visuals.extreme_bg_color = egui::Color32::TRANSPARENT;
            visuals.faint_bg_color = egui::Color32::TRANSPARENT;
            cc.egui_ctx.set_visuals(visuals);

            info!("Creating WheelApp instance in auto-run mode");
            let mut app = WheelApp::default();
            app.auto_run = true;
            Ok(Box::new(app))
        }),
    )
}
