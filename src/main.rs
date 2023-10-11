use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        prelude::{Activity, ChannelType},
    },
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};
use std::env;

#[inline]
fn get_intents() -> GatewayIntents {
    let mut intents = GatewayIntents::empty();
    intents.insert(GatewayIntents::GUILDS);
    intents.insert(GatewayIntents::GUILD_MESSAGES);
    intents
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!("{} is connected!", ready.user.name);
        ctx.set_activity(Activity::playing("Auto announce")).await;
    }

    async fn message(&self, ctx: Context, message: Message) {
        log::trace!("Received message");
        if message.is_private() || message.author.bot {
            return;
        }

        let channel = message.channel(&ctx).await.unwrap().guild().unwrap();
        if channel.kind != ChannelType::News {
            return;
        }

        if channel
            .topic
            .unwrap_or("".into())
            .contains("DisableAnnounce")
        {
            log::debug!("Announce disabled channel");
            return;
        }

        log::debug!("Announce message. trying to publish");

        let posted = message.crosspost(&ctx.http).await;

        if let Err(e) = posted {
            if e.to_string().contains("Permissions") || e.to_string().contains("Access") {
                log::debug!("Announce failed due to permissions");
                return;
            }
            log::error!("Failed to publish message: {}", e);
            return;
        }

        log::debug!("Announce success");

        return;
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::builder()
        .filter_module("announcer", {
            if cfg!(debug_assertions) {
                log::LevelFilter::Trace
            } else {
                log::LevelFilter::Info
            }
        })
        .init();

    log::info!("Starting...");

    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment");

    let mut client = Client::builder(token, get_intents())
        .event_handler(Handler)
        .await
        .unwrap();

    client.start_autosharded().await.unwrap();
}
