use std::collections::hash_map::{Entry, HashMap};

use crate::{
    fatal,
    tester::{Status, TestCmd, Tests},
};

/// Parse test data into a set of `Test`s.
pub(crate) fn parse_tests(test_str: &str) -> Tests {
    let lines = test_str.lines().collect::<Vec<_>>();
    let mut tests = HashMap::new();
    let mut line_off = 0;
    let mut ignore = false;
    while line_off < lines.len() {
        let indent = indent_level(&lines, line_off);
        if indent == lines[line_off].len() {
            line_off += 1;
            continue;
        }
        let (test_name, val) = key_val(&lines, line_off, indent);
        if test_name == "ignore" {
            ignore = true;
            line_off += 1;
            continue;
        }
        if !val.is_empty() {
            fatal(&format!(
                "Test name '{}' can't have a value on line {}.",
                test_name, line_off
            ));
        }
        match tests.entry(test_name.to_lowercase()) {
            Entry::Occupied(_) => fatal(&format!(
                "Command name '{}' is specified more than once, line {}.",
                test_name, line_off
            )),
            Entry::Vacant(e) => {
                line_off += 1;
                let mut testcmd = TestCmd::default();
                while line_off < lines.len() {
                    let sub_indent = indent_level(&lines, line_off);
                    if sub_indent == lines[line_off].len() {
                        line_off += 1;
                        continue;
                    }
                    if sub_indent == indent {
                        break;
                    }
                    let (end_line_off, key, val) = key_multiline_val(&lines, line_off, sub_indent);
                    line_off = end_line_off;
                    match key {
                        "extra-args" => {
                            let val_str = val.join("\n");
                            testcmd.args.push(val_str);
                        }
                        "status" => {
                            let val_str = val.join("\n");
                            let status = match val_str.to_lowercase().as_str() {
                                "success" => Status::Success,
                                "error" => Status::Error,
                                "signal" => Status::Signal,
                                x => {
                                    if let Ok(i) = x.parse::<i32>() {
                                        Status::Int(i)
                                    } else {
                                        fatal(&format!(
                                            "Unknown status '{}' on line {}",
                                            val_str, line_off
                                        ));
                                    }
                                }
                            };
                            testcmd.status = status;
                        }
                        "stderr" => {
                            testcmd.stderr = val;
                        }
                        "stdout" => {
                            testcmd.stdout = val;
                        }
                        _ => fatal(&format!("Unknown key '{}' on line {}.", key, line_off)),
                    }
                }
                e.insert(testcmd);
            }
        }
    }
    Tests { ignore, tests }
}

fn indent_level(lines: &[&str], line_off: usize) -> usize {
    lines[line_off]
        .chars()
        .take_while(|c| c.is_whitespace())
        .count()
}

/// Turn a line such as `key: val` into its separate components.
fn key_val<'a>(lines: &[&'a str], line_off: usize, indent: usize) -> (&'a str, &'a str) {
    let line = lines[line_off];
    let key_len = line[indent..]
        .chars()
        .take_while(|c| !(c.is_whitespace() || c == &':'))
        .count();
    let key = &line[indent..indent + key_len];
    let mut content_start = indent + key_len;
    content_start += line[content_start..]
        .chars()
        .take_while(|c| c.is_whitespace())
        .count();
    match line[content_start..].chars().nth(0) {
        Some(':') => content_start += ':'.len_utf8(),
        _ => fatal(&format!(
            "Invalid key terminator at line {}.\n  {}",
            line_off, line
        )),
    }
    content_start += line[content_start..]
        .chars()
        .take_while(|c| c.is_whitespace())
        .count();
    (key, &line[content_start..].trim())
}

/// Turn one more lines of the format `key: val` (where `val` may spread over many lines) into its
/// separate components. Guarantees to trim leading and trailing newlines.
fn key_multiline_val<'a>(
    lines: &[&'a str],
    mut line_off: usize,
    indent: usize,
) -> (usize, &'a str, Vec<&'a str>) {
    let (key, first_line_val) = key_val(lines, line_off, indent);
    line_off += 1;
    let mut val = vec![first_line_val];
    if line_off < lines.len() {
        let sub_indent = indent_level(lines, line_off);
        while line_off < lines.len() {
            let cur_indent = indent_level(lines, line_off);
            if cur_indent == lines[line_off].len() {
                val.push("");
                line_off += 1;
                continue;
            }
            if cur_indent <= indent {
                break;
            }
            val.push(&lines[line_off][sub_indent..].trim());
            line_off += 1;
        }
    }
    // Remove trailing empty strings
    while !val.is_empty() && val[val.len() - 1].is_empty() {
        val.pop();
    }
    // Remove leading empty strings
    while !val.is_empty() && val[0].is_empty() {
        val.remove(0);
    }

    (line_off, key, val)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_key_multiline() {
        assert_eq!(key_multiline_val(&["x:", ""], 0, 0), (2, "x", vec![]));
        assert_eq!(
            key_multiline_val(&["x: y", "  z", "a"], 0, 0),
            (2, "x", vec!["y", "z"])
        );
        assert_eq!(
            key_multiline_val(&["x:", "  z", "a"], 0, 0),
            (2, "x", vec!["z"])
        );
        assert_eq!(
            key_multiline_val(&["x:", "  z  ", "  a  ", "  ", "b"], 0, 0),
            (4, "x", vec!["z", "a"])
        );
    }
}
