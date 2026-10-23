//! The clap command tree.

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("rstory").chain(args.iter().copied()))
    }

    #[test]
    fn every_command_of_the_contract_parses() {
        for args in [
            &["daemon"][..],
            &["daemon", "--backend", "x11"],
            &["ls"],
            &["ls", "--json", "--limit", "5"],
            &["show", "0"],
            &["show", "ab12", "--json"],
            &["copy", "1"],
            &["pin", "1"],
            &["unpin", "1"],
            &["rm", "1"],
            &["clear"],
            &["clear", "--all", "--yes"],
            &["pause"],
            &["pause", "--for", "50s"],
            &["resume"],
            &["status", "--json"],
            &["config"],
            &["config", "--path"],
            &["pick", "--plain"],
            &["doctor", "--json"],
            &["install-autostart", "--systemd", "--now"],
            &["install-gnome-extension", "--enable"],
            &["completions", "bash"],
            &["completions", "zsh"],
            &["completions", "fish"],
        ] {
            assert!(parse(args).is_ok(), "{args:?}");
        }
    }

    #[test]
    fn bad_input_is_a_usage_error() {
        for args in [
            &["show"][..],
            &["show", "abc"],
            &["pause", "--for", "soon"],
            &["pause", "--for", "0s"],
            &["pause", "--for", "25h"],
            &["ls", "--limit", "0"],
            &["completions", "powershell"],
            &["nope"],
        ] {
            let err = parse(args).expect_err(&format!("{args:?}"));
            assert_eq!(err.exit_code(), 2, "{args:?}");
        }
    }

    #[test]
    fn every_command_has_help_with_an_example() {
        use clap::CommandFactory;
        let cmd = Cli::command();
        for sub in cmd.get_subcommands() {
            let name = sub.get_name();
            if ["daemon", "pick", "doctor", "install-autostart", "install-gnome-extension"]
                .contains(&name)
            {
                continue; // owned by other specs
            }
            let help = sub.clone().render_long_help().to_string();
            assert!(help.contains("rstory "), "{name} has no example");
            assert!(sub.get_about().is_some(), "{name} has no about");
        }
    }

    #[test]
    fn completions_scripts_are_generated() {
        for shell in ["bash", "zsh", "fish"] {
            let mut out = Vec::new();
            completions(shell.parse().unwrap(), &mut out);
            let text = String::from_utf8(out).unwrap();
            assert!(text.contains("rstory"), "{shell}");
            assert!(text.contains("clear"), "{shell}");
        }
    }
}
