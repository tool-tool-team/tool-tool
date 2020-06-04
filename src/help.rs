use crate::{DESCRIPTION, NAME, VERSION};
use crate::config::Configuration;

struct Command {
    pub name: String,
    pub tool: String,
    // TODO: description
}

pub fn print_help(configuration: &Configuration) {
    println!("🔧 {} {} 🔧", NAME, VERSION);
    println!();
    println!("{}", DESCRIPTION);
    println!();
    println!("Available commands:");
    println!();
    let mut commands = vec![];
    let mut max_name = 7;
    let mut max_tool = 4;
    for tool in &configuration.tools {
        for (name, _) in &tool.commands {
            max_name = max_name.max(name.len());
            max_tool = max_tool.max(tool.name.len());
            commands.push(Command {
                name: name.clone(),
                tool: format!("{} {}", tool.name, tool.version),
            })
        }
    }
    commands.sort_by_key(|command| command.name.clone());
    commands.insert(0, Command {
        name: "Command".to_string(),
        tool: "Tool".to_string(),
    });
    for command in commands {
        println!("   {:<max_name$}  {:<max_tool$}", command.name, command.tool, max_name= max_name, max_tool=max_tool);
    }
}
