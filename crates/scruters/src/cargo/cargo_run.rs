use alloc::borrow::Cow;
use tokio::process::Command;

pub(crate) struct CargoRunArgs<'a> {
    pub cargo_program: &'a str,
    pub color: bool,
    pub args: Option<Cow<'static, [Cow<'static, str>]>>,
}

impl Default for CargoRunArgs<'_> {
    fn default() -> Self {
        Self {
            cargo_program: "cargo",
            color: true,
            args: None,
        }
    }
}

impl CargoRunArgs<'_> {
    pub fn to_args(&self) -> Vec<&str> {
        let mut cargo_args = vec!["run"];

        if self.color {
            cargo_args.extend(["--color", "always"]);
        }

        if let Some(args) = self.args.as_ref() {
            let args = args.iter().map(AsRef::as_ref);
            cargo_args.extend(args);
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
