use std::convert::TryFrom;

use kdl::{KdlNode, KdlValue};
use regex::Regex;

pub const CONFIG_STR: &str = include_str!("./../../config.kdl");

#[derive(Debug, Clone)]
pub struct RuleNode {
    pub pattern: Regex,
    pub template: String,
}

impl TryFrom<KdlNode> for RuleNode {
    type Error = String;

    fn try_from(node: KdlNode) -> Result<Self, Self::Error> {
        match node.name.as_str() {
            "noargs" => {
                if let [KdlValue::String(url)] = &node.values[..] {
                    Ok(RuleNode {
                        pattern: Regex::new("^$").unwrap(),
                        template: url.clone(),
                    })
                } else {
                    Err("noargs node must have one string value".to_string())
                }
            }
            "freeform" => {
                if let [KdlValue::String(template)] = &node.values[..] {
                    Ok(RuleNode {
                        pattern: Regex::new("^.*$").unwrap(),
                        template: template.clone(),
                    })
                } else {
                    Err("freeform node must have one string value".to_string())
                }
            }
            "regex" => {
                if let [KdlValue::String(template), KdlValue::String(pattern)] = &node.values[..] {
                    Ok(RuleNode {
                        pattern: Regex::new(&pattern).map_err(|err| err.to_string())?,
                        template: template.clone(),
                    })
                } else {
                    Err("regex node must have two string values".to_string())
                }
            }
            _ => {
                Err("expected noargs, freeform, or regex node but found something else".to_string())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RulesNode(Vec<RuleNode>);

impl TryFrom<KdlNode> for RulesNode {
    type Error = String;

    fn try_from(node: KdlNode) -> Result<Self, Self::Error> {
        if node.name != "rules" {
            return Err("expected 'rules' node but found something else".to_string());
        }

        let tests = node
            .children
            .into_iter()
            .map(RuleNode::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(tests))
    }
}

#[derive(Debug, Clone)]
pub struct TestNode {
    pub input: String,
    pub expected_url: String,
}

impl TryFrom<KdlNode> for TestNode {
    type Error = String;

    fn try_from(node: KdlNode) -> Result<Self, Self::Error> {
        if node.name != "accept" {
            return Err("expected 'accept' node but found something else".to_string());
        }

        if let [KdlValue::String(input), KdlValue::String(expected_url)] = &node.values[..] {
            Ok(Self {
                input: input.clone(),
                expected_url: expected_url.clone(),
            })
        } else {
            Err("expected accept node to have two string values".to_string())
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestsNode(Vec<TestNode>);

impl TryFrom<KdlNode> for TestsNode {
    type Error = String;

    fn try_from(node: KdlNode) -> Result<Self, Self::Error> {
        if node.name != "tests" {
            return Err("expected 'tests' node but found something else".to_string());
        }

        let tests = node
            .children
            .into_iter()
            .map(TestNode::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(tests))
    }
}

#[derive(Debug, Clone)]
pub struct CommandNode {
    pub names: Vec<String>,
    pub rules: Vec<RuleNode>,
    pub tests: Vec<TestNode>,
    pub is_default: bool,
}

impl TryFrom<KdlNode> for CommandNode {
    type Error = String;

    fn try_from(node: KdlNode) -> Result<Self, Self::Error> {
        if node.name != "command" {
            return Err("expected 'command' node but found something else".to_string());
        }

        let names = node
            .values
            .into_iter()
            .map(String::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())?;

        if names.is_empty() {
            return Err("command node must have at least one name".to_string());
        }

        let is_default = node
            .properties
            .get("default")
            .map(bool::try_from)
            .unwrap_or(Ok(false))
            .map_err(|err| err.to_string())?;

        let mut rules = Vec::new();
        let mut tests = Vec::new();

        for child_node in &node.children {
            match child_node.name.as_str() {
                "rules" => rules.append(&mut RulesNode::try_from(child_node.clone())?.0),
                "tests" => tests.append(&mut TestsNode::try_from(child_node.clone())?.0),
                _ => {
                    return Err(
                        "expected rules or tests nodes but found something else".to_string()
                    );
                }
            }
        }

        Ok(Self {
            names,
            rules,
            tests,
            is_default,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub commands: Vec<CommandNode>,
}

pub fn load_config() -> Result<Config, String> {
    Config::try_from(CONFIG_STR)
}

impl TryFrom<&str> for Config {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let commands = kdl::parse_document(value)
            .map_err(|err| err.to_string())?
            .into_iter()
            .map(CommandNode::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let num_default = commands.iter().filter(|c| c.is_default).count();
        if num_default > 1 {
            return Err("more than one command is marked as default".to_string());
        }

        // TODO check for duplicate names

        Ok(Self { commands })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_is_valid() {
        load_config().unwrap();
    }

    // TODO test various config scenarios
}
