use async_graphql::EmptySubscription;
use clap::{AppSettings, Clap};
use std::{fs, io::Write};

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short, long)]
    output: String,
}

fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();
    let schema = di::create_schema(
        graphql::Query::default(),
        graphql::Mutation::default(),
        EmptySubscription,
    );
    let mut output_file = fs::File::create(&opts.output)?;
    output_file.write_all(schema.sdl().as_bytes())?;
    Ok(())
}
