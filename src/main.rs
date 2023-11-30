use anyhow::anyhow;
use regex::Regex;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::{error, info};

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        // if contains "x.com" or "twitter.com" in the message, only say url with replaced domain to "fxtwitter.com"
        if msg.content.contains("https://x.com") || msg.content.contains("https://twitter.com") {
            let re = match Regex::new(r"(https://(x\.com|twitter\.com)[^\s]+)") {
                Ok(re) => re,
                Err(err) => {
                    error!("Error creating regex: {:?}", err);
                    return;
                }
            };
            let mut replaced_urls = Vec::new();
            for cap in re.captures_iter(&msg.content) {
                replaced_urls.push(
                    cap[0]
                        .replace("twitter.com", "fxtwitter.com")
                        .replace("x.com", "fxtwitter.com"),
                );
            }
            let urls_string = replaced_urls.join("\n");
            if let Err(e) = msg.channel_id.say(&ctx.http, urls_string).await {
                error!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}

// TODO: #1 unit tests
// #[cfg(test)]
// mod tests {
//     use super::*;

//     // test Bot::message
//     // mock say() and check if it is called with expected argument
//     #[test]
//     fn test_message() {
//         use mockall::predicate::*;
//         use serenity::http::Http;
//         use serenity::model::id::ChannelId;

//         let mut mock = MockHttp::new();
//         mock.expect_say()
//             .withf(|_, msg| msg == "https://fxtwitter.com/123")
//             .returning(|_, _| Ok(()));

//         let ctx = Context::new(mock, 0);
//         let msg = Message {
//             content: "https://twitter.com/123".to_string(),

//             ..Message::default()
//         };
//         let bot = Bot;
//         let _ = bot.message(ctx, msg);

//         // check if say() is called with expected argument
//         mock.checkpoint();

//         let mut mock = MockHttp::new();
//         mock.expect_say()
//             .withf(|_, msg| msg == "https://fxtwitter.com/123\nhttps://fxtwitter.com/456")
//             .returning(|_, _| Ok(()));

//         let ctx = Context::new(mock, 0);
//         let msg = Message {
//             content: "https://twitter.com/123 https://twitter.com/456".to_string(),

//             ..Message::default()
//         };
//     }
// }
