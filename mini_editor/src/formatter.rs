use std::path::Path;

/// 格式化入口：根据文件名后缀决定格式化器
pub fn format_code(path: Option<&Path>, code: &str) -> Result<String, String> {
    let ext = path
        .and_then(|p| p.extension())
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if code.trim().is_empty() {
        return Err("文件内容为空".to_string());
    }

    match ext.as_str() {
        "json" => format_json(code),
        "xml" => format_xml(code),
        "" => {
            if is_likely_json(code) {
                format_json(code)
            } else if is_likely_xml(code) {
                format_xml(code)
            } else {
                // 尝试都格式化一下
                if let Ok(v) = format_json(code) {
                    return Ok(v);
                }
                if let Ok(v) = format_xml(code) {
                    return Ok(v);
                }
                Err("无法判断文件类型：请保存为 .json 或 .xml 后再试。".to_string())
            }
        }
        other => {
            if is_likely_json(code) {
                format_json(code)
            } else if other == "xml" || is_likely_xml(code) {
                format_xml(code)
            } else {
                Err(format!("暂不支持 .{} 文件，目前支持 .json / .xml", other))
            }
        }
    }
}

fn is_likely_json(s: &str) -> bool {
    let t = s.trim();
    (t.starts_with('{') && t.ends_with('}')) || (t.starts_with('[') && t.ends_with(']'))
}
fn is_likely_xml(s: &str) -> bool {
    let t = s.trim();
    t.starts_with('<') && t.ends_with('>')
}

pub fn format_json(input: &str) -> Result<String, String> {
    // 优先用 serde_json，如果没有依赖会编译报错，提示用户加依赖
    // 如果你不想加依赖，把下面注释掉用手写版
    let v: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("JSON 解析失败: {}", e))?;
    serde_json::to_string_pretty(&v).map_err(|e| format!("格式化失败: {}", e))
}

/// XML 格式化 - 修复版： <tag>文本</tag> 保持在同一行
pub fn format_xml(input: &str) -> Result<String, String> {
    let input = input.trim();
    if !input.starts_with('<') {
        return Err("XML 解析失败：不是以 < 开头".to_string());
    }

    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(len * 2);
    let mut indent: usize = 0;
    let mut i = 0;
    let mut stack: Vec<String> = Vec::new();

    while i < len {
        // 跳过空白
        while i < len && chars[i].is_whitespace() && chars[i] != '<' {
            // 空白在标签之间忽略，在文本之间保留由文本分支处理
            // 这里直接跳过标签间的空白
            let mut k = i;
            while k < len && chars[k].is_whitespace() {
                k += 1;
            }
            if k < len && chars[k] == '<' {
                i = k;
                break;
            } else {
                break;
            }
        }
        if i >= len {
            break;
        }

        if chars[i] != '<' {
            // 文本节点 - 理论上不会直接进这里，因为文本由标签分支的 lookahead 处理
            // 但为了健壮还是处理
            let mut j = i;
            while j < len && chars[j] != '<' {
                j += 1;
            }
            let text: String = chars[i..j].iter().collect();
            let t = text.trim();
            if !t.is_empty() {
                out.push_str(&"  ".repeat(indent));
                out.push_str(t);
                out.push('\n');
            }
            i = j;
            continue;
        }

        // 此时 chars[i] == '<'
        // 找标签结束 >
        let mut j = i + 1;
        let mut in_quote = false;
        let mut quote_char = '"';
        let mut is_comment = false;
        if j + 2 < len && chars[j] == '!' && chars[j + 1] == '-' && chars[j + 2] == '-' {
            is_comment = true;
        }
        if is_comment {
            // 找 -->
            while j + 2 < len {
                if chars[j] == '-' && chars[j + 1] == '-' && chars[j + 2] == '>' {
                    j += 2;
                    break;
                }
                j += 1;
            }
        } else {
            while j < len {
                let ch = chars[j];
                if (ch == '"' || ch == '\'') && (j == 0 || chars[j - 1] != '\\') {
                    if !in_quote {
                        in_quote = true;
                        quote_char = ch;
                    } else if ch == quote_char {
                        in_quote = false;
                    }
                }
                if ch == '>' && !in_quote {
                    break;
                }
                j += 1;
            }
        }
        if j >= len {
            return Err("XML 解析失败：标签未闭合，缺少 >".to_string());
        }

        let tag_full: String = chars[i..=j].iter().collect();
        let tag_trim = tag_full.trim().to_string();

        let is_decl = tag_trim.starts_with("<?");
        let is_closing = tag_trim.starts_with("</");
        let is_self_closing = tag_trim.ends_with("/>") || is_decl;

        if is_comment || is_decl {
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&"  ".repeat(indent));
            out.push_str(&tag_trim);
            out.push('\n');
            i = j + 1;
            continue;
        }

        if is_closing {
            if indent > 0 {
                indent -= 1;
            }
            if let Some(top) = stack.last() {
                let name = extract_tag_name(&tag_trim);
                if top == &name {
                    stack.pop();
                }
            }
            // 闭合标签如果前面是行内文本已经在同一行，就不需要再缩进
            if out.ends_with("</") || out.ends_with('>') && !out.ends_with("\n") {
                // 前面是 <tag>文本，已经在同一行，这里直接追加
                out.push_str(&tag_trim);
                out.push('\n');
            } else {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str(&"  ".repeat(indent));
                out.push_str(&tag_trim);
                out.push('\n');
            }
            i = j + 1;
            continue;
        }

        // 开始标签
        // Lookahead：检查是否是 <tag>text</tag> 这种行内形式
        let tag_name = extract_tag_name(&tag_trim);
        let mut k = j + 1;
        // 跳过空白
        while k < len && chars[k].is_whitespace() {
            k += 1;
        }
        if k < len && chars[k] != '<' {
            // 有文本
            let mut text_end = k;
            while text_end < len && chars[text_end] != '<' {
                text_end += 1;
            }
            let text_content: String = chars[k..text_end].iter().collect();
            let text_trimmed = text_content.trim();
            // 看文本后面是不是对应的闭合标签
            if !text_trimmed.is_empty() && text_end < len && chars[text_end] == '<' {
                // 找下一个标签结束
                let mut m = text_end + 1;
                let mut iq = false;
                let mut qc = '"';
                while m < len {
                    let ch = chars[m];
                    if (ch == '"' || ch == '\'') && chars[m - 1] != '\\' {
                        if !iq {
                            iq = true;
                            qc = ch;
                        } else if ch == qc {
                            iq = false;
                        }
                    }
                    if ch == '>' && !iq {
                        break;
                    }
                    m += 1;
                }
                if m < len {
                    let closing_tag: String = chars[text_end..=m].iter().collect();
                    let closing_name = extract_tag_name(closing_tag.trim());
                    if closing_name == tag_name && closing_tag.trim().starts_with("</") {
                        // 行内标签！ <tag>text</tag>
                        if !out.is_empty() && !out.ends_with('\n') {
                            out.push('\n');
                        }
                        out.push_str(&"  ".repeat(indent));
                        out.push_str(&tag_trim);
                        out.push_str(text_trimmed);
                        out.push_str(closing_tag.trim());
                        out.push('\n');
                        i = m + 1;
                        continue;
                    }
                }
            }
        }

        // 普通开始标签，换行+缩进
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&"  ".repeat(indent));
        out.push_str(&tag_trim);
        out.push('\n');

        if !is_self_closing {
            stack.push(tag_name);
            indent += 1;
        }
        i = j + 1;
    }

    if !stack.is_empty() {
        return Err(format!(
            "XML 解析失败：标签 <{}> 未闭合",
            stack.last().unwrap()
        ));
    }

    Ok(out.trim_end().to_string() + "\n")
}

fn extract_tag_name(tag: &str) -> String {
    let t = tag.trim();
    let mut s = t;
    if s.starts_with("</") {
        s = &s[2..];
    } else if s.starts_with('<') {
        s = &s[1..];
    }
    if s.ends_with("/>") {
        s = &s[..s.len() - 2];
    } else if s.ends_with('>') {
        s = &s[..s.len() - 1];
    }
    if s.starts_with('?') {
        s = &s[1..];
    }
    if s.ends_with('?') {
        s = &s[..s.len() - 1];
    }
    let end = s
        .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .unwrap_or(s.len());
    s[..end].trim().to_string()
}
