#![windows_subsystem = "windows"]
use eframe::egui;
use egui::text::{CCursor, CCursorRange};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod formatter;
use formatter::format_code;

const EDITOR_FONT_SIZE: f32 = 15.0;
const GUTTER_RIGHT_PADDING: f32 = 3.0;
const EDITOR_LEFT_PADDING: f32 = 0.0;

fn load_icon() -> Option<egui::IconData> {
    let decode = |bytes: &[u8]| -> Option<egui::IconData> {
        // image crate 会自动根据文件头判断格式
        let img = image::load_from_memory(bytes).ok()?;
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        if w == 0 || h == 0 {
            return None;
        }
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width: w,
            height: h,
        })
    };
    let mut candidates = Vec::new();

    // 当前工作目录
    candidates.push(PathBuf::from("assets/icon.png"));
    candidates.push(PathBuf::from("assets/app.png"));
    candidates.push(PathBuf::from("assets/app.ico"));
    candidates.push(PathBuf::from("icon.png"));

    // exe 所在目录
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("assets/icon.png"));
            candidates.push(dir.join("assets/app.png"));
            candidates.push(dir.join("assets/app.ico"));
            candidates.push(dir.join("icon.png"));
            // exe 在 target/debug 下，assets 在项目根
            candidates.push(dir.join("../assets/icon.png"));
            candidates.push(dir.join("../assets/app.png"));
            candidates.push(dir.join("../assets/app.ico"));
            candidates.push(dir.join("../../assets/icon.png"));
        }
    }

    for p in candidates {
        if let Ok(bytes) = fs::read(&p) {
            if let Some(icon) = decode(&bytes) {
                // println!("icon loaded from {:?}", p); // 调试用
                return Some(icon);
            }
        }
    }
    None
}

fn main() -> eframe::Result<()> {
    #[cfg(windows)]
    {
        let ico_path = "assets/app.ico";
        if std::path::Path::new(ico_path).exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon(ico_path);
            res.set("FileDescription", "Mini Editor");
            res.compile().unwrap();
        }
    }
    let mut options = eframe::NativeOptions::default();
    if let Some(icon) = load_icon() {
        options.viewport = egui::ViewportBuilder::default()
            .with_icon(icon)
            .with_inner_size([900.0, 600.0]);
    }
    eframe::run_native(
        "",
        options,
        Box::new(|cc| {
            let mut visuals = egui::Visuals::light();
            visuals.panel_fill = egui::Color32::from_rgb(255, 255, 255);
            visuals.window_fill = egui::Color32::from_rgb(255, 255, 255);
            visuals.extreme_bg_color = egui::Color32::from_rgb(245, 245, 245);
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(200, 200, 200);
            visuals.widgets.inactive.fg_stroke = egui::Stroke::NONE;
            // 0.36: Rounding -> CornerRadius
            visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(170, 170, 170);
            visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(140, 140, 140);
            visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
            visuals.text_cursor.stroke.color = egui::Color32::BLACK;
            cc.egui_ctx.set_visuals(visuals);

            let mut fonts = egui::FontDefinitions::default();
            if let Some(data) = load_chinese_font() {
                fonts.font_data.insert(
                    "chinese".to_owned(),
                    egui::FontData::from_owned(data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "chinese".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "chinese".to_owned());
            }
            cc.egui_ctx.set_fonts(fonts);

            let mut style = (*cc.egui_ctx.style_of(egui::Theme::Light)).clone();
            style.text_styles.insert(
                egui::TextStyle::Monospace,
                egui::FontId::new(EDITOR_FONT_SIZE, egui::FontFamily::Monospace),
            );
            style.spacing.item_spacing = egui::vec2(0.0, 0.0);
            style.spacing.scroll.bar_width = 8.0;
            cc.egui_ctx.set_style_of(egui::Theme::Light, style);
            Ok(Box::new(MiniEditor::new(cc.egui_ctx.clone())))
        }),
    )
}

fn load_chinese_font() -> Option<Vec<u8>> {
    for path in ["C:/Windows/Fonts/msyh.ttc", "C:/Windows/Fonts/msyh.ttf"] {
        if let Ok(data) = fs::read(path) {
            return Some(data);
        }
    }
    None
}

fn try_load_file(path: &Path) -> Option<String> {
    if let Ok(content) = fs::read_to_string(path) {
        Some(content)
    } else if let Ok(bytes) = fs::read(path) {
        Some(String::from_utf8_lossy(&bytes).to_string())
    } else {
        None
    }
}

fn get_startup_file() -> (String, Option<PathBuf>) {
    if let Some(arg) = std::env::args().nth(1) {
        let path = PathBuf::from(&arg);
        if path.is_file() {
            if let Some(content) = try_load_file(&path) {
                return (content, Some(path));
            }
        }
    }
    ("".to_owned(), None)
}

struct MiniEditor {
    code: String,
    file_path: Option<PathBuf>,
    theme: egui_extras::syntax_highlighting::CodeTheme,
    last_content_height: f32,
    status_msg: Option<(String, f64)>, // (消息, 过期时间戳)
}

impl MiniEditor {
    fn new(ctx: egui::Context) -> Self {
        let (code, file_path) = get_startup_file();
        Self {
            code,
            file_path,
            theme: egui_extras::syntax_highlighting::CodeTheme::from_memory(
                &ctx,
                &ctx.style_of(egui::Theme::Light),
            ),
            last_content_height: 200.0,
            status_msg: None,
        }
    }

    fn load_from_path(&mut self, path: PathBuf) {
        if let Some(content) = try_load_file(&path) {
            self.code = content;
            self.file_path = Some(path);
        }
    }

    fn current_file_name(&self) -> String {
        if let Some(p) = &self.file_path {
            p.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未知文件")
                .to_owned()
        } else {
            "未命名".to_owned()
        }
    }

    fn set_status(&mut self, msg: String) {
        // 显示 3 秒
        let expire = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            + 3.0;
        // 用 egui 的时间会更准，这里先用系统时间占位，实际会在 ui 里用 ctx.time 刷新
        // 所以我们会在调用处用 ctx.input(|i| i.time) + 3.0 覆盖，这里保留接口
        self.status_msg = Some((msg, expire));
    }

    fn set_status_with_time(&mut self, msg: String, now: f64) {
        self.status_msg = Some((msg, now + 3.0));
    }

    fn do_format(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        match format_code(self.file_path.as_deref(), &self.code) {
            Ok(formatted) => {
                if formatted == self.code {
                    self.set_status_with_time("已是格式化状态".to_owned(), now);
                } else {
                    let old_len = self.code.len();
                    self.code = formatted;
                    // 重置光标到开头，避免越界
                    let id = egui::Id::new("code_editor");
                    if let Some(mut state) = egui::TextEdit::load_state(ctx, id) {
                        state
                            .cursor
                            .set_char_range(Some(CCursorRange::one(CCursor::new(0))));
                        state.store(ctx, id);
                    }
                    self.set_status_with_time(
                        format!("格式化完成 ({} -> {} 字节)", old_len, self.code.len()),
                        now,
                    );
                }
            }
            Err(e) => {
                self.set_status_with_time(format!("格式化失败: {}", e), now);
            }
        }
    }
}

impl Default for MiniEditor {
    fn default() -> Self {
        Self::new(egui::Context::default())
    }
}

fn ccursor_to_byte(text: &str, cursor: usize) -> usize {
    text.chars()
        .take(cursor)
        .map(char::len_utf8)
        .sum::<usize>()
        .min(text.len())
}
fn find_line_range(text: &str, byte_pos: usize) -> (usize, usize) {
    let bytes = text.as_bytes();
    let mut start = byte_pos.min(bytes.len());
    while start > 0 && bytes[start - 1] != b'\n' {
        start -= 1;
    }
    let mut end = byte_pos.min(bytes.len());
    while end < bytes.len() && bytes[end] != b'\n' {
        end += 1;
    }
    (start, end)
}
fn line_count(text: &str) -> usize {
    text.split('\n').count().max(1)
}

fn draw_line_numbers(
    ui: &egui::Ui,
    gutter_rect: egui::Rect,
    editor_output: &egui::text_edit::TextEditOutput,
    count: usize,
    font: &egui::FontId,
) {
    let painter = ui.painter_at(gutter_rect);
    let color = egui::Color32::from_rgb(150, 150, 150);
    let galley = Arc::clone(&editor_output.galley);
    let origin = editor_output.galley_pos;
    for (index, row) in galley.rows.iter().take(count).enumerate() {
        let row_center = origin.y + row.rect().center().y;
        if row_center < gutter_rect.min.y - 30.0 || row_center > gutter_rect.max.y + 30.0 {
            continue;
        }
        let number = (index + 1).to_string();
        let pos = egui::pos2(gutter_rect.max.x - GUTTER_RIGHT_PADDING, row_center);
        painter.text(pos, egui::Align2::RIGHT_CENTER, number, font.clone(), color);
    }
}

fn draw_active_line(
    ctx: &egui::Context,
    ui: &egui::Ui,
    gutter_rect: egui::Rect,
    output: &egui::text_edit::TextEditOutput,
) {
    let editor_id = egui::Id::new("code_editor");
    if !ctx.memory(|memory| memory.has_focus(editor_id)) {
        return;
    }
    let Some(state) = egui::TextEdit::load_state(ctx, editor_id) else {
        return;
    };
    let Some(cursor_range) = state.cursor.range(&output.galley) else {
        return;
    };
    let layout_cursor = output.galley.layout_from_cursor(cursor_range.primary);
    let Some(row) = output.galley.rows.get(layout_cursor.row) else {
        return;
    };
    let line_rect = egui::Rect::from_min_max(
        egui::pos2(gutter_rect.min.x, output.galley_pos.y + row.min_y()),
        egui::pos2(
            output.response.rect.max.x,
            output.galley_pos.y + row.max_y(),
        ),
    );
    ui.painter().rect_filled(
        line_rect,
        0.0,
        egui::Color32::from_rgba_unmultiplied(235, 243, 255, 110),
    );
}

// === 0.36 核心改动：App 实现 ===
impl eframe::App for MiniEditor {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let editor_font = egui::FontId::new(EDITOR_FONT_SIZE, egui::FontFamily::Monospace);

        // 拖拽文件
        let dropped = ctx.input(|input| {
            if !input.raw.dropped_files.is_empty() {
                Some(input.raw.dropped_files[0].path().to_path_buf())
            } else {
                None
            }
        });
        // 快捷键状态 - Alt+F 需要同时吃掉按键和文本事件，否则会输入 f
        let (ctrl_s, ctrl_d, alt_f) = ctx.input_mut(|i| {
            let mut alt_f_triggered = false;
            if i.consume_key(egui::Modifiers::ALT, egui::Key::F) {
                alt_f_triggered = true;
            }
            // 关键修复：丢弃 Alt+F 产生的 Text("f") 事件
            if i.modifiers.alt || alt_f_triggered {
                i.events.retain(|ev| {
                    if let egui::Event::Text(t) = ev {
                        if t.eq_ignore_ascii_case("f") {
                            return false;
                        }
                    }
                    true
                });
            }
            let ctrl_s = i.consume_key(egui::Modifiers::CTRL, egui::Key::S);
            let ctrl_d = i.consume_key(egui::Modifiers::CTRL, egui::Key::D);
            (ctrl_s, ctrl_d, alt_f_triggered)
        });

        if let Some(dropped) = dropped {
            self.load_from_path(dropped);
        }

        if ctrl_s {
            if let Some(path) = &self.file_path {
                let _ = fs::write(path, &self.code);
                self.set_status(format!("已保存: {}", path.display()));
            } else {
                self.set_status("未命名文件，无法保存".to_owned());
            }
        }

        if alt_f {
            self.do_format(&ctx);
        }

        // 清理过期消息
        if let Some((_, expire)) = &self.status_msg {
            if ctx.input(|i| i.time) > *expire {
                self.status_msg = None;
            }
        }

        // 中央编辑区 - 0.36 CentralPanel::show 接收 &mut Ui
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(255, 255, 255))
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .id_salt("editor_outer_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                        let count = line_count(&self.code);
                        let max_digits = count.to_string().len().max(2);
                        // 0.36 获取字体宽度方式
                        let char_width = ui.fonts_mut(|fonts| fonts.glyph_width(&editor_font, '0'));
                        let gutter_width =
                            (max_digits as f32 * char_width + GUTTER_RIGHT_PADDING + 8.0).max(44.0);

                        ui.horizontal_top(|ui| {
                            let gutter_height = self.last_content_height.max(20.0);
                            let (gutter_rect, _) = ui.allocate_exact_size(
                                egui::vec2(gutter_width, gutter_height),
                                egui::Sense::hover(),
                            );
                            let line_color = egui::Color32::from_rgb(0xff, 0xff, 0xff);
                            let (divider_rect, _) = ui.allocate_exact_size(
                                egui::vec2(1.0, gutter_height),
                                egui::Sense::hover(),
                            );
                            ui.painter().vline(
                                divider_rect.center().x,
                                divider_rect.y_range(),
                                (1.0, line_color),
                            );

                            let mut editor_output_opt = None;
                            egui::Frame::new()
                                .inner_margin(egui::Margin::symmetric(EDITOR_LEFT_PADDING as i8, 0))
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                                    let mut layouter = {
                                        let theme = self.theme.clone();
                                        move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _wrap_width: f32| {
                                            let mut job =
                                                egui_extras::syntax_highlighting::highlight(
                                                    ui.ctx(),
                                                    &ui.ctx().style_of(egui::Theme::Light),
                                                    &theme,
                                                    text.as_str(),
                                                    "rs",
                                                );
                                            for section in &mut job.sections {
                                                section.format.font_id.size = EDITOR_FONT_SIZE;
                                            }
                                            job.wrap.max_width = f32::INFINITY;
                                            job.wrap.break_anywhere = false;
                                            ui.fonts_mut(|fonts| fonts.layout_job(job))
                                        }
                                    };
                                    let output = egui::TextEdit::multiline(&mut self.code)
                                        .id(egui::Id::new("code_editor"))
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                        .frame(egui::Frame::NONE)
                                        .lock_focus(true)
                                        .layouter(&mut layouter)
                                        .show(ui);
                                    editor_output_opt = Some(output);
                                });

                            if let Some(output) = editor_output_opt.as_ref() {
                                self.last_content_height = output.galley.size().y.max(200.0);
                                let adjusted_gutter = egui::Rect::from_min_size(
                                    egui::pos2(gutter_rect.min.x, output.galley_pos.y),
                                    egui::vec2(gutter_width, self.last_content_height),
                                );
                                draw_active_line(&ctx, ui, adjusted_gutter, output);
                                draw_line_numbers(ui, adjusted_gutter, output, count, &editor_font);
                                if ui.memory(|memory| memory.focused().is_none()) {
                                    output.response.request_focus();
                                }
                                if ctrl_d {
                                    delete_current_line(&ctx, &mut self.code, output);
                                }
                            }
                        });
                    });
            });
    }
}

fn delete_current_line(
    ctx: &egui::Context,
    code: &mut String,
    output: &egui::text_edit::TextEditOutput,
) {
    let id = egui::Id::new("code_editor");
    if let Some(mut state) = egui::TextEdit::load_state(ctx, id) {
        if let Some(range) = state.cursor.range(&output.galley) {
            let byte_index = ccursor_to_byte(code, range.primary.index.into());
            let (start, end) = find_line_range(code, byte_index);
            let is_last_line = end == code.len();
            let removes_previous_newline =
                is_last_line && start > 0 && code.as_bytes()[start - 1] == b'\n';
            let delete_start = if removes_previous_newline {
                start - 1
            } else {
                start
            };
            let mut new_code = String::with_capacity(code.len());
            new_code.push_str(&code[..delete_start]);
            if end < code.len() {
                let next = if code[end..].starts_with('\n') {
                    end + 1
                } else {
                    end
                };
                new_code.push_str(&code[next..]);
            }
            *code = new_code;
            let new_cursor = if removes_previous_newline {
                code.chars().count()
            } else {
                code[..delete_start.min(code.len())].chars().count()
            };
            state
                .cursor
                .set_char_range(Some(CCursorRange::one(CCursor::new(new_cursor))));
            state.store(ctx, id);
        }
    }
}
