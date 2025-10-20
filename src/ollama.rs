use std::env;
use std::sync::Arc;

use base64::Engine;

use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::chat::ChatMessage;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::images::Image;
use ollama_rs::Ollama;

use serenity::all::{Attachment, CreateThread, Message};
use serenity::prelude::Context;

use tokio::sync::Mutex;

use crate::HISTORY;

pub(crate) async fn thread_response(ctx: Context, msg: Message) {
    let typing = msg.channel_id.start_typing(&ctx.http);

    let history_lock = {
        let mut histories = HISTORY.lock().await;
        histories
            .entry(msg.channel_id.get())
            .or_insert_with(|| Arc::new(Mutex::new(Vec::new())))
            .clone()
    };

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

    let full_prompt = image_summary(msg.attachments, full_prompt).await;

    let res = {
        let mut history = history_lock.lock().await;
        ollama
            .send_chat_messages_with_history(
                &mut *history,
                ChatMessageRequest::new(model, vec![ChatMessage::user(full_prompt)]),
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
}

pub(crate) async fn message_response(ctx: Context, msg: Message) {
    let typing = msg.channel_id.start_typing(&ctx.http);

    let history_lock = {
        let mut histories = HISTORY.lock().await;
        histories
            .entry(msg.channel_id.get())
            .or_insert_with(|| Arc::new(Mutex::new(Vec::new())))
            .clone()
    };

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

    let full_prompt = image_summary(msg.attachments, full_prompt).await;

    let res = {
        let mut history = history_lock.lock().await;
        ollama
            .send_chat_messages_with_history(
                &mut *history,
                ChatMessageRequest::new(model, vec![ChatMessage::user(full_prompt)]),
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

async fn image_summary(attachments: Vec<Attachment>, mut prompt: String) -> String {
    let ollama = Ollama::new(
        env::var("OLLAMA_URL").expect("Expected a URL in the environment"),
        env::var("OLLAMA_PORT")
            .expect("Expected a port in the environment")
            .parse()
            .expect("Expected port to be an integer"),
    );

    let image_model = "image";
    let image_prompt = "Describe the following image in detail:";

    let mut num: u8 = 1;

    for attachment in attachments {
        if attachment
            .content_type
            .as_ref()
            .is_some_and(|ct| ct.contains("image"))
        {
            let attachment = attachment
                .download()
                .await
                .expect("Failed to download attachment");
            let attachment = base64::engine::general_purpose::STANDARD.encode(attachment);
            let summary = ollama
                .generate(
                    GenerationRequest::new(image_model.to_string(), image_prompt.to_string())
                        .add_image(Image::from_base64(attachment)),
                )
                .await;
            if let Ok(summary) = summary {
                prompt = format!(
                    "{} Image description #{}: {}",
                    prompt, num, summary.response
                );
                num += 1;
            }
        }
    }

    prompt
}
