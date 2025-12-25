use crate::config::AppConfig;
use regex::Regex;

pub fn format_str(cfg: &AppConfig, text: &str) -> String {
    if !cfg.format_input {
        return text.to_string();
    }

    // Mimic utils.format_str: normalize commas/spaces per line, preserve newlines.
    let comma_re = Regex::new(r"[,\s]*,[,\s]*").expect("valid regex");
    let spaces_re = Regex::new(r" +").expect("valid regex");

    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        if line == "\n" {
            out.push('\n');
            continue;
        }
        let (content, has_nl) = line
            .strip_suffix('\n')
            .map(|c| (c, true))
            .unwrap_or((line, false));
        let mut s = comma_re.replace_all(content, ", ").to_string();
        s = spaces_re.replace_all(&s, " ").to_string();
        let s = s.trim();
        out.push_str(s);
        if has_nl {
            out.push('\n');
        }
    }
    out
}
