use std::collections::HashMap;

use uritemplate::UriTemplate;

use crate::config::CommandNode;

use super::config;

pub struct CommandHandler {
    command_map: HashMap<String, CommandNode>,
    default_command: Option<CommandNode>,
}

impl CommandHandler {
    pub fn new(config: &config::Config) -> CommandHandler {
        let mut ret = Self {
            command_map: HashMap::new(),
            default_command: None,
        };

        for command in config.commands.iter() {
            for name in command.names.iter() {
                ret.command_map.insert(name.to_string(), command.to_owned());
                if command.is_default {
                    ret.default_command = Some(command.to_owned());
                }
            }
        }

        ret
    }

    fn handle_strict(&self, input: &str) -> Option<String> {
        let mut split = input.splitn(2, char::is_whitespace);
        let name = split.next().unwrap_or("");
        let arg = split.next().unwrap_or("").trim_start();

        let command = self.command_map.get(name)?;
        handle_command(command, arg)
    }

    pub fn handle(&self, input: &str) -> Option<String> {
        if let Some(ret) = self.handle_strict(input) {
            Some(ret)
        } else if let Some(command) = &self.default_command {
            handle_command(&command, input)
        } else {
            None
        }
    }
}

fn handle_command(command: &CommandNode, arg: &str) -> Option<String> {
    let (rule, captures) = command
        .rules
        .iter()
        .find_map(|rule| rule.pattern.captures(arg).map(|captures| (rule, captures)))?;

    let mut template = UriTemplate::new(rule.template.as_str());

    rule.pattern
        .capture_names()
        .enumerate()
        .for_each(|(i, name)| {
            if let Some(name) = name {
                if let Some(val) = captures.name(name) {
                    template.set(name, val.as_str());
                }
            } else if let Some(val) = captures.get(i) {
                template.set(format!("{}", i).as_str(), val.as_str());
            }
        });

    Some(template.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_commands() {
        let config = config::load_config().unwrap();
        let command_handler = CommandHandler::new(&config);

        for command in config.commands.iter() {
            for test in command.tests.iter() {
                let result = command_handler.handle(test.input.as_str());
                let expected = Some(test.expected_url.to_string());
                assert_eq!(result, expected, "input command is {:?}", test.input);
            }
        }
    }
}
