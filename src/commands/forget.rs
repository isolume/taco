use std::collections::HashMap;
use ollama_rs::generation::chat::ChatMessage;
use serenity::all::{CommandOptionType, CreateCommandOption};
use serenity::builder::CreateCommand;
use serenity::model::application::{ResolvedOption, ResolvedValue};

pub fn run(options: &[ResolvedOption], mut history: HashMap<u64, Vec<ChatMessage>>) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::Channel(channel), ..
                }) = options.first()
    {
        history.remove(&channel.id.get());
        format!("Forgetting channel: {}", channel.id)
    } else {
        history.clear();
        "Forgetting everything".to_string()
    }
}

pub fn register() -> CreateCommand {
    
    CreateCommand::new("forget")
        .description("forget everything, or just a little bit")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Channel, "channel", "The channel to forget")
                .required(false)
        )
}