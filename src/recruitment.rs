use crate::general::BLUE_ARCHIVE_BLUE;
use crate::image::get_image_from_url;
use blue_gacha::banner::{Banner, BannerBuilder};
use blue_gacha::gacha::Recruitment as RecruitmentTrait;
use blue_gacha::gacha::{GachaBuilder, Rarity};
use blue_gacha::i18n::Language;
use blue_gacha::student::Student;
use image::jpeg::JpegEncoder;
use image::{ColorType, ImageEncoder, RgbaImage};
use lazy_static::lazy_static;
use log::{error, info};
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::utils::Colour;

use std::time::Instant;

const STUDENTS_JSON: &str = include_str!("../data/students.json");
const CDN_URL: &str = "https://rerollcdn.com/BlueArchive";
const BANNER_IMG_URL: &str = "https://pbs.twimg.com/media/EuzV9rdUUAAD2jx?format=jpg&name=large";
const THUMB_WIDTH: u32 = 202; // OG: 404 (2020-02-11) from https://thearchive.gg
const THUMB_HEIGHT: u32 = 228; // OG: 456 (2020-02-11) from https://thearchive.gg

const THREE_STAR_RATE: f32 = 2.5;
const TWO_STAR_RATE: f32 = 18.5;
const ONE_STAR_RATE: f32 = 79.0;

lazy_static! {
    static ref STUDENTS: Vec<Student> = serde_json::from_str(STUDENTS_JSON).unwrap();
    static ref BANNER: Banner = create_2021_02_25_izuna_banner();
}

pub async fn roll(ctx: &Context, msg: &Message) -> CommandResult {
    let author_name = format!("{}#{}", msg.author.name, msg.author.discriminator);
    info!("{} requested a single roll", author_name);

    let channel = msg.channel_id;
    let student = BANNER.roll();

    let eng_name = student.name.get(Language::English).unwrap();
    let url_name = &eng_name;

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

pub async fn roll10(ctx: &Context, msg: &Message) -> CommandResult {
    let author_name = format!("{}#{}", msg.author.name, msg.author.discriminator);
    info!("{} requested a ten roll", author_name);
    let channel = msg.channel_id;

    let typing = channel.start_typing(&ctx.http)?;

    const IMG_WIDTH: u32 = THUMB_WIDTH * 5;
    const IMG_HEIGHT: u32 = THUMB_HEIGHT * 2;

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
        let image = get_image_from_url(&img_url, THUMB_WIDTH, THUMB_HEIGHT).await;

        images.push(image);
    }
    let elapsed_ms = (Instant::now() - start).as_millis();
    info!("10-roll, DL, and resize took {}ms", elapsed_ms);

    let start = Instant::now();
    for x in (0..IMG_WIDTH).step_by(THUMB_WIDTH as usize) {
        let index: usize = (((IMG_WIDTH - x) / THUMB_WIDTH) - 1) as usize;

        // Top Image
        image::imageops::overlay(&mut collage, &images[index], x, 0);

        // Bottom Image
        image::imageops::overlay(&mut collage, &images[index + 5], x, THUMB_HEIGHT);
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

    let icon_url = format!("{}/Icons/icon-brand.png", CDN_URL);

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
                    .footer(|footer| {
                        footer
                            .icon_url(icon_url)
                            .text("Image Source: https://thearchive.gg")
                    })
            })
        })
        .await?;
    Ok(())
}

pub async fn banner(ctx: &Context, msg: &Message) -> CommandResult {
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

fn _create_2021_02_04_hoshino_shiroko_banner() -> Banner {
    let three_stars = "ヒナ, イオリ, ハルナ, イズミ, アル, スミレ, エイミ, カリン, ネル, マキ, ヒビキ, サヤ, シュン, シロコ, ホシノ, ヒフミ, ツルギ";
    let two_stars = "アカリ, ジュンコ, ムツキ, カヨコ, フウカ, ユウカ, アカネ, ハレ, ウタハ, チセ, ツバキ, セリカ, アヤネ, ハスミ, ハナエ, アイリ";
    let one_stars =
        "チナツ, ハルカ, ジュリ, コタマ, アスナ, コトリ, フィーナ, スズミ, シミコ, セリナ, ヨシミ";

    let mut pool = Vec::new();
    pool.extend(get_students(three_stars));
    pool.extend(get_students(two_stars));
    pool.extend(get_students(one_stars));

    let hoshino = pool
        .iter()
        .find(|student| student.name == "ホシノ")
        .cloned()
        .unwrap()
        .into_priority_student(0.7 / 2.0);

    let shiroko = pool
        .iter()
        .find(|student| student.name == "シロコ")
        .cloned()
        .unwrap()
        .into_priority_student(0.7 / 2.0);

    let sparkable = vec![hoshino.student().clone(), shiroko.student().clone()];
    let priority = vec![hoshino, shiroko];

    let gacha = GachaBuilder::new(79.0, 18.5, 2.5)
        .with_pool(pool)
        .with_priority(&priority)
        .finish()
        .unwrap();

    BannerBuilder::new("拝啓、はじまりの季節へ")
        .with_gacha(&gacha)
        .with_name_translation(Language::English, "Dear Sensei, a new season approaches")
        .with_sparkable_students(&sparkable)
        .finish()
        .unwrap()
}

fn _create_2021_02_11_mashiro_banner() -> Banner {
    let three_stars = "ヒナ, イオリ, ハルナ, イズミ, アル, スミレ, エイミ, カリン, ネル, マキ, ヒビキ, サヤ, シュン, シロコ, ホシノ, ヒフミ, ツルギ, マシロ";
    let two_stars = "アカリ, ジュンコ, ムツキ, カヨコ, フウカ, ユウカ, アカネ, ハレ, ウタハ, チセ, ツバキ, セリカ, アヤネ, ハスミ, ハナエ, アイリ";
    let one_stars =
        "チナツ, ハルカ, ジュリ, コタマ, アスナ, コトリ, フィーナ, スズミ, シミコ, セリナ, ヨシミ";

    let mut pool = Vec::new();
    pool.extend(get_students(three_stars));
    pool.extend(get_students(two_stars));
    pool.extend(get_students(one_stars));

    let mashiro = pool
        .iter()
        .find(|student| student.name == "マシロ")
        .cloned()
        .unwrap()
        .into_priority_student(0.7);

    let sparkable = vec![mashiro.student().clone()];
    let priority = vec![mashiro];

    let gacha = GachaBuilder::new(79.0, 18.5, 2.5)
        .with_pool(pool)
        .with_priority(&priority)
        .finish()
        .unwrap();

    BannerBuilder::new("赤の季節、黒の制服")
        .with_gacha(&gacha)
        .with_name_translation(Language::English, "Red Season, Black Uniform")
        .with_sparkable_students(&sparkable)
        .finish()
        .unwrap()
}

fn create_2021_02_25_izuna_banner() -> Banner {
    let three_stars = "ヒナ, イオリ, ハルナ, イズミ, アル, スミレ, エイミ, カリン, ネル, マキ, ヒビキ, サヤ, シュン, シロコ, ホシノ, ヒフミ, ツルギ, マシロ, イズナ";
    let two_stars = "アカリ, ジュンコ, ムツキ, カヨコ, フウカ, ユウカ, アカネ, ハレ, ウタハ, チセ, ツバキ, セリカ, アヤネ, ハスミ, ハナエ, アイリ, シズコ";
    let one_stars =
        "チナツ, ハルカ, ジュリ, コタマ, アスナ, コトリ, フィーナ, スズミ, シミコ, セリナ, ヨシミ";

    let mut pool = Vec::new();
    pool.extend(get_students(three_stars));
    pool.extend(get_students(two_stars));
    pool.extend(get_students(one_stars));

    let shizuko = pool
        .iter()
        .find(|student| student.name == "シズコ")
        .cloned()
        .unwrap()
        .into_priority_student(3.0);

    let izuna = pool
        .iter()
        .find(|student| student.name == "イズナ")
        .cloned()
        .unwrap()
        .into_priority_student(0.7);

    let sparkable = vec![izuna.student().clone()];
    let priority = vec![izuna, shizuko];

    let gacha = GachaBuilder::new(ONE_STAR_RATE, TWO_STAR_RATE, THREE_STAR_RATE)
        .with_pool(pool)
        .with_priority(&priority)
        .finish()
        .unwrap();

    BannerBuilder::new("祭り囃子はしのぶれど")
        .with_gacha(&gacha)
        .with_name_translation(Language::English, "The Concealed Festival Band")
        .with_sparkable_students(&sparkable)
        .finish()
        .unwrap()
}

fn _create_2021_03_11_haruna_banner() -> Banner {
    let three_stars = "ヒナ, イオリ, ハルナ, イズミ, アル, スミレ, エイミ, カリン, ネル, マキ, ヒビキ, サヤ, シュン, シロコ, ホシノ, ヒフミ, ツルギ, マシロ, イズナ";
    let two_stars = "アカリ, ジュンコ, ムツキ, カヨコ, フウカ, ユウカ, アカネ, ハレ, ウタハ, チセ, ツバキ, セリカ, アヤネ, ハスミ, ハナエ, アイリ, シズコ";
    let one_stars =
        "チナツ, ハルカ, ジュリ, コタマ, アスナ, コトリ, フィーナ, スズミ, シミコ, セリナ, ヨシミ";

    let mut pool = Vec::new();
    pool.extend(get_students(three_stars));
    pool.extend(get_students(two_stars));
    pool.extend(get_students(one_stars));

    let haruna = pool
        .iter()
        .find(|student| student.name == "ハルナ")
        .cloned()
        .unwrap()
        .into_priority_student(0.7);

    let sparkable = vec![haruna.student().clone()];
    let priority = vec![haruna];

    let gacha = GachaBuilder::new(ONE_STAR_RATE, TWO_STAR_RATE, THREE_STAR_RATE)
        .with_pool(pool)
        .with_priority(&priority)
        .finish()
        .unwrap();

    BannerBuilder::new("その味は身命を賭してでも")
        .with_gacha(&gacha)
        .with_name_translation(
            Language::English,
            "Even if you risk your life for that taste",
        )
        .with_sparkable_students(&sparkable)
        .finish()
        .unwrap()
}

fn _create_2021_03_18_aru_banner() -> Banner {
    let three_stars = "ヒナ, イオリ, ハルナ, イズミ, アル, スミレ, エイミ, カリン, ネル, マキ, ヒビキ, サヤ, シュン, シロコ, ホシノ, ヒフミ, ツルギ, マシロ, イズナ";
    let two_stars = "アカリ, ジュンコ, ムツキ, カヨコ, フウカ, ユウカ, アカネ, ハレ, ウタハ, チセ, ツバキ, セリカ, アヤネ, ハスミ, ハナエ, アイリ, シズコ";
    let one_stars =
        "チナツ, ハルカ, ジュリ, コタマ, アスナ, コトリ, フィーナ, スズミ, シミコ, セリナ, ヨシミ";

    let mut pool = Vec::new();
    pool.extend(get_students(three_stars));
    pool.extend(get_students(two_stars));
    pool.extend(get_students(one_stars));

    let aru = pool
        .iter()
        .find(|student| student.name == "アル")
        .cloned()
        .unwrap()
        .into_priority_student(0.7);

    let sparkable = vec![aru.student().clone()];
    let priority = vec![aru];

    let gacha = GachaBuilder::new(ONE_STAR_RATE, TWO_STAR_RATE, THREE_STAR_RATE)
        .with_pool(pool)
        .with_priority(&priority)
        .finish()
        .unwrap();

    BannerBuilder::new("悪は一日にして成らず")
        .with_gacha(&gacha)
        .with_name_translation(Language::English, "Evil does not grow in a single Day")
        .with_sparkable_students(&sparkable)
        .finish()
        .unwrap()
}

fn get_students(student_list: &str) -> Vec<Student> {
    let names = student_list.split(", ");
    let mut students = match names.size_hint().1 {
        Some(size) => Vec::with_capacity(size),
        None => Vec::new(),
    };

    for name in names {
        let maybe_student = STUDENTS.iter().find(|student| student.name == name);

        match maybe_student {
            Some(student) => {
                students.push(student.clone());
            }
            None => error!("Could not find {} in students.json", name),
        };
    }

    students
}

fn get_rarity_colour(rarity: Rarity) -> Colour {
    match rarity {
        Rarity::One => Colour::from_rgb(227, 234, 240),
        Rarity::Two => Colour::from_rgb(255, 248, 124),
        Rarity::Three => Colour::from_rgb(253, 198, 229),
    }
}
