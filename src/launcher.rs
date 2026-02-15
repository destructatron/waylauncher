use std::process::Command;

use log::{error, info};

/// Remove freedesktop field codes (%f, %F, %u, %U, %d, %D, %n, %N, %i, %c, %k, %v, %m)
/// from an Exec string.
pub fn strip_field_codes(exec: &str) -> String {
    let mut result = String::with_capacity(exec.len());
    let mut chars = exec.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(&next) = chars.peek() {
                if "fFuUdDnNickvm".contains(next) {
                    chars.next(); // consume the field code letter
                    // Also consume a trailing space if present
                    if chars.peek() == Some(&' ') {
                        chars.next();
                    }
                    continue;
                }
            }
        }
        result.push(ch);
    }

    result.trim().to_string()
}

/// Launch an application from its Exec string.
/// If `terminal` is true, wraps the command in a terminal emulator.
pub fn launch(exec: &str, terminal: bool) {
    let cleaned = strip_field_codes(exec);

    let args = match shell_words::split(&cleaned) {
        Ok(args) if !args.is_empty() => args,
        Ok(_) => {
            error!("Empty exec string after processing");
            return;
        }
        Err(e) => {
            error!("Failed to parse exec string '{}': {}", cleaned, e);
            return;
        }
    };

    let (program, program_args) = if terminal {
        let terminal_cmd = std::env::var("TERMINAL").unwrap_or_else(|_| "xdg-terminal-exec".into());
        (terminal_cmd, {
            let mut a = vec!["-e".to_string()];
            a.extend(args);
            a
        })
    } else {
        let (first, rest) = args.split_first().unwrap();
        (first.clone(), rest.to_vec())
    };

    info!("Launching: {} {:?}", program, program_args);

    match Command::new(&program).args(&program_args).spawn() {
        Ok(_) => {}
        Err(e) => error!("Failed to launch '{}': {}", program, e),
    }
}
