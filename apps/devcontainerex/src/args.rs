pub enum Args {
    Exec(ExecArgs),
    Passthrough(Vec<String>),
}

pub struct ExecArgs {
    pre: Vec<String>,
    post: Vec<String>,
    pub workspace_folder: Option<String>,
    pub has_workspace_flag: bool,
    pub has_id_label: bool,
    pub help_requested: bool,
}

impl Args {
    pub fn parse(argv: Vec<String>) -> Args {
        match find_exec_token(&argv) {
            Some(i) if !contains_help(&argv[..i]) => {
                let post = argv[i + 1..].to_vec();
                let mut pre = argv;
                pre.truncate(i);
                Args::Exec(ExecArgs::new(pre, post))
            }
            _ => Args::Passthrough(argv),
        }
    }
}

impl ExecArgs {
    fn new(pre: Vec<String>, post: Vec<String>) -> Self {
        let workspace_folder = peek_workspace_flag(&pre).or_else(|| peek_workspace_flag(&post));
        let has_workspace_flag = [&pre, &post]
            .iter()
            .any(|a| has_flag(a, "--workspace-folder") || has_flag(a, "-w"));
        let has_id_label = has_flag(&pre, "--id-label") || has_flag(&post, "--id-label");
        let help_requested = contains_help(&post);
        ExecArgs {
            pre,
            post,
            workspace_folder,
            has_workspace_flag,
            has_id_label,
            help_requested,
        }
    }

    pub fn into_argv(self, insert: Vec<String>) -> Vec<String> {
        let mut argv = self.pre;
        argv.push("exec".to_string());
        argv.extend(insert);
        argv.extend(self.post);
        argv
    }
}

fn find_exec_token(argv: &[String]) -> Option<usize> {
    argv.iter()
        .position(|a| a == "--" || a == "exec")
        .filter(|&i| argv[i] == "exec")
}

fn contains_help(tokens: &[String]) -> bool {
    tokens
        .iter()
        .take_while(|a| *a != "--")
        .any(|a| a == "--help" || a == "--version")
}

fn peek_workspace_flag(args: &[String]) -> Option<String> {
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        if a == "--" {
            break;
        }
        if a == "--workspace-folder" || a == "-w" {
            return iter.next().cloned();
        }
        if let Some(v) = a.strip_prefix("--workspace-folder=") {
            return Some(v.to_string());
        }
        if let Some(v) = a.strip_prefix("-w=") {
            return Some(v.to_string());
        }
    }
    None
}

fn has_flag(args: &[String], name: &str) -> bool {
    for a in args {
        if a == "--" {
            break;
        }
        if a == name {
            return true;
        }
        if let Some(rest) = a.strip_prefix(name) {
            if rest.starts_with('=') {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    fn parse_exec(args: &[&str]) -> ExecArgs {
        match Args::parse(v(args)) {
            Args::Exec(e) => e,
            Args::Passthrough(_) => panic!("expected Exec"),
        }
    }

    #[test]
    fn no_args_is_passthrough() {
        assert!(matches!(Args::parse(Vec::new()), Args::Passthrough(_)));
    }

    #[test]
    fn exec_without_command_is_kept_as_is() {
        let e = parse_exec(&["exec"]);
        assert_eq!(e.into_argv(Vec::new()), v(&["exec"]));
    }

    #[test]
    fn exec_as_first_arg() {
        assert_eq!(find_exec_token(&v(&["exec", "bash"])), Some(0));
    }

    #[test]
    fn exec_after_options() {
        assert_eq!(
            find_exec_token(&v(&["--log-level", "trace", "exec", "bash"])),
            Some(2)
        );
    }

    #[test]
    fn other_subcommands_are_passthrough() {
        assert!(matches!(
            Args::parse(v(&["up", "--workspace-folder", "."])),
            Args::Passthrough(_)
        ));
        assert!(matches!(
            Args::parse(v(&["features", "test"])),
            Args::Passthrough(_)
        ));
    }

    #[test]
    fn exec_after_double_dash_is_ignored() {
        // `up -- exec` のような並びで exec と誤認しない
        assert!(matches!(
            Args::parse(v(&["up", "--", "exec"])),
            Args::Passthrough(_)
        ));
    }

    #[test]
    fn help_before_exec_is_passthrough() {
        assert!(matches!(
            Args::parse(v(&["--help", "exec", "bash"])),
            Args::Passthrough(_)
        ));
    }

    #[test]
    fn help_after_exec_is_detected() {
        assert!(parse_exec(&["exec", "--help"]).help_requested);
        assert!(!parse_exec(&["exec", "bash"]).help_requested);
    }

    #[test]
    fn help_after_double_dash_is_container_side() {
        // `--` 以降はコンテナ内コマンドの引数なのでヘルプ扱いにしない
        assert!(!parse_exec(&["exec", "--", "somecmd", "--help"]).help_requested);
    }

    #[test]
    fn insert_goes_right_after_exec_token() {
        let e = parse_exec(&["--log-level", "trace", "exec", "bash"]);
        assert_eq!(
            e.into_argv(v(&["--id-label", "a=b"])),
            v(&["--log-level", "trace", "exec", "--id-label", "a=b", "bash"])
        );
    }

    #[test]
    fn workspace_space_form() {
        let e = parse_exec(&["exec", "--workspace-folder", "/p", "bash"]);
        assert_eq!(e.workspace_folder.as_deref(), Some("/p"));
        assert!(e.has_workspace_flag);
    }

    #[test]
    fn workspace_equals_form() {
        let e = parse_exec(&["exec", "--workspace-folder=/p", "bash"]);
        assert_eq!(e.workspace_folder.as_deref(), Some("/p"));
    }

    #[test]
    fn workspace_short_form() {
        assert_eq!(
            parse_exec(&["exec", "-w", "/p"])
                .workspace_folder
                .as_deref(),
            Some("/p")
        );
        assert_eq!(
            parse_exec(&["exec", "-w=/p"]).workspace_folder.as_deref(),
            Some("/p")
        );
    }

    #[test]
    fn workspace_before_exec_token() {
        let e = parse_exec(&["--workspace-folder", "/p", "exec", "bash"]);
        assert_eq!(e.workspace_folder.as_deref(), Some("/p"));
        assert!(e.has_workspace_flag);
    }

    #[test]
    fn workspace_none() {
        let e = parse_exec(&["exec", "bash"]);
        assert_eq!(e.workspace_folder, None);
        assert!(!e.has_workspace_flag);
    }

    #[test]
    fn workspace_not_peeked_after_double_dash() {
        // `--` 以降はコンテナ内コマンドの引数なので覗かない
        let e = parse_exec(&["exec", "--", "somecmd", "--workspace-folder", "/x"]);
        assert_eq!(e.workspace_folder, None);
    }

    #[test]
    fn id_label_both_forms() {
        assert!(parse_exec(&["exec", "--id-label", "a=b"]).has_id_label);
        assert!(parse_exec(&["exec", "--id-label=a=b"]).has_id_label);
        assert!(parse_exec(&["--id-label", "a=b", "exec"]).has_id_label);
        assert!(!parse_exec(&["exec", "--id-labelx"]).has_id_label);
        assert!(!parse_exec(&["exec", "bash"]).has_id_label);
        assert!(!parse_exec(&["exec", "--", "--id-label"]).has_id_label);
    }
}
