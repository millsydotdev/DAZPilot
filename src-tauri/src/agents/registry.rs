use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;

use super::{AgentRequest, AgentResponse};

pub type AgentHandler = fn(AgentRequest) -> AgentResponse;

pub struct AgentNode {
    pub agent_type: String,
    pub description: String,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub capabilities: Vec<String>,
    pub handler: AgentHandler,
}

pub struct AgentRegistry {
    nodes: HashMap<String, AgentNode>,
    roots: Vec<String>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            roots: Vec::new(),
        }
    }

    pub fn register(&mut self, node: AgentNode) -> Result<(), String> {
        let agent_type = node.agent_type.clone();
        if self.nodes.contains_key(&agent_type) {
            return Err(format!("Agent '{}' is already registered", agent_type));
        }
        if let Some(ref parent) = node.parent {
            if !self.nodes.contains_key(parent) {
                return Err(format!(
                    "Parent agent '{}' not found for '{}'",
                    parent, agent_type
                ));
            }
            self.nodes
                .get_mut(parent)
                .unwrap()
                .children
                .push(agent_type.clone());
        } else {
            self.roots.push(agent_type.clone());
        }
        self.nodes.insert(agent_type, node);
        Ok(())
    }

    pub fn get(&self, agent_type: &str) -> Option<&AgentNode> {
        self.nodes.get(agent_type)
    }

    pub fn get_mut(&mut self, agent_type: &str) -> Option<&mut AgentNode> {
        self.nodes.get_mut(agent_type)
    }

    pub fn unregister(&mut self, agent_type: &str) -> Result<AgentNode, String> {
        let node = self
            .nodes
            .remove(agent_type)
            .ok_or_else(|| format!("Agent '{}' not found", agent_type))?;
        if let Some(ref parent) = node.parent {
            if let Some(parent_node) = self.nodes.get_mut(parent) {
                parent_node.children.retain(|c| c != agent_type);
            }
        } else {
            self.roots.retain(|r| r != agent_type);
        }
        for child in &node.children {
            if let Some(child_node) = self.nodes.get_mut(child) {
                child_node.parent = node.parent.clone();
            }
        }
        Ok(node)
    }

    pub fn get_children(&self, agent_type: &str) -> Vec<&AgentNode> {
        self.nodes
            .get(agent_type)
            .map(|parent| {
                parent
                    .children
                    .iter()
                    .filter_map(|child_type| self.nodes.get(child_type))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_parent(&self, agent_type: &str) -> Option<&AgentNode> {
        self.nodes
            .get(agent_type)
            .and_then(|node| node.parent.as_ref())
            .and_then(|parent_type| self.nodes.get(parent_type))
    }

    pub fn get_descendants(&self, agent_type: &str) -> Vec<&AgentNode> {
        let mut result = Vec::new();
        if let Some(node) = self.nodes.get(agent_type) {
            for child_type in &node.children {
                if let Some(child) = self.nodes.get(child_type) {
                    result.push(child);
                    result.extend(self.get_descendants(child_type));
                }
            }
        }
        result
    }

    pub fn get_ancestors(&self, agent_type: &str) -> Vec<&AgentNode> {
        let mut result = Vec::new();
        let mut current = self.nodes.get(agent_type);
        while let Some(node) = current {
            if let Some(ref parent_type) = node.parent {
                if let Some(parent) = self.nodes.get(parent_type) {
                    result.push(parent);
                    current = Some(parent);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        result
    }

    pub fn get_roots(&self) -> &[String] {
        &self.roots
    }

    pub fn contains(&self, agent_type: &str) -> bool {
        self.nodes.contains_key(agent_type)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn list_agents(&self) -> Vec<AgentInfo> {
        let mut agents: Vec<AgentInfo> = self.nodes.values().map(|n| n.into()).collect();
        agents.sort_by(|a, b| a.agent_type.cmp(&b.agent_type));
        agents
    }

    pub fn find_by_capability(&self, capability: &str) -> Vec<&AgentNode> {
        let lower = capability.to_lowercase();
        self.nodes
            .values()
            .filter(|node| {
                node.capabilities.iter().any(|c| c.to_lowercase() == lower)
                    || node.agent_type.to_lowercase().contains(&lower)
            })
            .collect()
    }

    pub fn find_by_input(&self, input: &str) -> Vec<(&AgentNode, usize)> {
        let mut matches: Vec<(&AgentNode, usize)> = self
            .nodes
            .values()
            .filter_map(|node| {
                let count = node
                    .capabilities
                    .iter()
                    .filter(|c| input_matches_capability(input, c))
                    .count();
                if count > 0 {
                    Some((node, count))
                } else {
                    None
                }
            })
            .collect();
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches
    }

    pub fn print_tree(&self) -> String {
        let mut output = String::new();
        for root in &self.roots {
            self.print_node(root, 0, &mut output);
        }
        output
    }

    fn print_node(&self, agent_type: &str, depth: usize, output: &mut String) {
        if let Some(node) = self.nodes.get(agent_type) {
            let indent = "  ".repeat(depth);
            let kids = if node.children.is_empty() {
                String::new()
            } else {
                format!(" [{} children]", node.children.len())
            };
            output.push_str(&format!(
                "{}- {}: {}{}\n",
                indent, node.agent_type, node.description, kids
            ));
            for child in &node.children {
                self.print_node(child, depth + 1, output);
            }
        }
    }
}

pub fn input_matches_capability(input: &str, capability: &str) -> bool {
    let input = input.to_lowercase();
    let capability = capability.to_lowercase();

    if capability.split_whitespace().count() > 1 {
        return input.contains(&capability);
    }

    input
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|word| word == capability)
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentInfo {
    pub agent_type: String,
    pub description: String,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub capabilities: Vec<String>,
}

impl From<&AgentNode> for AgentInfo {
    fn from(node: &AgentNode) -> Self {
        AgentInfo {
            agent_type: node.agent_type.clone(),
            description: node.description.clone(),
            parent: node.parent.clone(),
            children: node.children.clone(),
            capabilities: node.capabilities.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentTreeNode {
    pub agent_type: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub children: Vec<AgentTreeNode>,
}

impl AgentRegistry {
    pub fn get_agent_tree(&self) -> Vec<AgentTreeNode> {
        self.roots
            .iter()
            .filter_map(|root| self.build_tree_node(root))
            .collect()
    }

    fn build_tree_node(&self, agent_type: &str) -> Option<AgentTreeNode> {
        let node = self.nodes.get(agent_type)?;
        let children: Vec<AgentTreeNode> = node
            .children
            .iter()
            .filter_map(|child| self.build_tree_node(child))
            .collect();
        Some(AgentTreeNode {
            agent_type: node.agent_type.clone(),
            description: node.description.clone(),
            capabilities: node.capabilities.clone(),
            children,
        })
    }
}

impl From<&AgentNode> for AgentTreeNode {
    fn from(node: &AgentNode) -> Self {
        AgentTreeNode {
            agent_type: node.agent_type.clone(),
            description: node.description.clone(),
            capabilities: node.capabilities.clone(),
            children: Vec::new(),
        }
    }
}

static AGENT_REGISTRY: Mutex<Option<AgentRegistry>> = Mutex::new(None);

pub fn init_registry() {
    let mut guard = AGENT_REGISTRY.lock().unwrap();
    if guard.is_none() {
        *guard = Some(AgentRegistry::new());
    }
}

pub fn global_registry() -> std::sync::MutexGuard<'static, Option<AgentRegistry>> {
    AGENT_REGISTRY.lock().unwrap()
}

pub fn with_registry<F, R>(f: F) -> R
where
    F: FnOnce(&AgentRegistry) -> R,
{
    let guard = AGENT_REGISTRY.lock().unwrap();
    let registry = guard.as_ref().expect("AgentRegistry not initialized");
    f(registry)
}

pub fn with_registry_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut AgentRegistry) -> R,
{
    let mut guard = AGENT_REGISTRY.lock().unwrap();
    let registry = guard.as_mut().expect("AgentRegistry not initialized");
    f(registry)
}
