//! # Client
//!
//! `client` contains the CLI client type and functions.

use crate::common;
use crate::result::Result;
use clap::{App, Arg, ArgMatches, SubCommand};

/// `add_lookup` adds a lookup command to the `App`.
fn add_lookup(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("lookup")
        .about("Lookup an item in the store")
        .arg(
            Arg::with_name("key")
                .help("Key of the item to lookup")
                .takes_value(true)
                .value_name("KEY"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_get` adds a get command to the `App`.
fn add_get(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("get")
        .about("Get an item from the store")
        .arg(
            Arg::with_name("key")
                .help("Key of the item to get")
                .takes_value(true)
                .value_name("KEY"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_count` adds a count command to the `App`.
fn add_count(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("count")
        .about("Counts items in the store")
        .arg(
            Arg::with_name("from")
                .help("Key from where the count starts")
                .long("from")
                .takes_value(true)
                .value_name("FROM"),
        )
        .arg(
            Arg::with_name("to")
                .help("Key to where the count ends")
                .long("to")
                .takes_value(true)
                .value_name("TO"),
        )
        .arg(
            Arg::with_name("skip")
                .help("Number of items skipeed at start")
                .long("skip")
                .takes_value(true)
                .value_name("SKIP"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_query` adds a query command to the `App`.
fn add_query(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("query")
        .about("Queries items in the store")
        .arg(
            Arg::with_name("from")
                .help("Key from where the count starts")
                .long("from")
                .takes_value(true)
                .value_name("FROM"),
        )
        .arg(
            Arg::with_name("to")
                .help("Key to where the count ends")
                .long("to")
                .takes_value(true)
                .value_name("TO"),
        )
        .arg(
            Arg::with_name("count")
                .help("Maximum number of items returned")
                .long("count")
                .takes_value(true)
                .value_name("COUNT"),
        )
        .arg(
            Arg::with_name("skip")
                .help("Number of items skipeed at start")
                .long("skip")
                .takes_value(true)
                .value_name("SKIP"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_insert` adds a insert command to the `App`.
fn add_insert(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("insert")
        .about("Inserts an item in the store")
        .arg(
            Arg::with_name("key")
                .help("Key of the item to insert")
                .short("k")
                .long("key")
                .takes_value(true)
                .value_name("KEY"),
        )
        .arg(
            Arg::with_name("value")
                .help("Value of the item to insert")
                .short("v")
                .long("value")
                .takes_value(true)
                .value_name("VALUE"),
        )
        .arg(
            Arg::with_name("format")
                .help("Item format")
                .short("F")
                .long("format")
                .takes_value(true)
                .value_name("FORMAT")
                .possible_values(&["json", "hex"])
                .default_value("hex")
                .requires("value"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_update` adds a update command to the `App`.
fn add_update(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("update")
        .about("Updates an item in the store")
        .arg(
            Arg::with_name("key")
                .help("Key of the item to update")
                .short("k")
                .long("key")
                .takes_value(true)
                .value_name("KEY"),
        )
        .arg(
            Arg::with_name("value")
                .help("Value of the item to update")
                .short("v")
                .long("value")
                .takes_value(true)
                .value_name("VALUE"),
        )
        .arg(
            Arg::with_name("format")
                .help("Item format")
                .short("F")
                .long("format")
                .takes_value(true)
                .value_name("FORMAT")
                .possible_values(&["json", "hex"])
                .default_value("hex")
                .requires("value"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_remove` adds a remove command to the `App`.
fn add_remove(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("remove")
        .about("Remove an item from the store")
        .arg(
            Arg::with_name("key")
                .help("Key of the item to remove")
                .takes_value(true)
                .value_name("KEY"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_cleanup` adds a cleanup command to the `App`.
fn add_cleanup(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("cleanup")
        .about("Cleanup the store items")
        .arg(
            Arg::with_name("to")
                .help("Timestamp to clean up to")
                .takes_value(true)
                .value_name("TO"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_clean` adds a clean command to the `App`.
fn add_clean(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("clean").about("Clean the store");

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_storable` adds the storable commands to the `App`.
fn add_storable(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut app = add_lookup(app);
    app = add_get(app);
    app = add_count(app);
    app = add_query(app);
    app = add_insert(app);
    app = add_update(app);
    app = add_remove(app);
    app = add_cleanup(app);
    app = add_clean(app);

    app
}

/// `add_fetch` adds a fetch command to the `App`.
fn add_fetch(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("fetch")
        .about("Fetch from remote")
        .arg(
            Arg::with_name("address")
                .help("Node address to fetch from")
                .short("a")
                .long("address")
                .takes_value(true)
                .value_name("ADDRESS")
                .required(true),
        )
        .arg(
            Arg::with_name("keys")
                .help("Keys of the items to fetch")
                .long("keys")
                .takes_value(true)
                .value_name("KEYS")
                .conflicts_with("count"),
        )
        .arg(
            Arg::with_name("count")
                .help("Count of the items to fetch")
                .short("C")
                .long("count")
                .takes_value(true)
                .value_name("COUNT")
                .conflicts_with("keys"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_push` adds a push command to the `App`.
fn add_push(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("push")
        .about("Push to remote")
        .arg(
            Arg::with_name("address")
                .help("Node address to push to")
                .short("a")
                .long("address")
                .takes_value(true)
                .value_name("ADDRESS")
                .required(true),
        )
        .arg(
            Arg::with_name("key")
                .help("Key of the item to push")
                .short("k")
                .long("key")
                .takes_value(true)
                .value_name("KEY")
                .conflicts_with("value"),
        )
        .arg(
            Arg::with_name("value")
                .help("Value of the item to push")
                .short("v")
                .long("value")
                .takes_value(true)
                .value_name("VALUE")
                .conflicts_with("key"),
        )
        .arg(
            Arg::with_name("format")
                .help("Item format")
                .short("F")
                .long("format")
                .takes_value(true)
                .value_name("FORMAT")
                .possible_values(&["json", "hex"])
                .default_value("hex")
                .requires("value"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_wallet_create` adds a create command to the wallet subcommand.
fn add_wallet_create(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("create").about("Create a new wallet");

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_wallet` adds a wallet command to the `App`.
fn add_wallet(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("wallet").about("Wallet operations");

    cmd = add_wallet_create(cmd);
    cmd = add_storable(cmd);

    app.subcommand(cmd)
}

/// `add_account_create` adds a create command to the account subcommand.
fn add_account_create(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("create")
        .about("Create a new account")
        .arg(
            Arg::with_name("eve")
                .help("Create an eve transaction")
                .long("eve")
                .takes_value(false),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_add_signer` adds a command to the `App` to add a signer.
fn add_add_signer(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("add-signer")
        .about("Add a signer to an account")
        .arg(
            Arg::with_name("address")
                .help("Address of the account")
                .short("a")
                .long("address")
                .takes_value(true)
                .value_name("ADDRESS")
                .required(true),
        )
        .arg(
            Arg::with_name("signer")
                .help("Address of the signer")
                .long("signer")
                .takes_value(true)
                .value_name("ADDRESS")
                .required(true),
        )
        .arg(
            Arg::with_name("weight")
                .help("Weight of the signer")
                .long("weight")
                .takes_value(true)
                .required(true),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_account` adds a account command to the `App`.
fn add_account(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("account").about("Account operations");

    cmd = add_account_create(cmd);
    cmd = add_add_signer(cmd);
    cmd = add_storable(cmd);

    app.subcommand(cmd)
}

/// `add_transaction_create` adds a create command to the transaction subcommand.
fn add_transaction_create(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("create")
        .about("Create a new transaction")
        .arg(
            Arg::with_name("eve")
                .help("Create an eve transaction")
                .long("eve")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("address")
                .help("Address of the eve account")
                .takes_value(true)
                .value_name("ADDRESS")
                .requires("eve"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_set_locktime` adds a command to set the transaction locktime to the `App`.
fn add_set_locktime(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("set-locktime")
        .about("Sets the locktime of the transaction")
        .arg(
            Arg::with_name("locktime")
                .help("Locktime of the transaction")
                .takes_value(true)
                .value_name("LOCKTIME")
                .required(true),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_add_input` adds a command to add a transaction input to the `App`.
fn add_add_input(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("add-input")
        .about("Add an input to a transaction")
        .arg(
            Arg::with_name("address")
                .help("Address of the account")
                .short("a")
                .long("address")
                .takes_value(true)
                .value_name("ADDRESS")
                .required(true),
        )
        .arg(
            Arg::with_name("amount")
                .help("Amount of the input")
                .long("amount")
                .takes_value(true)
                .required(true),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_add_output` adds a command to add a transaction output to the `App`.
fn add_add_output(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("add-output")
        .about("Add an output to a transaction")
        .arg(
            Arg::with_name("address")
                .help("Address of the account")
                .short("a")
                .long("address")
                .takes_value(true)
                .value_name("ADDRESS")
                .required(true),
        )
        .arg(
            Arg::with_name("amount")
                .help("Amount of the output")
                .long("amount")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("custom")
                .help("Custom data of the output")
                .long("custom")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("format")
                .help("Format of the output data")
                .short("F")
                .long("format")
                .takes_value(true)
                .value_name("FORMAT")
                .possible_values(&["binary", "hex"])
                .default_value("binary")
                .requires("custom"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_set_coinbase` adds a command to set the transaction coinbase to the `App`.
fn add_set_coinbase(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("set-coinbase")
        .about("Sets the coinbase of the transaction")
        .arg(
            Arg::with_name("coinbase")
                .help("Coinbase of the transaction")
                .takes_value(true)
                .value_name("COINBASE")
                .required(true),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_transaction` adds a transaction command to the `App`.
fn add_transaction(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("transaction").about("Transaction operations");

    cmd = add_transaction_create(cmd);
    cmd = add_set_locktime(cmd);
    cmd = add_add_input(cmd);
    cmd = add_add_output(cmd);
    cmd = add_set_coinbase(cmd);
    cmd = add_fetch(cmd);
    cmd = add_push(cmd);
    cmd = add_storable(cmd);

    app.subcommand(cmd)
}

/// `add_node_create` adds a create command to the node subcommand.
fn add_node_create(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("create")
        .about("Create a new node")
        .arg(
            Arg::with_name("address")
                .help("Address of the node to create")
                .takes_value(true)
                .value_name("ADDRESS")
                .required(true),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_node` adds a node command to the `App`.
fn add_node(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("node").about("Node operations");

    cmd = add_node_create(cmd);
    cmd = add_fetch(cmd);
    cmd = add_push(cmd);
    cmd = add_storable(cmd);

    app.subcommand(cmd)
}

/// `add_consensus_state` adds a consensus-state command to the `App`.
fn add_consensus_state(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("consensus-state").about("Consensus state operations");

    cmd = add_get(cmd);
    cmd = add_count(cmd);
    cmd = add_query(cmd);
    cmd = add_cleanup(cmd);
    cmd = add_clean(cmd);

    app.subcommand(cmd)
}

/// `add_consensus_message` adds a consensus-message command to the `App`.
fn add_consensus_message(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("consensus-message").about("Consensus message operations");

    cmd = add_get(cmd);
    cmd = add_count(cmd);
    cmd = add_query(cmd);
    cmd = add_cleanup(cmd);
    cmd = add_clean(cmd);

    app.subcommand(cmd)
}

/// `add_import` adds an import command to the `App`.
fn add_import(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("import")
        .about("Import store comma-separated items from a file")
        .arg(
            Arg::with_name("file")
                .help("File with the comma separated key-values to import")
                .takes_value(true)
                .value_name("FILE")
                .conflicts_with("file"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_export` adds an export command to the `App`.
fn add_export(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("export")
        .about("Export the store in comma-separated items to a file")
        .arg(
            Arg::with_name("file")
                .help("File where to export")
                .takes_value(true)
                .value_name("FILE")
                .conflicts_with("file"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_size` adds a size command to the `App`.
fn add_size(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("size")
        .about("Returns the size of the store keys and values")
        .arg(
            Arg::with_name("keys")
                .help("Returns the size of the store keys")
                .long("keys")
                .takes_value(false)
                .conflicts_with("all"),
        )
        .arg(
            Arg::with_name("values")
                .help("Returns the size of the store values")
                .long("values")
                .takes_value(false)
                .conflicts_with("all"),
        )
        .arg(
            Arg::with_name("all")
                .help("Returns the size of the store keys and values")
                .long("all")
                .takes_value(false),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_store` adds a store command to the `App`.
fn add_store(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("store").about("Store operations");

    cmd = add_import(cmd);
    cmd = add_export(cmd);
    cmd = add_size(cmd);
    cmd = add_clean(cmd);

    app.subcommand(cmd)
}

/// `add_hash` adds a hash command to the `App`.
fn add_hash(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("hash")
        .about("Hashes a message or a file")
        .arg(
            Arg::with_name("message")
                .help("Message to hash")
                .short("m")
                .long("message")
                .takes_value(true)
                .value_name("MESSAGE")
                .conflicts_with("file")
                .required_unless("file"),
        )
        .arg(
            Arg::with_name("file")
                .help("File to hash")
                .short("f")
                .long("file")
                .takes_value(true)
                .value_name("FILE")
                .conflicts_with("message")
                .required_unless("message"),
        )
        .arg(
            Arg::with_name("format")
                .help("Format of the file to hash")
                .short("F")
                .long("format")
                .takes_value(true)
                .value_name("FORMAT")
                .possible_values(&["binary", "hex"])
                .default_value("binary")
                .requires("file"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_sign` adds a sign command to the `App`.
fn add_sign(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("sign")
        .about("Signs a message or a store transaction")
        .arg(
            Arg::with_name("address")
                .help("Account address")
                .short("a")
                .long("address")
                .takes_value(true)
                .value_name("ADDRESS")
                .required(true),
        )
        .arg(
            Arg::with_name("message")
                .help("Hex of the message to sign")
                .short("m")
                .long("message")
                .takes_value(true)
                .value_name("MESSAGE")
                .conflicts_with("key")
                .required_unless("key"),
        )
        .arg(
            Arg::with_name("key")
                .help("Key of the transaction to sign")
                .short("k")
                .long("key")
                .takes_value(true)
                .value_name("KEY")
                .conflicts_with("message")
                .required_unless("message"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_balance` adds a balance command to the `App`.
fn add_balance(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("balance")
        .about("Returns the balance of an account")
        .arg(
            Arg::with_name("address")
                .help("Account address")
                .takes_value(true)
                .value_name("ADDRESS")
                .required(true),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_send` adds a send command to the `App`.
fn add_send(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("send")
        .about("Send an amount from an address to an other")
        .arg(
            Arg::with_name("from")
                .help("Address of the sending account")
                .long("from")
                .takes_value(true)
                .value_name("FROM")
                .required(true),
        )
        .arg(
            Arg::with_name("to")
                .help("Address of the receiving account")
                .long("to")
                .takes_value(true)
                .value_name("TO")
                .required(true),
        )
        .arg(
            Arg::with_name("amount")
                .help("Amount to send")
                .long("amount")
                .takes_value(true)
                .value_name("AMOUNT")
                .required(true),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_mine` adds a mine command to the `App`.
fn add_mine(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("mine")
        .about("Mines a transaction")
        .arg(
            Arg::with_name("key")
                .help("Transaction key")
                .short("k")
                .long("key")
                .takes_value(true)
                .value_name("KEY")
                .conflicts_with("value"),
        )
        .arg(
            Arg::with_name("value")
                .help("Transaction value")
                .short("v")
                .long("value")
                .takes_value(true)
                .value_name("VALUE")
                .conflicts_with("key"),
        )
        .arg(
            Arg::with_name("format")
                .help("Transaction format")
                .short("F")
                .long("format")
                .takes_value(true)
                .value_name("FORMAT")
                .possible_values(&["json", "hex"])
                .default_value("hex")
                .requires("value"),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `CliClient` is the type of the CLI client.
pub struct CliClient {}

impl CliClient {
    /// `CLI_NAME` is the CLI client app name.
    pub const CLI_NAME: &'static str = "alsac";

    /// `CLI_ABOUT` is the CLI client app description.
    pub const CLI_ABOUT: &'static str = "Alsacoin client";

    /// `app` returns the `CliClient` clap `App`.
    pub fn app() -> App<'static, 'static> {
        let mut app = common::app(Self::CLI_NAME, Self::CLI_ABOUT);

        app = add_hash(app);
        app = add_sign(app);
        app = add_balance(app);
        app = add_send(app);
        app = add_mine(app);

        app = add_wallet(app);
        app = add_account(app);
        app = add_transaction(app);

        app = add_node(app);

        app = add_consensus_state(app);
        app = add_consensus_message(app);

        app = add_store(app);

        app
    }

    /// `args` returns the `CliClient` clap `ArgMatches`.
    pub fn args() -> ArgMatches<'static> {
        CliClient::app().get_matches()
    }

    /// `init` inits the `CliClient` environment.
    pub fn init() -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `reset` resets the `CliClient` environment.
    pub fn reset() -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `run` runs the `CliClient` application.
    pub fn run() -> Result<()> {
        //CliClient::init()?;

        let matches = CliClient::args();
        println!("{} matches: {:?}", Self::CLI_NAME, matches);

        Ok(())
    }
}
