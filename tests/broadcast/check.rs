use std::collections::{HashMap, HashSet};

use super::causal::CausalChecker;

fn check_equals(left: &[String], right: &[String], ordered: bool) -> bool {
    if ordered {
        left == right
    } else {
        let mut left = left.to_vec();
        left.sort();
        let mut right = right.to_vec();
        right.sort();
        left == right
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn check_delivery_on_correct_nodes(
    s: mc::SystemHandle,
    nodes: usize,
    expect: &[String],
    ordered: bool,
) -> Result<(), String> {
    for node in 0..nodes {
        let Some(got) = s.read_locals(node.to_string(), "bcast") else {
            // node crashed
            continue;
        };
        if !check_equals(expect, &got, ordered) {
            return Err(format!(
                "Node №{}: expect={:?}, got={:?}",
                node, expect, got
            ));
        }
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////

pub fn check_someone_deliver(s: mc::SystemHandle, nodes: usize) -> Result<usize, String> {
    for node in 0..nodes {
        if s.read_locals(node.to_string(), "bcast")
            .map(|v| !v.is_empty())
            .unwrap_or(false)
        {
            return Ok(node);
        }
    }
    Err("no one deliver".into())
}

////////////////////////////////////////////////////////////////////////////////

#[allow(unused)]
pub fn check_depth(s: mc::StateView, max_depth: usize) -> Result<(), String> {
    if s.depth() <= max_depth {
        Ok(())
    } else {
        Err(format!(
            "too big depth: {}, max depth: {}",
            s.depth(),
            max_depth
        ))
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn check_locals_cnt(
    s: mc::SystemHandle,
    nodes: usize,
    max_locals: usize,
) -> Result<(), String> {
    for node in 0..nodes {
        let locals = s
            .read_locals(node.to_string(), "bcast")
            .map(|v| v.len())
            .unwrap_or(0);
        if locals > max_locals {
            return Err(format!(
                "too many locals: {}, but max is {}",
                locals, max_locals
            ));
        };
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////

pub fn check_casual_order(s: mc::SystemHandle, nodes: usize) -> Result<(), String> {
    let mut checker = CausalChecker::new(nodes);
    for log in s.log().iter() {
        match log {
            mc::LogEntry::ProcessSentLocalMessage(msg) => {
                let node: usize = msg.process.node.parse().unwrap();
                if msg.content != "connect" {
                    checker.deliver(node, &msg.content)?;
                }
            }
            mc::LogEntry::ProcessReceivedLocalMessage(msg) => {
                let node: usize = msg.process.node.parse().unwrap();
                if msg.content != "connect" {
                    checker.send(node, &msg.content);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////

pub fn check_uniform_agreement(s: mc::SystemHandle, nodes: usize) -> Result<(), String> {
    let mut messages = HashSet::new();
    for log in s.log().iter() {
        if let mc::LogEntry::ProcessSentLocalMessage(msg) = log {
            if msg.content != "connect" {
                messages.insert(msg.content.clone());
            }
        }
    }
    let messages = messages.into_iter().collect::<Vec<_>>();
    check_delivery_on_correct_nodes(s, nodes, &messages, false)
}

////////////////////////////////////////////////////////////////////////////////

pub fn check_validity(s: mc::SystemHandle) -> Result<(), String> {
    let mut messages: HashMap<String, HashSet<String>> = Default::default();
    for log in s.log().iter() {
        match log {
            mc::LogEntry::NodeCrashed(e) => {
                messages.remove(&e.node);
            }
            mc::LogEntry::ProcessReceivedLocalMessage(e) => {
                if e.content != "connect" {
                    let exists = !messages
                        .entry(e.process.node.clone())
                        .or_default()
                        .insert(e.content.clone());
                    assert!(!exists);
                }
            }
            mc::LogEntry::ProcessSentLocalMessage(e) => {
                messages
                    .entry(e.process.node.clone())
                    .or_default()
                    .remove(&e.content);
            }
            _ => {}
        }
    }
    for (node, e) in messages.into_iter() {
        if !e.is_empty() {
            return Err(format!(
                "Validity violation: node {node} not delivered registered messages: {e:?}"
            ));
        }
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////

pub fn check_validity_and_agreement(s: mc::SystemHandle, nodes: usize) -> Result<(), String> {
    check_uniform_agreement(s.clone(), nodes)?;
    check_validity(s.clone())
}
