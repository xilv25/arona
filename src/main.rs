use dotenv::dotenv;
use log::{debug, error, info};
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::macros::{command, group, help};
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use std::collections::HashSet;
use std::env;

#[group]
#[commands(ping, source)]
struct General;

#[group]
#[commands(roll, banner, roll10)]
struct Recruitment;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "!"
        .group(&GENERAL_GROUP)
        .group(&RECRUITMENT_GROUP)
        .help(&MY_HELP);

    debug!("Initialized the StandardFramework struct");

    // Login with a bot token from the environment
    let mut client = match env::var("DISCORD_DEV_BOT_TOKEN") {
        Ok(token) => {
            debug!("DISCORD_ENV_BOT_TOKEN is present. Running as アロナDev");
            let client = Client::builder(&token)
                .event_handler(Handler)
                .framework(framework)
                .await
                .expect("Failed to create Serenity Client");
            info!("アロナDev Client has begun with Token: {}", &token);
            client
        }
        Err(_) => {
            debug!("DISCORD_ENV_BOT_TOKEN is not present. Running as アロナ");
            let token = env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN was not set.");
            let client = Client::builder(&token)
                .event_handler(Handler)
                .framework(framework)
                .await
                .expect("Failed to create Serenity Client");

            info!("アロナ Client has begun with Token: {}", &token);
            client
        }
    };

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        error!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
#[aliases(response)]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    arona::general::ping(ctx, msg).await
}

#[command]
#[aliases(pull)]
async fn roll(ctx: &Context, msg: &Message) -> CommandResult {
    arona::recruitment::roll(ctx, msg).await
}

#[command]
#[aliases(tenroll)]
async fn roll10(ctx: &Context, msg: &Message) -> CommandResult {
    arona::recruitment::roll10(ctx, msg).await
}

#[command]
async fn banner(ctx: &Context, msg: &Message) -> CommandResult {
    arona::recruitment::banner(ctx, msg).await
}

#[command]
#[aliases(github, code, dev)]
async fn source(ctx: &Context, msg: &Message) -> CommandResult {
    arona::general::source(ctx, msg).await
}

#[help]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let author_name = format!("{}#{}", msg.author.name, msg.author.discriminator);
    info!("{} asked for help", author_name);
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}
