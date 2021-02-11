use bluearch_recruitment::banner::{Banner, BannerBuilder};
use bluearch_recruitment::gacha::{GachaBuilder, Recruitment};
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
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(roll, roll10, banner)]
struct RecruitmentCommands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

const STUDENTS_JSON: &str = include_str!("../data/students.json");

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

    let student = BANNER.roll();

    msg.reply(
        ctx,
        format!(
            "You Pulled: {} {}",
            student.name.get(Language::English).unwrap(),
            student.rarity
        ),
    )
    .await?;

    Ok(())
}

#[command]
async fn roll10(ctx: &Context, msg: &Message) -> CommandResult {
    let author_name = format!("{}#{}", msg.author.name, msg.author.discriminator);
    info!("{} Requested a 10-roll", author_name);

    let students = BANNER.roll10();
    let mut response = "You Pulled:\n".to_string();

    for student in students.iter() {
        response += &format!(
            "{} {}\n",
            student.name.get(Language::English).unwrap(),
            student.rarity
        );
    }

    msg.reply(ctx, response).await?;
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
