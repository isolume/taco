use std::env;

use ollama_rs::generation::chat::ChatMessage;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::Ollama;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::all::{ActivityData, CreateInteractionResponse, CreateInteractionResponseMessage, CreateThread, GuildId, Interaction, Ready};

use lazy_static::lazy_static;
use ollama_rs::generation::completion::request::GenerationRequest;

mod commands;

struct Handler;

lazy_static! {
    static ref OLLAMA: Mutex<Ollama> = Mutex::new(Ollama::new_with_history(
        env::var("OLLAMA_URL").expect("Expected a URL in the environment"),
        env::var("OLLAMA_PORT").expect("Expected a port in the environment").parse().expect("Expected port to be an integer"),
        1000
    ));
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id.get() == 1081466107153633371 {
            return;
        }
        
        let mut ollama = OLLAMA.lock().await;
        let summary_model = "llama3.1:latest".to_string();
        let model = "taco".to_string();
        let prompt = msg.content;
        let full_prompt = if let Some(referenced_message) = msg.referenced_message {
            format!("SYSTEM: Referenced message: {}. Referenced author's ID: <@{}>. User's ID: <@{}>. User's message:> {}",referenced_message.content, referenced_message.author.id.get(), msg.author.id.get(), prompt)
        } else {
            format!("SYSTEM: User's ID: <@{}>. User's message:> {}", msg.author.id.get(), prompt)
        };

        let channel = msg
            .channel_id
            .to_channel(&ctx.http)
            .await
            .expect("Channel id could not be converted to Channel")
            .guild()
            .expect("Channel could not be converted to guild channel");

        if msg.channel_id.get() == 1270867600309489756 {
            let typing = msg.channel_id.start_typing(&ctx.http);
            let history_id = msg.id.get().to_string();

            let res = ollama.send_chat_messages_with_history(
                ChatMessageRequest::new(
                    model.clone(),
                    vec![ChatMessage::user(full_prompt)],
                ),
                history_id
            ).await;
            
            let summary_prompt = format!("SYSTEM: Summarize the following text in 12 words or less. Summarize the text as is, and do not ask questions back.\
             Assume that you are not talking to a user with your response, but instead writing the summary for the header of a news publication. Also, do not surround the text with quotes.\
             Here is the text: {}", prompt);
            
            let summary = ollama.generate(GenerationRequest::new(summary_model, summary_prompt)).await;

            if let (Ok(res), Ok(summary)) = (res, summary) {
                let res = res.message.expect("Message could not be found!");
                let summary = sub_strings(summary.response.as_str(), 100).first().expect("Could not find first 100 chars").to_string();
                let chat = msg.channel_id.create_thread_from_message(
                    &ctx.http, msg.id.get(),
                    CreateThread::new(summary)
                ).await.expect("Cannot create thread");
                
                for chunk in sub_strings(&res.content, 2000) {
                    if let Err(why) = chat.say(&ctx.http, chunk).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
            typing.stop();
        }
        else if channel.parent_id.expect("Channel parent id could not be found").get() == 1270867600309489756 || msg.guild_id.is_none() {
            let typing = msg.channel_id.start_typing(&ctx.http);
            let history_id = msg.channel_id.get().to_string();

            let res = ollama.send_chat_messages_with_history(
                ChatMessageRequest::new(
                    model,
                    vec![ChatMessage::user(full_prompt)],
                ),
                history_id
            ).await;

            if let Ok(res) = res {
                let res = res.message.expect("Message could not be found!");
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
        
        let guild_id = GuildId::new(
            1066751637898662039
        );
        
        let commands = guild_id
            .set_commands(&ctx.http, vec![
                commands::ping::register(),
            ]).await;
        
        println!("The following commands have been registered for guild {}: {:#?}", guild_id, commands)
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                _ => Some("Error, command not implemented!".to_string())
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
    
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Error creating client");
    
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