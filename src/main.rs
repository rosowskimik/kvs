mod args;

// use kvs::KvStore;

fn main() {
    let matches = args::get_cli_args();

    // let mut store = KvStore::new();

    match matches.subcommand() {
        ("set", Some(args)) => {
            panic!("unimplemented")
        }
        ("get", Some(args)) => {
            panic!("unimplemented")
        }
        ("rm", Some(args)) => {
            panic!("unimplemented")
        }
        _ => unreachable!(),
    };
}
