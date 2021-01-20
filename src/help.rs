use crate::config::{Configuration, CONFIG_FILENAME};
use crate::{Result, HOMEPAGE};
use crate::{NAME, VERSION};
use std::io::Write;

struct Command {
    pub name: String,
    pub tool: String,
    // TODO: description
}

pub fn print_help(configuration: &Configuration, out: &mut dyn Write) -> Result<()> {
    writeln!(out, "🔧 {} {} 🔧", NAME, VERSION)?;
    writeln!(out)?;
    writeln!(out, "🔧 A light-weight meta-tool to version and install tool dependencies for your software projects")?;
    writeln!(out)?;
    for configuration_file in &configuration.configuration_files {
        writeln!(out, "🔧 Loaded configuration from {}", configuration_file)?;
    }
    writeln!(out)?;
    writeln!(out, "Usage: tt [-v] <command> <args...>")?;
    writeln!(out, "  Run tool <command> with the provided arguments")?;
    writeln!(out)?;
    writeln!(out, "Flags:")?;
    writeln!(out, "  -v     Verbose debug output")?;
    writeln!(out)?;
    writeln!(out, "Usage: tt --download")?;
    writeln!(out, "  Download configured tools for later use")?;
    writeln!(out)?;
    writeln!(out, "Usage: tt --getBinaryPath <command>")?;
    writeln!(out, "  Writes the absolute path to the binary to stdout. This can be used for integration with other tooling.")?;
    writeln!(out)?;
    writeln!(out, "Usage: tt --getToolPath <tool>")?;
    writeln!(out, "  Writes the absolute path to the tool directory to stdout. This can be used for integration with other tooling.")?;
    writeln!(out)?;
    if configuration.configuration_files.is_empty() {
        writeln!(out, "No tool-tool file named {} found in current directory or ancestors, please create one and configure your tools.", CONFIG_FILENAME)?;
        writeln!(out, "Refer to {} for further information", HOMEPAGE)?;
    } else {
        print_commands(out, &configuration)?;
    }
    Ok(())
}

fn print_commands(out: &mut dyn Write, configuration: &Configuration) -> Result<()> {
    writeln!(out, "Available commands:")?;
    writeln!(out)?;
    let mut commands = vec![];
    let mut max_name = 7;
    let mut max_tool = 4;
    for tool in &configuration.tools {
        for name in tool.commands.keys() {
            max_name = max_name.max(name.len());
            max_tool = max_tool.max(tool.name.len());
            commands.push(Command {
                name: name.clone(),
                tool: format!("{} {}", tool.name, tool.version),
            })
        }
    }
    commands.sort_by_key(|command| command.name.clone());
    commands.insert(
        0,
        Command {
            name: "Command".to_string(),
            tool: "Tool".to_string(),
        },
    );
    for command in commands {
        writeln!(
            out,
            "   {:<max_name$}  {:<max_tool$}",
            command.name,
            command.tool,
            max_name = max_name,
            max_tool = max_tool
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ToolConfiguration;
    use std::io::Cursor;

    fn assert_help(configuration: &Configuration) {
        let mut buffer = Cursor::new(vec![]);
        print_help(configuration, &mut buffer).unwrap();
        let help_text = String::from_utf8(buffer.into_inner()).unwrap();
        let help_text = help_text.replace(&format!("🔧 {} {} 🔧", NAME, VERSION), "🔧 tool-tool $VER$ 🔧");
        insta::assert_yaml_snapshot!(help_text);
    }

    #[test]
    fn help_empty() {
        assert_help(&Configuration::default());
    }

    #[test]
    fn help_configured() {
        let mut configuration = Configuration::default();
        configuration
            .configuration_files
            .push("foo.bar.yaml".to_string());
        configuration.tools.push(ToolConfiguration {
            name: "foo".to_string(),
            version: "1.2.3".to_string(),
            commands: [("bar".to_string(), "bar".to_string())]
                .iter()
                .cloned()
                .collect(),
            ..ToolConfiguration::default()
        });
        configuration.tools.push(ToolConfiguration {
            name: "fizz".to_string(),
            version: "4.5.6".to_string(),
            commands: [
                ("buzz".to_string(), "buzz".to_string()),
                ("apply".to_string(), "apply".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            ..ToolConfiguration::default()
        });
        assert_help(&configuration);
    }
}
