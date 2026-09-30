use crate::file_io::{get_startup_file, try_load_file};
use crate::formatter::format_code;
use eframe::egui;
use egui::text::{CCursor, CCursorRange};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub(crate) const EDITOR_FONT_SIZE: f32 = 15.0;
const GUTTER_RIGHT_PADDING: f32 = 3.0;
const EDITOR_LEFT_PADDING: f32 = 0.0;

pub struct MiniEditor {
    code: String,
    saved_code: String,
    file_path: Option<PathBuf>,
    window_title: String,
    theme: egui_extras::syntax_highlighting::CodeTheme,
    last_content_height: f32,
    status_msg: Option<(String, f64)>, // (消息, 过期时间戳)
    search_query: String,
    search_bar_open: bool,
    search_match_index: usize,
    replace_query: String,
    replace_bar_open: bool,
    centered_on_start: bool,
}

impl MiniEditor {
    pub fn new(ctx: egui::Context) -> Self {
        let (code, file_path) = get_startup_file();
        let title = if let Some(path) = &file_path {
            format!("{}", path.display())
        } else {
            "".to_owned()
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
        Self {
            saved_code: code.clone(),
            code,
            file_path,
            window_title: title,
            theme: egui_extras::syntax_highlighting::CodeTheme::from_memory(
                &ctx,
                &ctx.style_of(egui::Theme::Light),
            ),
            last_content_height: 200.0,
            status_msg: None,
            search_query: String::new(),
            search_bar_open: false,
            search_match_index: 0,
            replace_query: String::new(),
            replace_bar_open: false,
            centered_on_start: false,
        }
    }

    fn load_from_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        if let Some(content) = try_load_file(&path) {
            self.code = content;
            self.saved_code = self.code.clone();
            self.file_path = Some(path.clone());
            self.update_window_title(ctx);
        }
    }

    fn update_window_title(&mut self, ctx: &egui::Context) {
        let mut title = self
            .file_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        if self.file_path.is_some() && self.code != self.saved_code {
            title.push_str(" *");
        }
        if title != self.window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.window_title = title;
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
    ctx: &egui::Context,
    ui: &egui::Ui,
    gutter_rect: egui::Rect,
    editor_output: &egui::text_edit::TextEditOutput,
    count: usize,
    font: &egui::FontId,
) {
    let painter = ui.painter_at(gutter_rect);
    let normal_color = egui::Color32::from_rgb(150, 150, 150);
    let galley = Arc::clone(&editor_output.galley);
    let origin = editor_output.galley_pos;
    let active_row = active_editor_row(ctx, editor_output);
    for (index, row) in galley.rows.iter().take(count).enumerate() {
        let row_center = origin.y + row.rect().center().y;
        if row_center < gutter_rect.min.y - 30.0 || row_center > gutter_rect.max.y + 30.0 {
            continue;
        }
        let number = (index + 1).to_string();
        let pos = egui::pos2(gutter_rect.max.x - GUTTER_RIGHT_PADDING, row_center);
        let color = if active_row == Some(index) {
            egui::Color32::from_rgb(203, 79, 209)
        } else {
            normal_color
        };
        painter.text(pos, egui::Align2::RIGHT_CENTER, number, font.clone(), color);
    }
}

fn active_editor_row(
    ctx: &egui::Context,
    output: &egui::text_edit::TextEditOutput,
) -> Option<usize> {
    let editor_id = egui::Id::new("code_editor");
    if !ctx.memory(|memory| memory.has_focus(editor_id)) {
        return None;
    }
    let state = egui::TextEdit::load_state(ctx, editor_id)?;
    let cursor_range = state.cursor.range(&output.galley)?;
    Some(output.galley.layout_from_cursor(cursor_range.primary).row)
}

fn draw_active_line(
    ctx: &egui::Context,
    ui: &egui::Ui,
    gutter_rect: egui::Rect,
    output: &egui::text_edit::TextEditOutput,
) {
    let Some(active_row) = active_editor_row(ctx, output) else {
        return;
    };
    let Some(row) = output.galley.rows.get(active_row) else {
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
        if !self.centered_on_start {
            if let Some(command) = egui::ViewportCommand::center_on_screen(&ctx) {
                ctx.send_viewport_cmd(command);
                self.centered_on_start = true;
            }
        }
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
        let (ctrl_s, ctrl_d, ctrl_f, ctrl_h, ctrl_w, alt_f) = ctx.input_mut(|i| {
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
            let ctrl_f = i.consume_key(egui::Modifiers::CTRL, egui::Key::F);
            let ctrl_h = i.consume_key(egui::Modifiers::CTRL, egui::Key::H);
            let ctrl_w = i.consume_key(egui::Modifiers::CTRL, egui::Key::W);
            (ctrl_s, ctrl_d, ctrl_f, ctrl_h, ctrl_w, alt_f_triggered)
        });

        if ctrl_f {
            self.search_bar_open = true;
            self.replace_bar_open = false;
            ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("find_input")));
        }
        if ctrl_h {
            self.search_bar_open = true;
            self.replace_bar_open = true;
            ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("find_input")));
        }
        if ctrl_w {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        let mut find_action: Option<i32> = None;

        if let Some(dropped) = dropped {
            self.load_from_path(&ctx, dropped);
        }

        if ctrl_s {
            if let Some(path) = &self.file_path {
                match fs::write(path, &self.code) {
                    Ok(()) => {
                        self.saved_code = self.code.clone();
                        self.set_status(format!("已保存: {}", path.display()));
                    }
                    Err(error) => {
                        self.set_status(format!("保存失败: {}", error));
                    }
                }
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
                if self.search_bar_open {
                    let bar = egui::Frame::new()
                        .fill(egui::Color32::from_rgb(247, 248, 250))
                        .inner_margin(egui::Margin::symmetric(10, 5))
                        .show(ui, |ui| {
                            const CONTROL_HEIGHT: f32 = 28.0;
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let response = ui.add_sized(
                                    egui::vec2(250.0, CONTROL_HEIGHT),
                                    egui::TextEdit::singleline(&mut self.search_query)
                                        .id(egui::Id::new("find_input"))
                                        .hint_text("查找"),
                                );
                                if response.changed() {
                                    find_action = Some(0);
                                }
                                if response.has_focus()
                                    && ui.input(|input| input.key_pressed(egui::Key::Enter))
                                {
                                    find_action = Some(1);
                                }

                                let matches = find_matches(&self.code, &self.search_query);
                                if matches.is_empty() {
                                    self.search_match_index = 0;
                                } else {
                                    self.search_match_index %= matches.len();
                                }
                                if ui
                                    .add_enabled(
                                        !matches.is_empty(),
                                        egui::Button::new("‹")
                                            .min_size(egui::vec2(28.0, CONTROL_HEIGHT))
                                            .fill(egui::Color32::TRANSPARENT),
                                    )
                                    .on_hover_text("上一个匹配项")
                                    .clicked()
                                {
                                    find_action = Some(-1);
                                    ctx.memory_mut(|memory| {
                                        memory.request_focus(egui::Id::new("find_input"))
                                    });
                                }
                                if ui
                                    .add_enabled(
                                        !matches.is_empty(),
                                        egui::Button::new("›")
                                            .min_size(egui::vec2(28.0, CONTROL_HEIGHT))
                                            .fill(egui::Color32::TRANSPARENT),
                                    )
                                    .on_hover_text("下一个匹配项")
                                    .clicked()
                                {
                                    find_action = Some(1);
                                    ctx.memory_mut(|memory| {
                                        memory.request_focus(egui::Id::new("find_input"))
                                    });
                                }

                                let match_label = if matches.is_empty() {
                                    "0 / 0".to_owned()
                                } else {
                                    format!("{} / {}", self.search_match_index + 1, matches.len())
                                };
                                let badge_width = (match_label.chars().count() as f32 * 8.0 + 16.0)
                                    .max(52.0);
                                ui.add_sized(
                                    egui::vec2(badge_width, CONTROL_HEIGHT),
                                    egui::Button::new(egui::RichText::new(match_label).size(12.0))
                                        .fill(egui::Color32::WHITE)
                                        .stroke(egui::Stroke::new(
                                            1.0,
                                            egui::Color32::from_rgb(225, 228, 232),
                                        ))
                                        .sense(egui::Sense::hover()),
                                );

                                if ui
                                    .add(
                                        egui::Button::new(egui::RichText::new("×").size(18.0))
                                            .min_size(egui::vec2(28.0, CONTROL_HEIGHT)),
                                    )
                                    .on_hover_text("关闭查找")
                                    .clicked()
                                    || ui.input(|input| input.key_pressed(egui::Key::Escape))
                                {
                                    self.search_bar_open = false;
                                    ctx.memory_mut(|memory| {
                                        memory.request_focus(egui::Id::new("code_editor"))
                                    });
                                }
                            });

                            if self.replace_bar_open {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.add_sized(
                                        egui::vec2(250.0, CONTROL_HEIGHT),
                                        egui::TextEdit::singleline(&mut self.replace_query)
                                            .id(egui::Id::new("replace_input"))
                                            .hint_text("替换为"),
                                    );
                                    let matches = find_matches(&self.code, &self.search_query);
                                    if ui
                                        .add_enabled(
                                            !matches.is_empty(),
                                            egui::Button::new("替换")
                                                .min_size(egui::vec2(64.0, CONTROL_HEIGHT)),
                                        )
                                        .clicked()
                                    {
                                        let match_index =
                                            self.search_match_index.min(matches.len() - 1);
                                        let (start, end) = matches[match_index];
                                        self.code.replace_range(start..end, &self.replace_query);
                                        let remaining =
                                            find_matches(&self.code, &self.search_query);
                                        if remaining.is_empty() {
                                            self.search_match_index = 0;
                                        } else {
                                            self.search_match_index = (match_index
                                                + remaining.len()
                                                - 1)
                                                % remaining.len();
                                            find_action = Some(1);
                                        }
                                        ctx.memory_mut(|memory| {
                                            memory.request_focus(egui::Id::new("replace_input"))
                                        });
                                    }
                                    if ui
                                        .add_enabled(
                                            !matches.is_empty(),
                                            egui::Button::new("全部替换")
                                                .min_size(egui::vec2(78.0, CONTROL_HEIGHT)),
                                        )
                                        .clicked()
                                    {
                                        let count = matches.len();
                                        self.code = self
                                            .code
                                            .replace(&self.search_query, &self.replace_query);
                                        self.search_match_index = 0;
                                        find_action = Some(0);
                                        self.set_status_with_time(
                                            format!("已替换 {} 处", count),
                                            ctx.input(|input| input.time),
                                        );
                                        ctx.memory_mut(|memory| {
                                            memory.request_focus(egui::Id::new("replace_input"))
                                        });
                                    }
                                });
                            }
                        });
                    ui.painter().hline(
                        bar.response.rect.x_range(),
                        bar.response.rect.bottom(),
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(230, 232, 235)),
                    );
                }

                let matches = find_matches(&self.code, &self.search_query);
                if matches.is_empty() {
                    self.search_match_index = 0;
                } else {
                    self.search_match_index %= matches.len();
                }
                let find_target = if let Some(action) = find_action.filter(|_| !matches.is_empty()) {
                    if action == 0 {
                        self.search_match_index = 0;
                    } else if action < 0 {
                        self.search_match_index = if self.search_match_index == 0 {
                            matches.len() - 1
                        } else {
                            self.search_match_index - 1
                        };
                    } else {
                        self.search_match_index = (self.search_match_index + 1) % matches.len();
                    }
                    Some(matches[self.search_match_index])
                } else {
                    None
                };

                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(
                        egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                    )
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
                                draw_line_numbers(
                                    &ctx,
                                    ui,
                                    adjusted_gutter,
                                    output,
                                    count,
                                    &editor_font,
                                );
                                if let Some((start, end)) = find_target {
                                    let start_char = self.code[..start].chars().count();
                                    let end_char = self.code[..end].chars().count();
                                    let editor_id = egui::Id::new("code_editor");
                                    if let Some(mut state) = egui::TextEdit::load_state(&ctx, editor_id) {
                                        state.cursor.set_char_range(Some(CCursorRange::two(
                                            CCursor::new(start_char),
                                            CCursor::new(end_char),
                                        )));
                                        state.store(&ctx, editor_id);
                                        output.response.request_focus();
                                        if let Some(row) = output
                                            .galley
                                            .rows
                                            .get(output.galley.layout_from_cursor(CCursor::new(start_char)).row)
                                        {
                                            let row_rect = row.rect().translate(output.galley_pos.to_vec2());
                                            ui.scroll_to_rect(row_rect, Some(egui::Align::Center));
                                        }
                                    }
                                }
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
        self.update_window_title(&ctx);
    }
}

fn find_matches(text: &str, query: &str) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return Vec::new();
    }
    text.match_indices(query)
        .map(|(start, matched)| (start, start + matched.len()))
        .collect()
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
