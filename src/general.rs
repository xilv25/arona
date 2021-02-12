use log::{info, warn};
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::utils::Colour;
use std::time::{SystemTime, UNIX_EPOCH};

const BOT_SOURCE: &str = "https://github.com/Paoda/arona";
const GACHA_SOURCE: &str = "https://github.com/Paoda/bluearch-recruitment";
const IMG_SOURCE: &str = "https://thearchive.gg";
pub const BLUE_ARCHIVE_BLUE: Colour = Colour::from_rgb(0, 215, 251);

pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let author_name = format!("{}#{}", msg.author.name, msg.author.discriminator);
    info!("Ping requested from {}", author_name);

    let now = SystemTime::now();

    match now.duration_since(UNIX_EPOCH) {
        Ok(now_timestamp) => {
            let diff = now_timestamp.as_millis() - msg.timestamp.timestamp_millis() as u128;
            info!("It took {}ms to receive {}'s ping", diff, author_name);

            msg.reply(ctx, format!("Pong! (Response: {}ms)", diff))
                .await?;
        }
        Err(_) => {
            warn!("Failed to calculate UNIX Timestamp");
            msg.reply(ctx, "Pong! (Response: ??ms)").await?;
        }
    }

    Ok(())
}

pub async fn source(ctx: &Context, msg: &Message) -> CommandResult {
    let author_name = format!("{}#{}", msg.author.name, msg.author.discriminator);
    info!("{} requested bot / gacha / image sources", author_name);
    let channel = msg.channel_id;

    channel
        .send_message(ctx, |m| {
            m.embed(|embed| {
                embed
                    .field("Bot Source", BOT_SOURCE, false)
                    .field("Gacha Source", GACHA_SOURCE, false)
                    .field("Image Source", IMG_SOURCE, false)
                    .colour(BLUE_ARCHIVE_BLUE)
            })
        })
        .await?;

    Ok(())
}
