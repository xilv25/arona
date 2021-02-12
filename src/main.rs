use bluearch_recruitment::banner::{Banner, BannerBuilder};
use bluearch_recruitment::gacha::{GachaBuilder, Rarity, Recruitment as RecruitmentTrait};
use bluearch_recruitment::i18n::Language;
use bluearch_recruitment::student::Student;
use dotenv::dotenv;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::{ColorType, ImageEncoder, Rgba, RgbaImage};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::macros::{command, group, help};
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::utils::Colour;
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::Cursor;
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
#[group]
#[commands(ping, source)]
struct General;

#[group]
#[commands(roll, banner, roll10)]
struct Recruitment;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

const STUDENTS_JSON: &str = include_str!("../data/students.json");
const CDN_URL: &str = "https://rerollcdn.com/BlueArchive";
const BOT_SOURCE: &str = "https://git.paoda.moe/paoda/arona";
const GACHA_SOURCE: &str = "https://github.com/Paoda/bluearch-recruitment";
const IMG_SOURCE: &str = "https://thearchive.gg";
const BANNER_IMG_URL: &str = "https://static.wikia.nocookie.net/blue-archive/images/e/e0/Gacha_Banner_01.png/revision/latest/";
const BLUE_ARCHIVE_BLUE: Colour = Colour::from_rgb(0, 215, 251);

type Cache = Mutex<HashMap<String, RgbaImage>>;

lazy_static! {
    static ref STUDENTS: Vec<Student> = serde_json::from_str(STUDENTS_JSON).unwrap();
    static ref BANNER: Banner = create_banner();
    static ref CACHE: Cache = Mutex::new(HashMap::new());
}

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

#[command]
#[aliases(pull)]
async fn roll(ctx: &Context, msg: &Message) -> CommandResult {
    let author_name = format!("{}#{}", msg.author.name, msg.author.discriminator);
    info!("{} requested a single roll", author_name);

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
    let rarity_colour = get_rarity_colour(student.rarity);

    let rarity_str = match student.rarity {
        Rarity::One => ":star:",
        Rarity::Two => ":star::star:",
        Rarity::Three => ":star::star::star:",
    };

    channel
        .send_message(ctx, |m| {
            m.embed(|embed| {
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
#[aliases(tenroll)]
async fn roll10(ctx: &Context, msg: &Message) -> CommandResult {
    let author_name = format!("{}#{}", msg.author.name, msg.author.discriminator);
    info!("{} requested a ten roll", author_name);
    let channel = msg.channel_id;

    let typing = channel.start_typing(&ctx.http)?;

    const RESIZE_WIDTH: u32 = 202; // OG: 404 (2020-02-11)
    const RESIZE_HEIGHT: u32 = 228; // OG: 456 (2020-02-11)
    const IMG_WIDTH: u32 = RESIZE_WIDTH * 5;
    const IMG_HEIGHT: u32 = RESIZE_HEIGHT * 2;

    let mut collage = RgbaImage::new(IMG_WIDTH, IMG_HEIGHT);
    let mut images: Vec<RgbaImage> = Vec::with_capacity(10);

    let students = BANNER.roll10();
    let mut max_rarity = Rarity::One;

    let start = Instant::now();
    for student in students.iter() {
        let eng_name = student.name.get(Language::English).unwrap();
        let url_name = if eng_name == "Junko" {
            "Zunko"
        } else {
            &eng_name
        };

        max_rarity = max_rarity.max(student.rarity);

        let img_url = format!("{}/Characters/{}.png", CDN_URL, url_name);
        let image = get_image_from_url(&img_url, RESIZE_WIDTH, RESIZE_HEIGHT).await;

        images.push(image);
    }
    let elapsed_ms = (Instant::now() - start).as_millis();
    info!("10-roll, DL, and resize took {}ms", elapsed_ms);

    let start = Instant::now();
    for x in (0..IMG_WIDTH).step_by(RESIZE_WIDTH as usize) {
        let index: usize = (((IMG_WIDTH - x) / RESIZE_WIDTH) - 1) as usize;

        // Top Image
        image::imageops::overlay(&mut collage, &images[index], x, 0);

        // Bottom Image
        image::imageops::overlay(&mut collage, &images[index + 5], x, RESIZE_HEIGHT);
    }
    let elapsed_ms = (Instant::now() - start).as_millis();
    info!("Collage Build took {}ms", elapsed_ms);

    let mut jpeg = Vec::new();
    let encoder = JpegEncoder::new(&mut jpeg);

    let write_result = encoder.write_image(&collage, IMG_WIDTH, IMG_HEIGHT, ColorType::Rgba8);

    if let Err(err) = write_result {
        error!("Failed to Encode JPEG: {:?}", err);
        msg.reply(
            ctx,
            "アロナ failed to perform your 10-roll. Please try again",
        )
        .await?;
        return Ok(());
    }

    let files = vec![(jpeg.as_slice(), "result.jpeg")];
    let _ = typing.stop();
    channel
        .send_files(ctx, files, |m| {
            m.embed(|embed| {
                embed
                    .title(format!("{} 10-roll", BANNER.name))
                    .description(BANNER.name.get(Language::English).unwrap())
                    .attachment("result.jpeg")
                    .colour(get_rarity_colour(max_rarity))
            })
        })
        .await?;
    Ok(())
}

async fn get_image_from_url(url: &str, width: u32, height: u32) -> RgbaImage {
    if let Some(img) = check_cache(url) {
        info!("Cache Hit for {}", url);
        return img;
    }

    info!("Downloading {}", url);
    match reqwest::get(url).await {
        Ok(resp) => match resp.bytes().await {
            Ok(bytes) => match ImageReader::new(Cursor::new(bytes)).with_guessed_format() {
                Ok(img_reader) => match img_reader.decode() {
                    Ok(dynamic_img) => {
                        info!("Successfully decoded image from {}", url);
                        let img = image::imageops::resize(
                            &dynamic_img,
                            width,
                            height,
                            FilterType::Nearest,
                        );

                        add_to_cache(url, &img);
                        img
                    }
                    Err(err) => {
                        warn!("Decoding Error: {}", err);
                        generate_default_img(width, height)
                    }
                },
                Err(err) => {
                    error!("Unexpected IO Error Occurred: {}", err);
                    // We can recover here, but maybe it's worth panicking here?
                    generate_default_img(width, height)
                }
            },

            Err(err) => {
                warn!("Response Parse failed: {:}?", err);
                generate_default_img(width, height)
            }
        },
        Err(err) => {
            warn!("Download failed: {:?}", err);
            generate_default_img(width, height)
        }
    }
}

fn get_rarity_colour(rarity: Rarity) -> Colour {
    match rarity {
        Rarity::One => Colour::from_rgb(227, 234, 240),
        Rarity::Two => Colour::from_rgb(255, 248, 124),
        Rarity::Three => Colour::from_rgb(253, 198, 229),
    }
}

fn generate_default_img(width: u32, height: u32) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    for x in 0..height {
        for y in 0..width {
            img.put_pixel(x, y, Rgba([160, 32, 240, 1]));
        }
    }

    img
}

fn check_cache(url: &str) -> Option<RgbaImage> {
    if let Ok(lock) = CACHE.lock() {
        lock.get(url).cloned()
    } else {
        None
    }
}

fn add_to_cache(url: &str, img: &RgbaImage) {
    if let Ok(ref mut lock) = CACHE.lock() {
        lock.insert(url.to_string(), img.clone());
    }
}

#[command]
async fn banner(ctx: &Context, msg: &Message) -> CommandResult {
    let author_name = format!("{}#{}", msg.author.name, msg.author.discriminator);
    info!("{} requested banner information", author_name);

    let channel = msg.channel_id;
    let banner_eng = BANNER.name.get(Language::English).unwrap();

    channel
        .send_message(ctx, |m| {
            m.embed(|embed| {
                embed
                    .image(BANNER_IMG_URL)
                    .title(BANNER.name.clone())
                    .description(banner_eng)
                    .colour(BLUE_ARCHIVE_BLUE)
            })
        })
        .await?;

    Ok(())
}

#[command]
#[aliases(github, code, dev)]
async fn source(ctx: &Context, msg: &Message) -> CommandResult {
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

pub fn create_banner() -> Banner {
    let pool: Vec<Student> = STUDENTS
        .iter()
        .filter(|student| student.name != "ノゾミ")
        .cloned()
        .collect();

    let sparkable: Vec<Student> = pool
        .iter()
        .filter(|student| student.name == "ホシノ" || student.name == "シロコ")
        .cloned()
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
