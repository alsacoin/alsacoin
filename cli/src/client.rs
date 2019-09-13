//! # Client
//!
//! `client` contains the CLI client type and functions.

use crate::common;
use clap::{App, Arg, ArgMatches, SubCommand};

/// `add_sign` adds a sign command to the `App`.
fn add_sign(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_send` adds a send command to the `App`.
fn add_send(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_lookup` adds a lookup command to the `App`.
fn add_lookup(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_get` adds a get command to the `App`.
fn add_get(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_count` adds a count command to the `App`.
fn add_count(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_query` adds a query command to the `App`.
fn add_query(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_insert` adds a insert command to the `App`.
fn add_insert(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_insert_batch` adds a insert command to the `App`.
fn add_insert_batch(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_create` adds a create command to the `App`.
fn add_create(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_update` adds a update command to the `App`.
fn add_update(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_remove` adds a remove command to the `App`.
fn add_remove(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_remove_batch` adds a remove command to the `App`.
fn add_remove_batch(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_cleanup` adds a cleanup command to the `App`.
fn add_cleanup(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_clean` adds a clean command to the `App`.
fn add_clean(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_fetch` adds a fetch command to the `App`.
fn add_fetch(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_push` adds a push command to the `App`.
fn add_push(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_wallet` adds a wallet command to the `App`.
fn add_wallet(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_account` adds a account command to the `App`.
fn add_account(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_transaction` adds a transaction command to the `App`.
fn add_transaction(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_node` adds a node command to the `App`.
fn add_node(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_consensus_state` adds a consensus-state command to the `App`.
fn add_consensus_state(_app: App<'static, 'static>) -> App<'static, 'static> {
    // TODO
    unreachable!()
}

/// `add_consensus_message` adds a consensus-message command to the `App`.
fn add_consensus_message(_app: App<'static, 'static>) -> App<'static, 'static> {
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

        app = add_sign(app);
        app = add_send(app);

        app = add_wallet(app);
        app = add_account(app);
        app = add_transaction(app);

        app = add_node(app);

        app = add_consensus_state(app);
        app = add_consensus_message(app);

        app
    }

    /// `args` returns the `CliClient` clap `ArgMatches`.
    pub fn args() -> ArgMatches<'static> {
        CliClient::app().get_matches()
    }
}
