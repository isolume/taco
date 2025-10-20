use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use ollama_rs::generation::chat::ChatMessage;

use serenity::all::{
    ActivityData, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId,
    Interaction, Ready,
};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use lazy_static::lazy_static;

mod commands;
mod ollama;

use crate::ollama::{message_response, thread_response};

struct Handler;

lazy_static! {
    static ref HISTORY: Mutex<HashMap<u64, Arc<Mutex<Vec<ChatMessage>>>>> =
        Mutex::new(HashMap::new());
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let channel = if msg.guild_id.is_some() {
            Some(
                msg.channel_id
                    .to_channel(&ctx.http)
                    .await
                    .expect("Channel id could not be converted to Channel")
                    .guild()
                    .expect("Channel could not be converted to guild channel"),
            )
        } else {
            None
        };

        if msg.channel_id.get() == 1270867600309489756 {
            thread_response(ctx, msg).await;
        } else if (channel.as_ref().is_some_and(|ch| {
            ch.parent_id
                .expect("Channel parent ID could not be found")
                .get()
                == 1270867600309489756
        })) || msg.guild_id.is_none() && msg.author.id == 1041839238733373450
            || msg.content.contains("<@1081466107153633371>")
            || msg
                .referenced_message
                .as_ref()
                .is_some_and(|m| m.author.id.get() == 1081466107153633371)
        {
            message_response(ctx, msg).await;
        }
    }

    async fn ready(&self, ctx: Context, data: Ready) {
        println!("{} is connected!", data.user.name);

        ctx.set_activity(Some(ActivityData::custom("eating waffles")));

        let guild_id = GuildId::new(1066751637898662039);

        let commands = guild_id
            .set_commands(&ctx.http, vec![commands::forget::register()])
            .await;

        println!(
            "The following commands have been registered for guild {}: {:#?}",
            guild_id, commands
        )
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let content = match command.data.name.as_str() {
                "forget" => Some(commands::forget::run(&command.data.options()).await),
                _ => Some("error, command not implemented".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {}", why)
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}")
    }
}
