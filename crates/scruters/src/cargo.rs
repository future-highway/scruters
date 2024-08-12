pub(crate) use self::{
    cargo_run::CargoRunArgs, cargo_test::CargoTestArgs,
};
use tokio::process::Command;

mod cargo_run;
mod cargo_test;

pub(crate) enum CargoCommandArgs<'a> {
    Run(CargoRunArgs<'a>),
    Test(CargoTestArgs<'a>),
}

impl CargoCommandArgs<'_> {
    pub(crate) fn into_command(self) -> Command {
        match self {
            Self::Run(args) => args.into_command(),
            Self::Test(args) => args.into_command(),
        }
    }
}

impl<'a> From<CargoRunArgs<'a>> for CargoCommandArgs<'a> {
    fn from(args: CargoRunArgs<'a>) -> Self {
        Self::Run(args)
    }
}

impl<'a> From<CargoTestArgs<'a>> for CargoCommandArgs<'a> {
    fn from(args: CargoTestArgs<'a>) -> Self {
        Self::Test(args)
    }
}
