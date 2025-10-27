use log::debug;
use rand::Rng;
use std::f32::consts::PI;
use eframe::egui;

/// Represents a single item on the wheel
#[derive(Clone, Debug)]
pub struct WheelItem {
    pub name: String,
    pub executable: String,
    pub color: egui::Color32,
}

impl WheelItem {
    /// Creates a new WheelItem with an auto-generated color based on its index.
    /// Colors are distributed evenly around the HSL color wheel for maximum visual distinction.
    pub fn new(name: impl Into<String>, executable: impl Into<String>, index: usize, total: usize) -> Self {
        Self {
            name: name.into(),
            executable: executable.into(),
            color: Self::generate_color(index, total),
        }
    }

    /// Generates a color by distributing hues evenly around the HSL color wheel.
    /// This ensures vibrant, visually distinct colors that won't clash.
    fn generate_color(index: usize, total: usize) -> egui::Color32 {
        let hue = (index as f32 / total as f32) * 360.0;
        let saturation = 0.70;
        let lightness = 0.60;
        Self::hsl_to_rgb(hue, saturation, lightness)
    }

    /// Converts HSL color values to RGB for egui
    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> egui::Color32 {
        let (r, g, b) = Self::hsl_to_rgb_tuple(h, s, l);
        egui::Color32::from_rgb(r, g, b)
    }

    /// Converts HSL color values to RGB tuple
    fn hsl_to_rgb_tuple(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
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
        
        (r, g, b)
    }
}

/// Shared wheel logic for selecting items
pub struct Wheel {
    items: Vec<WheelItem>,
}

impl Wheel {
    /// Create a new wheel with the default PowerToys items
    pub fn new() -> Self {
        Self::with_items(Self::default_items())
    }

    /// Create a new wheel with custom items
    pub fn with_items(items: Vec<WheelItem>) -> Self {
        Self { items }
    }

    /// Get the default set of wheel items
    pub fn default_items() -> Vec<WheelItem> {
        let item_data = vec![
            ("Notepad", "notepad.exe"),
            ("Calculator", "calc.exe"),
            ("Paint", "mspaint.exe"),
            ("Explorer", "explorer.exe"),
            ("Command", "cmd.exe"),
            ("Task Mgr", "taskmgr.exe"),
            ("Snipping", "SnippingTool.exe"),
            ("Control", "control.exe"),
            ("PowerShell", "powershell.exe"),
            ("Character Map", "charmap.exe"),
            ("Workspaces Editor", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\PowerToys.WorkspacesEditor.exe"),
            ("PowerToys Settings", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\PowerToys.exe"),
            ("Environment Variables", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\WinUI3Apps\\PowerToys.EnvironmentVariables.exe"),
            ("Registry Preview", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\WinUI3Apps\\PowerToys.RegistryPreview.exe"),
            ("Power Rename", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\WinUI3Apps\\PowerToys.PowerRename.exe"),
            ("File Locksmith", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\WinUI3Apps\\PowerToys.FileLocksmithUI.exe"),
            ("Styles Report Tool", "C:\\Users\\crloewen\\AppData\\Local\\PowerToys\\Tools\\PowerToys.StylesReportTool.exe"),
        ];

        let total = item_data.len();
        item_data
            .into_iter()
            .enumerate()
            .map(|(i, (name, exe))| {
                WheelItem::new(name, exe, i, total)
            })
            .collect()
    }

    /// Get a reference to all items
    pub fn items(&self) -> &[WheelItem] {
        &self.items
    }

    /// Spin the wheel and return the winning item
    pub fn spin(&self) -> &WheelItem {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.items.len());
        debug!("Wheel spin: selected index {} of {} items", index, self.items.len());
        &self.items[index]
    }

    /// Calculate which segment would be at the pointer given a rotation angle
    pub fn get_segment_at_rotation(&self, rotation: f32) -> usize {
        let num_segments = self.items.len();
        let angle_per_segment = 2.0 * PI / num_segments as f32;

        // Normalize rotation to 0..2*PI
        let normalized_rotation = rotation % (2.0 * PI);
        let positive_rotation = if normalized_rotation < 0.0 {
            normalized_rotation + 2.0 * PI
        } else {
            normalized_rotation
        };

        // Calculate which segment index is at the top
        let pointer_angle = 0.0;
        let relative_angle = (pointer_angle - positive_rotation) % (2.0 * PI);
        let positive_relative = if relative_angle < 0.0 {
            relative_angle + 2.0 * PI
        } else {
            relative_angle
        };

        let segment_index = (positive_relative / angle_per_segment) as usize % num_segments;
        segment_index
    }

    /// Generate a random target rotation for animation (5-7 full spins plus random position)
    pub fn generate_spin_rotation(&self, current_rotation: f32) -> f32 {
        let mut rng = rand::thread_rng();
        let base_spins = rng.gen_range(5.0..7.0);
        let random_angle = rng.gen_range(0.0..(2.0 * PI));
        current_rotation + (base_spins * 2.0 * PI) + random_angle
    }
}

impl Default for Wheel {
    fn default() -> Self {
        Self::new()
    }
}
