//! # Client
//!
//! `client` contains the CLI client type and functions.

use crate::common;
use clap::{App, Arg, ArgMatches, SubCommand};

/// `add_key_option` adds a key option to the `SubCommand`.
fn add_key_option(_subcmd: SubCommand<'static>) -> SubCommand<'static> {
    // TODO
    unreachable!()
}

/// `add_keys_option` adds a keys option to the `SubCommand`.
fn add_keys_option(_subcmd: SubCommand<'static>) -> SubCommand<'static> {
    // TODO
    unreachable!()
}

/// `add_value_option` adds a value option to the `SubCommand`.
fn add_value_option(_subcmd: SubCommand<'static>) -> SubCommand<'static> {
    // TODO
    unreachable!()
}

/// `add_values_option` adds a values option to the `SubCommand`.
fn add_values_option(_subcmd: SubCommand<'static>) -> SubCommand<'static> {
    // TODO
    unreachable!()
}

/// `add_from_option` adds a from option to the `SubCommand`.
fn add_from_option(_subcmd: SubCommand<'static>) -> SubCommand<'static> {
    // TODO
    unreachable!()
}

/// `add_to_option` adds a to option to the `SubCommand`.
fn add_to_option(_subcmd: SubCommand<'static>) -> SubCommand<'static> {
    // TODO
    unreachable!()
}

/// `add_count_option` adds a count option to the `SubCommand`.
fn add_count_option(_subcmd: SubCommand<'static>) -> SubCommand<'static> {
    // TODO
    unreachable!()
}

/// `add_skip_option` adds a skip option skip the `SubCommand`.
fn add_skip_option(_subcmd: SubCommand<'static>) -> SubCommand<'static> {
    // TODO
    unreachable!()
}

/// `add_file_option` adds a file option file the `SubCommand`.
fn add_file_option(_subcmd: SubCommand<'static>) -> SubCommand<'static> {
    // TODO
    unreachable!()
}

/// `add_address_option` adds a address option address the `SubCommand`.
fn add_address_option(_subcmd: SubCommand<'static>) -> SubCommand<'static> {
    // TODO
    unreachable!()
}

/// `add_message_option` adds a message option message the `SubCommand`.
fn add_message_option(_subcmd: SubCommand<'static>) -> SubCommand<'static> {
    // TODO
    unreachable!()
}

/// `add_node_option` adds a node option node the `SubCommand`.
fn add_node_option(_subcmd: SubCommand<'static>) -> SubCommand<'static> {
    // TODO
    unreachable!()
}

/// `add_lookup` adds a lookup command to the `App`.
/// TODO: lookup [-key]
fn add_lookup(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_get` adds a get command to the `App`.
/// TODO: get [-key]
fn add_get(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_count` adds a count command to the `App`.
/// TODO: count -from -to -skip
fn add_count(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_query` adds a query command to the `App`.
/// TODO: query -from -to -count -skip
fn add_query(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_insert` adds a insert command to the `App`.
/// TODO: insert -key -value
fn add_insert(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_insert_batch` adds a insert command to the `App`.
/// TODO: insert-batch [-values | -file]
fn add_insert_batch(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_update` adds a update command to the `App`.
/// TODO: update -key -value
fn add_update(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_remove` adds a remove command to the `App`.
/// TODO: remove [-key]
fn add_remove(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_remove_batch` adds a remove command to the `App`.
/// TODO: remove-batch [-keys | -file]
fn add_remove_batch(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_cleanup` adds a cleanup command to the `App`.
/// TODO: cleanup -to
fn add_cleanup(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_clean` adds a clean command to the `App`.
/// TODO: clean
fn add_clean(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_fetch` adds a fetch command to the `App`.
/// TODO: fetch [-node] [-count | -keys]
fn add_fetch(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_push` adds a push command to the `App`.
/// TODO: push [-node] [-key | -value]
fn add_push(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_wallet` adds a wallet command to the `App`.
/// TODO: wallet create + storable subcommands
fn add_wallet(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_account` adds a account command to the `App`.
/// TODO: account create + storable subcommands
fn add_account(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_transaction` adds a transaction command to the `App`.
/// TODO: transaction create + storable subcommands
fn add_transaction(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_node` adds a node command to the `App`.
/// TODO: node create + storable subcommands
fn add_node(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_consensus_state` adds a consensus-state command to the `App`.
/// TODO: consensus-state storable subcommands
fn add_consensus_state(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_consensus_message` adds a consensus-message command to the `App`.
/// TODO: consensus-message storable subcommands
fn add_consensus_message(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_store` adds a store command to the `App`.
/// TODO: store [import -file | export -file | size [-keys | -values | -all] | clean]
fn add_store(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_hash` adds a hash command to the `App`.
/// TODO: hash [-message | -file]
fn add_hash(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_sign` adds a sign command to the `App`.
/// TODO: sign -address [-message | -key of the tx]
fn add_sign(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_send` adds a send command to the `App`.
/// TODO: send -amount -to
fn add_send(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_mine` adds a mine command to the `App`.
/// TODO: mine [-key | -value]
fn add_mine(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
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
