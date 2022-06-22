#![allow(dead_code)]
use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};

use serde::Deserialize;
use toml_spanned_value::SpannedValue;

#[derive(Debug, Deserialize)]
struct Config {
    age: i32,
}

fn main() {
    let toml_str = "\
age=1
ff=123

[shit]
ok=*true
";

    if let Err(e) = toml::from_str::<SpannedValue>(toml_str) {
        let pattern = regex::Regex::new(r" at line \d column \d").unwrap();
        let error_msg = e.to_string();
        let error_msg = pattern.replace(&error_msg, "");

        let (line, col) = e.line_col().unwrap();
        let offset =
            toml_str
                .split('\n')
                .take(line + 1)
                .enumerate()
                .fold(0, |cu, (idx, line_str)| {
                    if idx == line {
                        cu + col
                    } else {
                        cu + line_str.chars().count() + 1
                    }
                });

        let mut colors = ColorGenerator::new();
        let a = colors.next();
        Report::build(ReportKind::Error, (), offset)
            .with_message("Syntax Error")
            .with_label(
                Label::new(offset..offset + 1)
                    .with_message(error_msg.fg(a))
                    .with_color(a),
            )
            .finish()
            .print(Source::from(toml_str))
            .unwrap();
    }
}
