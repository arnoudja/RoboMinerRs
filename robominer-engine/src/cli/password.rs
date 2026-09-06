use anyhow::{Result, bail, ensure};
use std::io::{self, IsTerminal, Write};

/// Resolve a user password from CLI flag/env value, else a TTY prompt.
///
/// Passing `--password` on argv remains supported for automation, but operators
/// should prefer `ROBOMINER_USER_PASSWORD` or an interactive prompt so secrets
/// do not appear in process lists or shell history.
pub(crate) fn resolve_user_password(
    cli_password: Option<String>,
    prompt_label: &str,
) -> Result<String> {
    if let Some(password) = cli_password {
        ensure!(
            !password.is_empty(),
            "--password / ROBOMINER_USER_PASSWORD must not be empty"
        );
        return Ok(password);
    }

    let stdin = io::stdin();
    ensure!(
        stdin.is_terminal(),
        "password required: pass --password, set ROBOMINER_USER_PASSWORD, or run in a TTY"
    );

    eprint!("{prompt_label}: ");
    let _ = io::stderr().flush();
    let mut password = String::new();
    stdin.read_line(&mut password)?;
    let password = password.trim_end_matches(['\r', '\n']).to_string();
    if password.is_empty() {
        bail!("password must not be empty");
    }
    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::resolve_user_password;

    #[test]
    fn resolve_user_password_accepts_cli_value() {
        let password = resolve_user_password(Some("test-password-1".into()), "Password")
            .expect("cli password");
        assert_eq!(password, "test-password-1");
    }
}
