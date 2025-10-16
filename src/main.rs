use eframe::egui;
use log::{debug, info, warn, error};
use rand::Rng;
use std::f32::consts::PI;
use std::process::Command;

fn main() -> Result<(), eframe::Error> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
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

// Template structure for wheel items - easily swap out with different programs!
struct WheelItem {
    name: String,
    executable: String,
    color: egui::Color32,
}

struct WheelApp {
    // Wheel configuration
    items: Vec<WheelItem>,
    
    // Animation state
    rotation: f32,
    spinning: bool,
    spin_speed: f32,
    target_rotation: f32,
    animation_time: f32,
    
    // Result state
    winning_item: Option<String>,
    show_message: bool,
}

impl Default for WheelApp {
    fn default() -> Self {
        // TEMPLATE: Easily swap out these items with your own programs!
        let items = vec![
            WheelItem {
                name: "Notepad".to_string(),
                executable: "notepad.exe".to_string(),
                color: egui::Color32::from_rgb(255, 82, 82),
            },
            WheelItem {
                name: "Calculator".to_string(),
                executable: "calc.exe".to_string(),
                color: egui::Color32::from_rgb(255, 177, 66),
            },
            WheelItem {
                name: "Paint".to_string(),
                executable: "mspaint.exe".to_string(),
                color: egui::Color32::from_rgb(255, 235, 59),
            },
            WheelItem {
                name: "Explorer".to_string(),
                executable: "explorer.exe".to_string(),
                color: egui::Color32::from_rgb(102, 187, 106),
            },
            WheelItem {
                name: "Command".to_string(),
                executable: "cmd.exe".to_string(),
                color: egui::Color32::from_rgb(66, 165, 245),
            },
            WheelItem {
                name: "Task Mgr".to_string(),
                executable: "taskmgr.exe".to_string(),
                color: egui::Color32::from_rgb(171, 71, 188),
            },
            WheelItem {
                name: "Settings".to_string(),
                executable: "ms-settings:".to_string(),
                color: egui::Color32::from_rgb(236, 64, 122),
            },
            WheelItem {
                name: "Snipping".to_string(),
                executable: "SnippingTool.exe".to_string(),
                color: egui::Color32::from_rgb(255, 138, 101),
            },
            WheelItem {
                name: "Control".to_string(),
                executable: "control.exe".to_string(),
                color: egui::Color32::from_rgb(178, 223, 138),
            },
            WheelItem {
                name: "PowerShell".to_string(),
                executable: "powershell.exe".to_string(),
                color: egui::Color32::from_rgb(128, 203, 196),
            },
            WheelItem {
                name: "Wordpad".to_string(),
                executable: "write.exe".to_string(),
                color: egui::Color32::from_rgb(149, 117, 205),
            },
            WheelItem {
                name: "Character Map".to_string(),
                executable: "charmap.exe".to_string(),
                color: egui::Color32::from_rgb(255, 204, 128),
            },
        ];
        
        info!("Initializing WheelApp with {} items", items.len());
        for (i, item) in items.iter().enumerate() {
            debug!("Item {}: {} -> {}", i, item.name, item.executable);
        }
        
        Self {
            items,
            rotation: 0.0,
            spinning: false,
            spin_speed: 0.0,
            target_rotation: 0.0,
            animation_time: 0.0,
            winning_item: None,
            show_message: false,
        }
    }
}

impl eframe::App for WheelApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                info!("Winner: {} ({})", winning_item.name, winning_item.executable);
                self.winning_item = Some(winning_item.name.clone());
                self.show_message = true;
                
                // Launch the selected program
                self.launch_program(&winning_item.executable);
            } else {
                // Ease out cubic for smooth deceleration
                let t = self.animation_time / duration;
                let eased = 1.0 - (1.0 - t).powi(3);
                self.rotation = self.target_rotation * eased;
                
                // Request continuous repaint while spinning
                ctx.request_repaint();
            }
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading("🎰 Prize Wheel Roulette 🎰");
                ui.add_space(20.0);
                
                // Draw the wheel
                let wheel_size = 400.0;
                let (response, painter) = ui.allocate_painter(
                    egui::vec2(wheel_size, wheel_size),
                    egui::Sense::hover(),
                );
                
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
                painter.circle_stroke(
                    center,
                    20.0,
                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                );
                
                // Draw pointer at top - pointing DOWN into the wheel
                let pointer_tip_y = center.y - radius + 5.0;  // Just inside the wheel
                let pointer_base_y = center.y - radius - 30.0; // Above the wheel
                let pointer_points = vec![
                    egui::pos2(center.x, pointer_tip_y),           // Tip pointing down
                    egui::pos2(center.x - 15.0, pointer_base_y),   // Left base
                    egui::pos2(center.x + 15.0, pointer_base_y),   // Right base
                ];
                painter.add(egui::Shape::convex_polygon(
                    pointer_points,
                    egui::Color32::from_rgb(255, 50, 50),
                    egui::Stroke::new(3.0, egui::Color32::from_rgb(150, 0, 0)),
                ));
                
                ui.add_space(30.0);
                
                // Spin button
                let button_text = if self.spinning { "Spinning..." } else { "🎲 SPIN 🎲" };
                let button = egui::Button::new(
                    egui::RichText::new(button_text)
                        .size(28.0)
                        .strong()
                );
                
                if ui.add_sized([200.0, 60.0], button).clicked() && !self.spinning {
                    info!("Spin button clicked, starting new spin");
                    self.start_spin();
                }
                
                ui.add_space(20.0);
                
                // Show winning message
                if self.show_message {
                    if let Some(item_name) = &self.winning_item {
                        ui.add_space(10.0);
                        
                        ui.label(
                            egui::RichText::new(format!("🎉 Congratulations! 🎉"))
                                .size(32.0)
                                .color(egui::Color32::from_rgb(50, 200, 50))
                                .strong()
                        );
                        
                        ui.label(
                            egui::RichText::new(format!("Launching: {}", item_name))
                                .size(28.0)
                                .color(egui::Color32::from_rgb(255, 215, 0))
                                .strong()
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
        let galley = painter.layout_no_wrap(name.to_string(), font_id.clone(), egui::Color32::WHITE);
        
        // Draw semi-transparent dark background behind text
        let text_rect = egui::Rect::from_center_size(
            text_pos,
            galley.size() + egui::vec2(12.0, 8.0),
        );
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
        
        debug!("Spin parameters: base_spins={:.2}, random_angle={:.2}, target_rotation={:.2}", 
               base_spins, random_angle, self.target_rotation);
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
            if exe.starts_with("ms-") {
                // For Windows URI schemes (like ms-settings:), use start command
                debug!("Launching Windows URI scheme: {}", exe);
                match Command::new("cmd")
                    .args(["/C", "start", &exe])
                    .spawn() {
                    Ok(_) => info!("Successfully launched: {}", exe),
                    Err(e) => error!("Failed to launch {}: {}", exe, e),
                }
            } else {
                // For regular executables
                debug!("Launching executable: {}", exe);
                match Command::new(&exe).spawn() {
                    Ok(_) => info!("Successfully launched: {}", exe),
                    Err(e) => error!("Failed to launch {}: {}", exe, e),
                }
            }
        });
    }
}
