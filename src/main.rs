#![deny(rust_2018_idioms)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::collections::HashSet;

use dotenv::dotenv;
use serenity::client::Context;
use serenity::framework::standard::macros::{group, help};
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, DispatchError, HelpOptions, StandardFramework,
};
use serenity::model::prelude::*;

mod commands;
mod db;
mod dbl;
mod error;
#[rustfmt::skip]
mod schema;
mod tools;
mod util;

use commands::basic::*;
use commands::game::*;
use commands::mods::*;
use commands::subs::*;
use util::*;

const DATABASE_URL: &str = "DATABASE_URL";
const DISCORD_BOT_TOKEN: &str = "DISCORD_BOT_TOKEN";
const DBL_TOKEN: &str = "DBL_TOKEN";
const DBL_OVERRIDE_BOT_ID: &str = "DBL_OVERRIDE_BOT_ID";
const MODIO_HOST: &str = "MODIO_HOST";
const MODIO_API_KEY: &str = "MODIO_API_KEY";
const MODIO_TOKEN: &str = "MODIO_TOKEN";

const DEFAULT_MODIO_HOST: &str = "https://api.mod.io/v1";

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn try_main() -> CliResult {
    dotenv().ok();
    env_logger::init();

    if tools::tools() {
        return Ok(());
    }

    let (mut client, modio, mut rt) = util::initialize()?;

    rt.spawn(task(&client, modio.clone(), rt.executor()));

    let (bot, owners) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => (info.id, vec![info.owner.id].into_iter().collect()),
        Err(e) => panic!("Couldn't get application info: {}", e),
    };

    if let Ok(token) = util::var(DBL_TOKEN) {
        log::info!("Spawning DBL task");
        let bot = *bot.as_u64();
        let cache = client.cache_and_http.cache.clone();
        rt.spawn(dbl::task(bot, cache, &token, rt.executor())?);
    }

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix("~")
                    .dynamic_prefix(util::dynamic_prefix)
                    .on_mention(Some(bot))
                    .owners(owners)
            })
            .bucket("simple", |b| b.delay(1))
            .before(|_, msg, _| {
                log::debug!("cmd: {:?}: {:?}: {}", msg.guild_id, msg.author, msg.content);
                true
            })
            .group(&OWNER_GROUP)
            .group(if dbl::is_dbl_enabled() { &with_vote::GENERAL_GROUP } else { &GENERAL_GROUP })
            .group(&MODIO_GROUP)
            .on_dispatch_error(|ctx, msg, error| match error {
                DispatchError::NotEnoughArguments { .. } => {
                    let _ = msg.channel_id.say(ctx, "Not enough arguments.");
                }
                DispatchError::LackingPermissions(_) => {
                    let _ = msg
                        .channel_id
                        .say(ctx, "You have insufficient rights for this command, you need the `MANAGE_CHANNELS` permission.");
                }
                DispatchError::Ratelimited(_) => {
                    let _ = msg.channel_id.say(ctx, "Try again in 1 second.");
                }
                e => eprintln!("Dispatch error: {:?}", e),
            })
            .help(&HELP),
    );
    client.start()?;
    Ok(())
}

group!({
    name: "Owner",
    options: {},
    commands: [servers],
});

group!({
    name: "General",
    options: {},
    commands: [about, prefix, invite, guide],
});

group!({
    name: "modio",
    options: {},
    commands: [list_games, game, list_mods, mod_info, popular, subscriptions, subscribe, unsubscribe],
});

mod with_vote {
    use super::*;

    group!({
        name: "General",
        options: {},
        commands: [about, prefix, invite, guide, vote],
    });
}

#[help]
fn help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}
