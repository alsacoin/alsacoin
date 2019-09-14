//! # Client
//!
//! `client` contains the CLI client type and functions.

use crate::common;
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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_count` adds a count command to the `App`.
/// TODO: count -from -to -skip
fn add_count(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("count");

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_query` adds a query command to the `App`.
/// TODO: query -from -to -count -skip
fn add_query(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("query");

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_insert` adds a insert command to the `App`.
/// TODO: insert -key -value
fn add_insert(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("insert");

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_update` adds a update command to the `App`.
/// TODO: update -key -value
fn add_update(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("update");

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_cleanup` adds a cleanup command to the `App`.
fn add_cleanup(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("cleanup").about("Cleanup").arg(
        Arg::with_name("to")
            .help("Timestamp to clean up to")
            .takes_value(true)
            .value_name("TO"),
    );

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_clean` adds a clean command to the `App`.
fn add_clean(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("clean");

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_storable` adds the storable commands to the `App`.
fn add_storable(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut app = add_wallet_create(app);
    app = add_lookup(app);
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
                .value_name("KEY")
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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_wallet_create` adds a create command to the wallet subcommand.
fn add_wallet_create(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_wallet` adds a wallet command to the `App`.
fn add_wallet(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("wallet").about("Wallet operations");

    cmd = add_wallet_create(cmd);
    cmd = add_storable(cmd);

    app.subcommand(cmd)
}

/// `add_account_create` adds a create command to the account subcommand.
fn add_account_create(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_account` adds a account command to the `App`.
fn add_account(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("account").about("Account operations");

    cmd = add_account_create(cmd);
    cmd = add_storable(cmd);

    app.subcommand(cmd)
}

/// `add_transaction_create` adds a create command to the transaction subcommand.
fn add_transaction_create(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_transaction` adds a transaction command to the `App`.
fn add_transaction(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("transaction").about("Transaction operations");

    cmd = add_transaction_create(cmd);
    cmd = add_fetch(cmd);
    cmd = add_push(cmd);
    cmd = add_storable(cmd);

    app.subcommand(cmd)
}

/// `add_node_create` adds a create command to the node subcommand.
fn add_node_create(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
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

/// `add_consensus_state_create` adds a create command to the consensus-state subcommand.
fn add_consensus_state_create(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_consensus_state` adds a consensus-state command to the `App`.
fn add_consensus_state(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("consensus-state").about("Consensus state operations");

    cmd = add_consensus_state_create(cmd);
    cmd = add_storable(cmd);

    app.subcommand(cmd)
}

/// `add_consensus_message_create` adds a create command to the consensus-message subcommand.
fn add_consensus_message_create(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_consensus_message` adds a consensus-message command to the `App`.
fn add_consensus_message(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("consensus-message").about("Consensus message operations");

    cmd = add_consensus_message_create(cmd);
    cmd = add_storable(cmd);

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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

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

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `CliClient` is the type of the CLI client.
pub struct CliClient {}

impl CliClient {
    /// `CLI_CLIENT_NAME` is the CLI client app name.
    pub const CLI_CLIENT_NAME: &'static str = "alsac";

    /// `CLI_CLIENT_ABOUT` is the CLI client app description.
    pub const CLI_CLIENT_ABOUT: &'static str = "Alsacoin client";

    /// `app` returns the `CliClient` clap `App`.
    pub fn app() -> App<'static, 'static> {
        let mut app = common::app(Self::CLI_CLIENT_NAME, Self::CLI_CLIENT_ABOUT);

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
}
