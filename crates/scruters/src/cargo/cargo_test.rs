use alloc::borrow::Cow;
use tokio::process::Command;

pub(crate) struct CargoTestArgs<'a> {
    pub cargo_program: &'a str,
    pub color: bool,
    pub list: bool,
    pub fail_fast: bool,
    pub args: Option<Cow<'static, [Cow<'static, str>]>>,
    pub test_args: Option<&'a [&'a str]>,
}

impl Default for CargoTestArgs<'_> {
    fn default() -> Self {
        Self {
            cargo_program: "cargo",
            color: true,
            list: false,
            fail_fast: false,
            args: None,
            test_args: None,
        }
    }
}

impl CargoTestArgs<'_> {
    pub fn to_args(&self) -> Vec<&str> {
        let mut cargo_args = vec!["test"];

        if !self.fail_fast {
            cargo_args.push("--no-fail-fast");
        }

        if self.color {
            cargo_args.extend(["--color", "always"]);
        }

        if let Some(args) = self.args.as_ref() {
            let args = args.iter().map(AsRef::as_ref);
            cargo_args.extend(args);
        }

        cargo_args.push("--");

        if self.color {
            cargo_args.extend(["--color", "always"]);
        }

        if self.list {
            cargo_args
                .extend(["--list", "--format", "terse"]);
        }

        if let Some(test_args) = self.test_args {
            cargo_args.extend(test_args);
        }

        cargo_args
    }

    pub fn into_command(self) -> Command {
        let args = self.to_args();
        let mut command = Command::new(self.cargo_program);
        _ = command.args(args);
        command
    }
}
