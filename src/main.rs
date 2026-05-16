// redmine 바이너리 진입점. clap 파싱 후 cli::run 호출.
use clap::Parser;

use redmine_cli::cli::Cli;

fn main() {
    let cli = Cli::parse();
    redmine_cli::cli::run(cli);
}
