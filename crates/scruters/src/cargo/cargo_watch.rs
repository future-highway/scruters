use super::CargoTestArgs;
use tokio::process::Command;

pub(crate) struct CargoWatchTestArgs<'a> {
    test_args: CargoTestArgs<'a>,
}

impl<'a> CargoWatchTestArgs<'a> {
    pub fn new(test_args: CargoTestArgs<'a>) -> Self {
        Self { test_args }
    }

    pub fn into_command(self) -> Command {
        let mut command =
            Command::new(self.test_args.cargo_program);

        _ = command.args(["watch", "-x"]);
        _ = command.args(self.test_args.to_args());

        command
    }
}
