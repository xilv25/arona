# Blue Archive Gacha Discord Bot

Internally, this bot is named Arona / アロナ, but I'm sure every bot that has anything to do with Blue Archive will be called Arona.


## Setup 
`./data/students.json` contains an Array with every student currently (2020-02-09) available in it. Modify as you will to suit your needs.

This Bot uses my [Gacha Sim library](https://github.com/paoda/bluearch-recruitment). You can see what code modifications are necessary
in order to create your own banner in `./src/recruitment.rs` (in the `create_banner()` function) in the README over there. 


Check out `.env.example` to see the one environment variable you need to set. This project uses [dotenv-rs](https://github.com/dotenv-rs/dotenv)
so a `.env` file or setting an actual environment variable will work. 

## Building
In order to build this project, you'll need a rust compiler.

Assuming you've installed using [rustup](https://rustup.rs/), the steps are as follows:
* `git clone https://github.com/paoda/arona`
* `cd arona`
* `cargo build --release` (and the executable can be found in `./target/release/arona`)

## Thank You
Thanks to https://rerollcdn.com which provides all the images for the discord bot. 

Check out **https://thearchive.gg** for information about Blue Archive!