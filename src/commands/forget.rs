use ollama_rs::Ollama;
use serenity::all::{CommandOptionType, CreateCommandOption};
use serenity::builder::CreateCommand;
use serenity::model::application::{ResolvedOption, ResolvedValue};

pub fn run(options: &[ResolvedOption], mut ollama: Ollama) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::Channel(channel), .. 
                }) = options.first()
    {
        ollama.clear_messages_for_id(channel.id.get().to_string());
        format!("Forgetting channel: {}", channel.id)
    } else {
        ollama.clear_all_messages();
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