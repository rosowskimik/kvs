mod args;

use kvs::{KvStore, Result};
fn main() -> Result<()> {
    let matches = args::get_cli_args();

    let mut store = KvStore::open("./data")?;

    match matches.subcommand() {
        ("set", Some(args)) => {
            let key = args.value_of("key").unwrap();
            let value = args.value_of("value").unwrap();
            store.set(key, value)?;
        }
        ("get", Some(args)) => {
            let key = args.value_of("key").unwrap();
            let value = store
                .get(key)?
                .unwrap_or_else(|| "Key not found".to_string());

            println!("{}", value);
        }
        ("rm", Some(args)) => {
            let key = args.value_of("key").unwrap();
            if !store.remove(key)? {
                println!("Key not found");
                std::process::exit(1);
            }
        }
        _ => unreachable!(),
    };

    Ok(())
}
