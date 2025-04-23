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
    s: mc::StateView,
    nodes: usize,
    expect: &[String],
    ordered: bool,
) -> Result<(), String> {
    for node in 0..nodes {
        let proc = node;
        let Some(got) = s.system().read_locals(node.to_string(), proc.to_string()) else {
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

pub fn check_someone_deliver(s: mc::StateView, nodes: usize) -> Option<usize> {
    for node in 0..nodes {
        let proc = node;
        if s.system()
            .read_locals(node.to_string(), proc.to_string())
            .map(|v| !v.is_empty())
            .unwrap_or(false)
        {
            return Some(node);
        }
    }
    None
}

////////////////////////////////////////////////////////////////////////////////

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

pub fn check_locals_cnt(s: mc::StateView, nodes: usize, max_locals: usize) -> Result<(), String> {
    for node in 0..nodes {
        let proc = node;
        let locals = s
            .system()
            .read_locals(node.to_string(), proc.to_string())
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
