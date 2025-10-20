use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::chat::ChatMessage;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;

use serenity::all::{
    ActivityData, CreateInteractionResponse, CreateInteractionResponseMessage, CreateThread,
    GuildId, Interaction, Ready,
};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use lazy_static::lazy_static;

mod commands;

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
        let history_lock = {
            let mut histories = HISTORY.lock().await;
            histories
                .entry(msg.channel_id.get())
                .or_insert_with(|| Arc::new(Mutex::new(Vec::new())))
                .clone()
        };
        if msg.channel_id.get() == 1270867600309489756 {
            let typing = msg.channel_id.start_typing(&ctx.http);

            let mut ollama = Ollama::new(
                env::var("OLLAMA_URL").expect("Expected a URL in the environment"),
                env::var("OLLAMA_PORT")
                    .expect("Expected a port in the environment")
                    .parse()
                    .expect("Expected port to be an integer"),
            );
            let summary_model = "summary".to_string();
            let model = "taco".to_string();
            let prompt = msg.content;
            let referenced_message = msg.referenced_message;
            let full_prompt = if let Some(referenced_message) = referenced_message {
                format!("SYSTEM: Referenced message: {}. Referenced author's ID: <@{}>. User's ID: <@{}>. User's message:> {}", referenced_message.content, referenced_message.author.id.get(), msg.author.id.get(), prompt)
            } else {
                format!(
                    "SYSTEM: User's ID: <@{}>. User's message:> {}",
                    msg.author.id.get(),
                    prompt
                )
            };

            let res = {
                let mut history = history_lock.lock().await;
                ollama
                    .send_chat_messages_with_history(
                        &mut *history,
                        ChatMessageRequest::new(
                            model,
                            vec![ChatMessage::user(full_prompt)],
                        ),
                    )
                    .await
            };

            let summary = ollama
                .generate(GenerationRequest::new(summary_model, prompt))
                .await;

            if let (Ok(res), Ok(summary)) = (res, summary) {
                let res = res.message;
                let summary = sub_strings(summary.response.as_str(), 100)
                    .first()
                    .expect("Could not find first 100 chars")
                    .to_string();
                let chat = msg
                    .channel_id
                    .create_thread_from_message(&ctx.http, msg.id.get(), CreateThread::new(summary))
                    .await
                    .expect("Cannot create thread");

                for chunk in sub_strings(&res.content, 2000) {
                    if let Err(why) = chat.say(&ctx.http, chunk).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
            typing.stop();
        } else if (channel.as_ref().is_some_and(|ch| {
            ch.parent_id
                .expect("Channel parent ID could not be found")
                .get()
                == 1270867600309489756
        })) || msg.guild_id.is_none() && msg.author.id == 1041839238733373450 || msg.content.contains("<@1081466107153633371>") || msg.referenced_message.as_ref().is_some_and(|m| m.author.id.get() == 1081466107153633371)
        {
            let typing = msg.channel_id.start_typing(&ctx.http);

            let mut ollama = Ollama::new(
                env::var("OLLAMA_URL").expect("Expected a URL in the environment"),
                env::var("OLLAMA_PORT")
                    .expect("Expected a port in the environment")
                    .parse()
                    .expect("Expected port to be an integer"),
            );
            let model = "taco".to_string();
            let prompt = msg.content;
            let referenced_message = msg.referenced_message;
            let full_prompt = if let Some(referenced_message) = referenced_message {
                format!("SYSTEM: Referenced message: {}. Referenced author's ID: <@{}>. User's ID: <@{}>. User's message:> {}", referenced_message.content, referenced_message.author.id.get(), msg.author.id.get(), prompt)
            } else {
                format!(
                    "SYSTEM: User's ID: <@{}>. User's message:> {}",
                    msg.author.id.get(),
                    prompt
                )
            };

            let res = {
                let mut history = history_lock.lock().await;
                ollama
                    .send_chat_messages_with_history(
                        &mut *history,
                        ChatMessageRequest::new(
                            model,
                            vec![ChatMessage::user(full_prompt)],
                        ),
                    )
                    .await
            };

            if let Ok(res) = res {
                let res = res.message;
                for chunk in sub_strings(&res.content, 2000) {
                    if let Err(why) = msg.channel_id.say(&ctx.http, chunk).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
            typing.stop();
        }
    }

    async fn ready(&self, ctx: Context, data: Ready) {
        println!("{} is connected!", data.user.name);

        ctx.set_activity(Some(ActivityData::custom("Eating waffles")));

        let guild_id = GuildId::new(1066751637898662039);

        let commands = guild_id
            .set_commands(
                &ctx.http,
                vec![commands::forget::register()],
            )
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
                _ => Some("Error, command not implemented!".to_string()),
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

fn sub_strings(string: &str, sub_len: usize) -> Vec<&str> {
    let mut subs = Vec::with_capacity(string.len() / sub_len);
    let mut iter = string.chars();
    let mut pos = 0;

    while pos < string.len() {
        let mut len = 0;
        for ch in iter.by_ref().take(sub_len) {
            len += ch.len_utf8();
        }
        subs.push(&string[pos..pos + len]);
        pos += len;
    }
    subs
}
