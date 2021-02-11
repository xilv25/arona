use bluearch_recruitment::banner::{Banner, BannerBuilder};
use bluearch_recruitment::gacha::{GachaBuilder, Rarity, Recruitment};
use bluearch_recruitment::i18n::Language;
use bluearch_recruitment::student::Student;
use dotenv::dotenv;
use lazy_static::lazy_static;
use log::{error, info};
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::utils::Colour;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(roll, banner)]
struct RecruitmentCommands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

const STUDENTS_JSON: &str = include_str!("../data/students.json");
const CDN_URL: &str = "https://rerollcdn.com/BlueArchive";

lazy_static! {
    static ref STUDENTS: Vec<Student> = serde_json::from_str(STUDENTS_JSON).unwrap();
    static ref BANNER: Banner = create_banner();
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "!"
        .group(&GENERAL_GROUP)
        .group(&RECRUITMENTCOMMANDS_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN was not set.");
    info!("アロナ has started with Bot Token: {}", &token);

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Failed to create Serenity Client");

    info!("Successfully created the Serenity Client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        error!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let author_name = format!("{}#{}", msg.author.name, msg.author.discriminator);
    info!("Ping received from: {}", author_name);

    let now = SystemTime::now();

    match now.duration_since(UNIX_EPOCH) {
        Ok(now_timestamp) => {
            let diff = now_timestamp.as_millis() - msg.timestamp.timestamp_millis() as u128;
            msg.reply(ctx, format!("Pong! (Response: {}ms)", diff))
                .await?;
        }
        Err(_) => {
            msg.reply(ctx, "Pong! (Response: ??ms)").await?;
        }
    }

    Ok(())
}

#[command]
async fn roll(ctx: &Context, msg: &Message) -> CommandResult {
    let author_name = format!("{}#{}", msg.author.name, msg.author.discriminator);
    info!("{} Requested a single roll", author_name);

    let channel = msg.channel_id;
    let student = BANNER.roll();

    let eng_name = student.name.get(Language::English).unwrap();
    let url_name = if eng_name == "Junko" {
        "Zunko"
    } else {
        &eng_name
    };

    let img_url = format!("{}/Characters/{}.png", CDN_URL, url_name);
    let title_url = format!("https://www.thearchive.gg/characters/{}", url_name);
    let icon_url = format!("{}/Icons/icon-brand.png", CDN_URL);

    let rarity_str = match student.rarity {
        Rarity::One => ":star:",
        Rarity::Two => ":star::star:",
        Rarity::Three => ":star::star::star:",
    };

    let rarity_colour = match student.rarity {
        Rarity::One => Colour::from_rgb(227, 234, 240),
        Rarity::Two => Colour::from_rgb(255, 248, 124),
        Rarity::Three => Colour::from_rgb(253, 198, 229),
    };

    channel
        .send_message(ctx, |m| {
            m.reference_message(msg).embed(|embed| {
                embed
                    .image(img_url)
                    .title(format!("{}", student.name))
                    .description(format!("{}\t{}", eng_name, rarity_str))
                    .url(title_url)
                    .footer(|footer| {
                        footer
                            .icon_url(icon_url)
                            .text("Image Source: https://thearchive.gg")
                    })
                    .colour(rarity_colour)
            })
        })
        .await?;

    Ok(())
}

#[command]
async fn banner(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, format!("Current Banner: {}", BANNER.name))
        .await?;
    Ok(())
}

pub fn create_banner() -> Banner {
    let pool: Vec<Student> = STUDENTS
        .iter()
        .filter(|student| student.name != "ノゾミ")
        .map(|student| student.clone())
        .collect();

    let sparkable: Vec<Student> = pool
        .iter()
        .filter(|student| student.name == "ホシノ" || student.name == "シロコ")
        .map(|student| student.clone())
        .collect();

    let gacha = GachaBuilder::new(79.0, 18.5, 2.5)
        .with_pool(pool)
        .with_priority(&sparkable, 0.7)
        .finish()
        .unwrap();

    BannerBuilder::new("ピックアップ募集")
        .with_gacha(&gacha)
        .with_name_translation(Language::English, "Rate-Up Recruitment")
        .with_sparkable_students(&sparkable)
        .finish()
        .unwrap()
}
