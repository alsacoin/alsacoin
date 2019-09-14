//! # Client
//!
//! `client` contains the CLI client type and functions.

use crate::common;
use clap::{App, Arg, ArgMatches, SubCommand};

/// `add_lookup` adds a lookup command to the `App`.
/// TODO: lookup [-key]
fn add_lookup(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("lookup");

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_get` adds a get command to the `App`.
/// TODO: get [-key]
fn add_get(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("get");

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

/// `add_insert_batch` adds a insert command to the `App`.
/// TODO: insert-batch [-values | -file]
fn add_insert_batch(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("insert-batch");

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
/// TODO: remove [-key]
fn add_remove(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("remove");

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_remove_batch` adds a remove command to the `App`.
/// TODO: remove-batch [-keys | -file]
fn add_remove_batch(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("remove-batch");

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_cleanup` adds a cleanup command to the `App`.
/// TODO: cleanup -to
fn add_cleanup(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("cleanup");

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_clean` adds a clean command to the `App`.
/// TODO: clean
fn add_clean(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("clean");

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_fetch` adds a fetch command to the `App`.
/// TODO: fetch [-node] [-count | -keys]
fn add_fetch(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("fetch");

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_push` adds a push command to the `App`.
/// TODO: push [-node] [-key | -value]
fn add_push(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("push");

    cmd = common::add_config_option(cmd);
    cmd = common::add_verbose_option(cmd);

    app.subcommand(cmd)
}

/// `add_wallet` adds a wallet command to the `App`.
/// TODO: wallet create + storable subcommands
fn add_wallet(app: App<'static, 'static>) -> App<'static, 'static> {
    let cmd = SubCommand::with_name("wallet");

    app.subcommand(cmd)
}

/// `add_account` adds a account command to the `App`.
/// TODO: account create + storable subcommands
fn add_account(app: App<'static, 'static>) -> App<'static, 'static> {
    let cmd = SubCommand::with_name("account"); // TODO

    app.subcommand(cmd)
}

/// `add_transaction` adds a transaction command to the `App`.
/// TODO: transaction create + storable subcommands
fn add_transaction(app: App<'static, 'static>) -> App<'static, 'static> {
    let cmd = SubCommand::with_name("transaction"); // TODO

    app.subcommand(cmd)
}

/// `add_node` adds a node command to the `App`.
/// TODO: node create + storable subcommands
fn add_node(app: App<'static, 'static>) -> App<'static, 'static> {
    let cmd = SubCommand::with_name("node"); // TODO

    app.subcommand(cmd)
}

/// `add_consensus_state` adds a consensus-state command to the `App`.
/// TODO: consensus-state storable subcommands
fn add_consensus_state(app: App<'static, 'static>) -> App<'static, 'static> {
    let cmd = SubCommand::with_name("consensus-state"); // TODO

    app.subcommand(cmd)
}

/// `add_consensus_message` adds a consensus-message command to the `App`.
/// TODO: consensus-message storable subcommands
fn add_consensus_message(app: App<'static, 'static>) -> App<'static, 'static> {
    let cmd = SubCommand::with_name("consensus-message"); // TODO

    app.subcommand(cmd)
}

/// `add_store` adds a store command to the `App`.
/// TODO: store [import -file | export -file | size [-keys | -values | -all] | clean]
fn add_store(app: App<'static, 'static>) -> App<'static, 'static> {
    let cmd = SubCommand::with_name("store"); // TODO

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
