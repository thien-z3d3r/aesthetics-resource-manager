use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2, FontId, RichText, Sense, CornerRadius, ViewportCommand, Id};
use egui_plot::{Line, Plot, PlotPoints};
use sysinfo::{Components, System};
use std::collections::VecDeque;
use std::process::Command;
use std::time::{Duration, Instant};
use std::f32::consts::PI;

#[derive(Clone, PartialEq)]
struct Theme {
    name: String,
    bg: Color32,
    accent1: Color32,
    accent2: Color32,
    accent3: Color32,
    border: Color32,
    text_main: Color32,
    text_dim: Color32,
}

impl Theme {
    fn presets() -> Vec<Theme> {
        vec![
            Theme {
                name: "Cyberpunk".to_string(),
                bg: Color32::from_rgba_premultiplied(10, 12, 20, 220),
                accent1: Color32::from_rgb(255, 0, 110),
                accent2: Color32::from_rgb(0, 246, 255),
                accent3: Color32::from_rgb(255, 214, 10),
                border: Color32::from_rgb(60, 60, 80),
                text_main: Color32::from_rgb(230, 230, 240),
                text_dim: Color32::from_rgb(120, 130, 150),
            },
            Theme {
                name: "Catppuccin Mocha".to_string(),
                bg: Color32::from_rgba_premultiplied(30, 30, 46, 220),
                accent1: Color32::from_rgb(243, 139, 168), // Red
                accent2: Color32::from_rgb(137, 180, 250), // Blue
                accent3: Color32::from_rgb(249, 226, 175), // Yellow
                border: Color32::from_rgb(49, 50, 68),
                text_main: Color32::from_rgb(205, 214, 244),
                text_dim: Color32::from_rgb(166, 173, 200),
            },
            Theme {
                name: "Tokyo Night".to_string(),
                bg: Color32::from_rgba_premultiplied(26, 27, 38, 220),
                accent1: Color32::from_rgb(247, 118, 142),
                accent2: Color32::from_rgb(122, 162, 247),
                accent3: Color32::from_rgb(224, 175, 104),
                border: Color32::from_rgb(65, 72, 104),
                text_main: Color32::from_rgb(192, 202, 245),
                text_dim: Color32::from_rgb(86, 95, 137),
            },
            Theme {
                name: "Dark Traditional".to_string(),
                bg: Color32::from_rgba_premultiplied(18, 18, 18, 220),
                accent1: Color32::from_rgb(187, 134, 252),
                accent2: Color32::from_rgb(3, 218, 198),
                accent3: Color32::from_rgb(207, 102, 121),
                border: Color32::from_rgb(51, 51, 51),
                text_main: Color32::from_rgb(255, 255, 255),
                text_dim: Color32::from_rgb(176, 176, 176),
            },
            Theme {
                name: "Light Traditional".to_string(),
                bg: Color32::from_rgba_premultiplied(255, 255, 255, 220),
                accent1: Color32::from_rgb(98, 0, 238),
                accent2: Color32::from_rgb(3, 218, 198),
                accent3: Color32::from_rgb(176, 0, 32),
                border: Color32::from_rgb(224, 224, 224),
                text_main: Color32::from_rgb(0, 0, 0),
                text_dim: Color32::from_rgb(96, 96, 96),
            },
        ]
    }
}

struct GpuData {
    usage: f32,
    memory_usage: f32,
    temp: i32,
    vram_used: u64,
    vram_total: u64,
}

impl Default for GpuData {
    fn default() -> Self {
        Self { usage: 0.0, memory_usage: 0.0, temp: 0, vram_used: 0, vram_total: 0 }
    }
}

struct ZenMonitor {
    system: System,
    components: Components,
    gpu_data: GpuData,
    
    // Data Buffers
    cpu_hist: VecDeque<f64>,
    ram_hist: VecDeque<f64>,
    gpu_hist: VecDeque<f64>,
    time_pts: VecDeque<f64>,
    
    start_time: Instant,
    last_update: Instant,
    gpu_timer: Instant,
    
    // Theme
    themes: Vec<Theme>,
    current_theme_index: usize,
}

impl ZenMonitor {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.window_fill = Color32::TRANSPARENT; // Let our panel handle bg
        style.visuals.panel_fill = Color32::TRANSPARENT;
        cc.egui_ctx.set_style(style);

        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system,
            components: Components::new_with_refreshed_list(),
            gpu_data: GpuData::default(),
            cpu_hist: VecDeque::from(vec![0.0; 120]),
            ram_hist: VecDeque::from(vec![0.0; 120]),
            gpu_hist: VecDeque::from(vec![0.0; 120]),
            time_pts: VecDeque::from(vec![0.0; 120]),
            start_time: Instant::now(),
            last_update: Instant::now(),
            gpu_timer: Instant::now() - Duration::from_secs(5),
            themes: Theme::presets(),
            current_theme_index: 0,
        }
    }

    fn refresh_gpu(&mut self) {
        if let Ok(output) = Command::new("nvidia-smi")
            .args(&["--query-gpu=utilization.gpu,temperature.gpu,memory.total,memory.used","--format=csv,noheader,nounits"])
            .output() 
        {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                let p: Vec<&str> = stdout.trim().split(',').collect();
                if p.len() >= 4 {
                    self.gpu_data.usage = p[0].trim().parse().unwrap_or(0.0);
                    self.gpu_data.temp = p[1].trim().parse().unwrap_or(0);
                    let total: f32 = p[2].trim().parse().unwrap_or(1.0);
                    let used: f32 = p[3].trim().parse().unwrap_or(0.0);
                    self.gpu_data.vram_total = total as u64;
                    self.gpu_data.vram_used = used as u64;
                    self.gpu_data.memory_usage = (used / total) * 100.0;
                }
            }
        }
    }

    fn get_cpu_temp(&self) -> f32 {
        for c in &self.components {
            let l = c.label().to_lowercase();
            if l.contains("tctl") || l.contains("package") { return c.temperature().unwrap_or(0.0); }
        }
        self.components.iter().find_map(|c| c.temperature()).unwrap_or(0.0)
    }
}

// --- CUSTOM WIDGETS ---

fn draw_cyber_gauge(ui: &mut egui::Ui, pct: f32, color: Color32, label: &str, val: &str, theme: &Theme) {
    let size = Vec2::new(100.0, 120.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter();
    let center = rect.center() - Vec2::new(0.0, 10.0);
    let radius = 38.0;

    // Background Ring (Dim)
    let ring_color = if theme.name.contains("Light") {
        Color32::from_black_alpha(20)
    } else {
        Color32::from_white_alpha(10)
    };
    p.circle_stroke(center, radius, Stroke::new(4.0, ring_color));
    
    // Value Arc
    let start = PI * 0.75;
    let sweep = PI * 1.5;
    let end = start + sweep * (pct / 100.0).clamp(0.0, 1.0);
    
    // Neon Glow Effect
    let steps = 30;
    let pts: Vec<Pos2> = (0..=steps).map(|i| {
        let a = start + (end - start) * (i as f32 / steps as f32);
        center + Vec2::new(a.cos() * radius, a.sin() * radius)
    }).collect();
    
    if pts.len() > 1 {
        // Glow
        if !theme.name.contains("Light") {
             p.add(egui::Shape::line(pts.clone(), Stroke::new(12.0, color.gamma_multiply(0.2)))); 
        }
        // Core
        p.add(egui::Shape::line(pts, Stroke::new(3.0, color)));
    }

    // Text
    p.text(center, egui::Align2::CENTER_CENTER, format!("{:.0}%", pct), FontId::proportional(18.0), theme.text_main);
    p.text(center + Vec2::new(0.0, radius + 20.0), egui::Align2::CENTER_CENTER, label, FontId::monospace(14.0), color);
    p.text(center + Vec2::new(0.0, radius + 36.0), egui::Align2::CENTER_CENTER, val, FontId::proportional(11.0), theme.text_dim);
}

fn custom_window_frame(
    ctx: &egui::Context, 
    _frame: &mut eframe::Frame, 
    title: &str, 
    theme: &Theme,
    themes: &[Theme],
    current_theme_index: &mut usize,
    content: impl FnOnce(&mut egui::Ui)
) {
    let text_color = theme.text_main;
    
    // Updated Frame struct for egui 0.33+
    let panel_frame = egui::Frame {
        fill: theme.bg,
        corner_radius: CornerRadius::same(12),
        stroke: Stroke::new(1.0, theme.border),
        inner_margin: egui::Margin::same(15),
        outer_margin: egui::Margin::same(0),
        ..Default::default()
    };

    egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
        // --- Custom Title Bar ---
        let app_rect = ui.max_rect();
        let title_bar_height = 32.0;
        let title_bar_rect = Rect::from_min_size(app_rect.min, Vec2::new(app_rect.width(), title_bar_height));
        
        // Window Dragging
        let title_bar_response = ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click_and_drag());
        if title_bar_response.dragged() {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        // Paint Title Bar
        let painter = ui.painter();
        let divider_color = if theme.name.contains("Light") {
            Color32::from_black_alpha(20)
        } else {
            Color32::from_white_alpha(20)
        };
        
        painter.line_segment(
            [Pos2::new(app_rect.min.x + 10.0, app_rect.min.y + title_bar_height), 
             Pos2::new(app_rect.max.x - 10.0, app_rect.min.y + title_bar_height)],
            Stroke::new(1.0, divider_color)
        );
        
        // Title
        painter.text(
            Pos2::new(title_bar_rect.min.x + 15.0, title_bar_rect.center().y),
            egui::Align2::LEFT_CENTER,
            title,
            FontId::proportional(14.0),
            text_color,
        );
        
        // Theme Switcher & Close Button Container
        let controls_rect = Rect::from_min_max(
            Pos2::new(title_bar_rect.max.x - 150.0, title_bar_rect.min.y),
            Pos2::new(title_bar_rect.max.x, title_bar_rect.max.y)
        );
        
        ui.allocate_ui_at_rect(controls_rect, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                
                // Close Button
                if ui.button(RichText::new(" X ").color(Color32::from_rgb(200, 50, 50))).clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
                
                ui.add_space(5.0);
        
                // Theme ComboBox
                egui::ComboBox::from_id_salt("theme_selector")
                    .selected_text(RichText::new(&themes[*current_theme_index].name).size(12.0).color(text_color))
                    .width(100.0)
                    .show_ui(ui, |ui| {
                        for (i, t) in themes.iter().enumerate() {
                            ui.selectable_value(current_theme_index, i, &t.name);
                        }
                    });
            });
        });

        ui.add_space(title_bar_height);
        content(ui);
    });
}

impl eframe::App for ZenMonitor {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Logic
        let now = Instant::now();
        if now.duration_since(self.last_update) > Duration::from_millis(500) {
            self.system.refresh_cpu_usage();
            self.system.refresh_memory();
            self.components.refresh(true);
            
            self.cpu_hist.push_back(self.system.global_cpu_usage() as f64);
            self.ram_hist.push_back((self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0);
            self.gpu_hist.push_back(self.gpu_data.usage as f64);
            self.time_pts.push_back(now.duration_since(self.start_time).as_secs_f64());
            
            if self.cpu_hist.len() > 120 {
                self.cpu_hist.pop_front(); self.ram_hist.pop_front(); 
                self.gpu_hist.pop_front(); self.time_pts.pop_front();
            }
            self.last_update = now;
        }
        if now.duration_since(self.gpu_timer) > Duration::from_secs(2) {
            self.refresh_gpu();
            self.gpu_timer = now;
        }

        let current_theme = &self.themes[self.current_theme_index];
        let mut selected_theme_index = self.current_theme_index;

        // Draw
        custom_window_frame(
            ctx, 
            frame, 
            "ZEN MONITOR // SYSTEM_HUD", 
            current_theme,
            &self.themes,
            &mut selected_theme_index,
            |ui| {
             ui.add_space(5.0);
             
             // --- Gauges Container ---
             let bg_fill = if current_theme.name.contains("Light") {
                 Color32::from_black_alpha(10)
             } else {
                 Color32::from_black_alpha(40)
             };
             
             let border_stroke = if current_theme.name.contains("Light") {
                 Color32::from_black_alpha(10)
             } else {
                 Color32::from_white_alpha(10)
             };

             egui::Frame::group(ui.style())
                .fill(bg_fill)
                .stroke(Stroke::new(1.0, border_stroke))
                .corner_radius(CornerRadius::same(8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.columns(3, |cols| {
                            draw_cyber_gauge(&mut cols[0], self.system.global_cpu_usage(), current_theme.accent1, "CPU", &format!("{:.0}°C", self.get_cpu_temp()), current_theme);
                            draw_cyber_gauge(&mut cols[1], (self.system.used_memory() as f32 / self.system.total_memory() as f32) * 100.0, current_theme.accent2, "RAM", &format!("{:.1}GB", self.system.used_memory() as f32/1e9), current_theme);
                            draw_cyber_gauge(&mut cols[2], self.gpu_data.usage, current_theme.accent3, "GPU", &format!("{:.0}°C", self.gpu_data.temp), current_theme);
                        });
                    });
                    ui.add_space(5.0);
                });

             ui.add_space(15.0);

             // --- Graphs ---
             let plot = Plot::new("cyber_plot")
                .height(120.0)
                .show_axes([false, true])
                .show_grid([false, true])
                .y_axis_min_width(20.0)
                .include_y(0.0).include_y(100.0)
                .show_background(false)
                .legend(egui_plot::Legend::default().background_alpha(0.5).text_style(egui::TextStyle::Small)); // Updated legend

             plot.show(ui, |ui| {
                 ui.line(Line::new("CPU", PlotPoints::from_iter(self.time_pts.iter().zip(self.cpu_hist.iter()).map(|(&t,&v)|[t,v]))).color(current_theme.accent1).width(2.0));
                 ui.line(Line::new("RAM", PlotPoints::from_iter(self.time_pts.iter().zip(self.ram_hist.iter()).map(|(&t,&v)|[t,v]))).color(current_theme.accent2).width(2.0));
                 ui.line(Line::new("GPU", PlotPoints::from_iter(self.time_pts.iter().zip(self.gpu_hist.iter()).map(|(&t,&v)|[t,v]))).color(current_theme.accent3).width(2.0));
             });
             
             ui.add_space(5.0);
             ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                 ui.label(RichText::new("ONLINE").size(10.0).color(Color32::GREEN));
                 ui.label(RichText::new("SYSTEM STATUS:").size(10.0).color(current_theme.text_dim));
             });
        });
        
        self.current_theme_index = selected_theme_index;
        
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 420.0])
            .with_decorations(false) // No OS title bar
            .with_transparent(true)  // Transparent window
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native("Zen Monitor", options, Box::new(|cc| Ok(Box::new(ZenMonitor::new(cc)))))
}