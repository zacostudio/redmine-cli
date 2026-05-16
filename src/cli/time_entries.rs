// time-entry 서브커맨드 핸들러.
use clap::Subcommand;

use crate::client::RedmineClient;

#[derive(Subcommand, Debug)]
pub enum TimeEntryCommand {
    Create,
    List,
    Update,
    Delete,
}

pub fn handle(_cmd: TimeEntryCommand, _client: &RedmineClient) {
    unimplemented!("time-entry handler — Task 12");
}
