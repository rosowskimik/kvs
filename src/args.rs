use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg, ArgMatches,
    SubCommand,
};

pub fn get_cli_args<'src>() -> ArgMatches<'src> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("set")
                .about("Sets the value of <VALUE> under <KEY>")
                .arg(
                    Arg::with_name("key")
                        .required(true)
                        .takes_value(true)
                        .value_name("KEY"),
                )
                .arg(
                    Arg::with_name("value")
                        .required(true)
                        .takes_value(true)
                        .value_name("VALUE"),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Gets the value of a given <KEY>")
                .arg(
                    Arg::with_name("key")
                        .required(true)
                        .takes_value(true)
                        .value_name("KEY"),
                ),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Removes a given <KEY>")
                .arg(
                    Arg::with_name("key")
                        .required(true)
                        .takes_value(true)
                        .value_name("KEY"),
                ),
        )
        .get_matches();

    matches
}
