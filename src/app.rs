use eframe::egui;
use log::{debug, error, info};
use rand::Rng;
use std::f32::consts::PI;
use std::process::Command;

// Template structure for wheel items - easily swap out with different programs!
#[derive(Clone)]
struct WheelItem {
    name: String,
    executable: String,
    color: egui::Color32,
    powertoys_app: bool,
}

impl WheelItem {
    /// Creates a new WheelItem with an auto-generated color based on its index.
    /// Colors are distributed evenly around the HSL color wheel for maximum visual distinction.
    fn new(name: impl Into<String>, executable: impl Into<String>, index: usize, total: usize, powertoys_app: bool) -> Self {
        Self {
            name: name.into(),
            executable: executable.into(),
            color: Self::generate_color(index, total),
            powertoys_app,
        }
    }

    /// Generates a color by distributing hues evenly around the HSL color wheel.
    /// This ensures vibrant, visually distinct colors that won't clash.
    fn generate_color(index: usize, total: usize) -> egui::Color32 {
        // Distribute hues evenly: 0°, 360°/n, 2*360°/n, etc.
        let hue = (index as f32 / total as f32) * 360.0;
        let saturation = 0.70; // 70% saturation for vibrant but not overwhelming colors
        let lightness = 0.60;  // 60% lightness for good contrast with white text
        
        Self::hsl_to_rgb(hue, saturation, lightness)
    }

    /// Converts HSL color values to RGB.
    /// H: 0-360 degrees, S: 0.0-1.0, L: 0.0-1.0
    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> egui::Color32 {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
        
        let (r1, g1, b1) = match h_prime as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            5 => (c, 0.0, x),
            _ => (c, x, 0.0),
        };
        
        let m = l - c / 2.0;
        let r = ((r1 + m) * 255.0) as u8;
        let g = ((g1 + m) * 255.0) as u8;
        let b = ((b1 + m) * 255.0) as u8;
        
        egui::Color32::from_rgb(r, g, b)
    }
}

pub struct WheelApp {
    // Wheel configuration
    items: Vec<WheelItem>,

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
        // TEMPLATE: Easily swap out these items with your own programs!
        // Colors are now auto-generated based on HSL color distribution for maximum visual distinction.
        // Simply add (name, executable) pairs - colors will be automatically assigned!
        let item_data = vec![
            ("Notepad", "notepad.exe", false),
            ("Calculator", "calc.exe", false),
            ("Paint", "mspaint.exe", false),
            ("Explorer", "explorer.exe", false),
            ("Command", "cmd.exe", false),
            ("Task Mgr", "taskmgr.exe", false),
            ("Snipping", "SnippingTool.exe", false),
            ("Control", "control.exe", false),
            ("PowerShell", "powershell.exe", false),
            ("Character Map", "charmap.exe", false),
            ("Workspaces Editor", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\PowerToys.WorkspacesEditor.exe", true),
            ("PowerToys Settings", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\PowerToys.exe", true),
            ("Environment Variables", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\WinUI3Apps\\PowerToys.EnvironmentVariables.exe", true),
            ("Registry Preview", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\WinUI3Apps\\PowerToys.RegistryPreview.exe", true),
            ("Power Rename", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\WinUI3Apps\\PowerToys.PowerRename.exe", true),
            ("File Locksmith", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\WinUI3Apps\\PowerToys.FileLocksmithUI.exe", true),
            ("Styles Report Tool", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\Tools\\PowerToys.StylesReportTool.exe", true),
        ];

        let total = item_data.len();
        let items: Vec<WheelItem> = item_data
            .into_iter()
            .enumerate()
            .map(|(i, (name, exe, is_powertoys))| {
                WheelItem::new(name, exe, i, total, is_powertoys)
            })
            .collect();

        info!("Initializing WheelApp with {} items", items.len());
        for (i, item) in items.iter().enumerate() {
            debug!("Item {}: {} -> {}", i, item.name, item.executable);
        }

        Self {
            items,
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
            egui::Rgba::from_rgb(0.1, 0.1, 0.1).to_array() // Default dark background
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
                let segment_index = self.get_segment_at_pointer();
                debug!("Calculated winning segment index: {}", segment_index);
                let winning_item = &self.items[segment_index];
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
                let num_segments = self.items.len();
                let angle_per_segment = 2.0 * PI / num_segments as f32;

                for i in 0..num_segments {
                    let start_angle = i as f32 * angle_per_segment + self.rotation - PI / 2.0;
                    let end_angle = start_angle + angle_per_segment;

                    // Draw segment
                    self.draw_segment(
                        &painter,
                        center,
                        radius,
                        start_angle,
                        end_angle,
                        self.items[i].color,
                        &self.items[i].name,
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

        // Generate random target rotation (5-7 full spins plus random position)
        let mut rng = rand::thread_rng();
        let base_spins = rng.gen_range(5.0..7.0);
        let random_angle = rng.gen_range(0.0..(2.0 * PI));
        self.target_rotation = self.rotation + (base_spins * 2.0 * PI) + random_angle;

        debug!(
            "Spin parameters: base_spins={:.2}, random_angle={:.2}, target_rotation={:.2}",
            base_spins, random_angle, self.target_rotation
        );
    }

    fn get_segment_at_pointer(&self) -> usize {
        // We need to find which segment is at that position
        let num_segments = self.items.len();
        let angle_per_segment = 2.0 * PI / num_segments as f32;

        // Normalize rotation to 0..2*PI
        let normalized_rotation = self.rotation % (2.0 * PI);
        let positive_rotation = if normalized_rotation < 0.0 {
            normalized_rotation + 2.0 * PI
        } else {
            normalized_rotation
        };

        // Calculate which segment index is at the top
        // We need to account for the -PI/2 offset and reverse direction
        let pointer_angle = 0.0; // Top position in our coordinate system
        let relative_angle = (pointer_angle - positive_rotation) % (2.0 * PI);
        let positive_relative = if relative_angle < 0.0 {
            relative_angle + 2.0 * PI
        } else {
            relative_angle
        };

        let segment_index = (positive_relative / angle_per_segment) as usize % num_segments;
        segment_index
    }

    fn launch_program(&self, executable: &str) {
        // Launch the program in a separate thread so it doesn't block the UI
        info!("Attempting to launch program: {}", executable);
        let exe = executable.to_string();
        std::thread::spawn(move || {
            // For regular executables
            debug!("Launching executable: {}", exe);
            match Command::new(&exe).spawn() {
                Ok(_) => info!("Successfully launched: {}", exe),
                Err(e) => error!("Failed to launch {}: {}", exe, e),
            }
        });
    }
}

/// Launch the wheel application with the GUI
pub fn launch() -> Result<(), eframe::Error> {
    info!("Starting Prize Wheel Roulette application");
    debug!("Configuring window options");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 700.0])
            .with_resizable(true),
        ..Default::default()
    };

    debug!("Launching native window");
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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 700.0])
            .with_resizable(false)
            .with_transparent(true)
            .with_decorations(false),
        ..Default::default()
    };

    debug!("Launching native window in auto-run mode");
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
