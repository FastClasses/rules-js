use petgraph::algo::kosaraju_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

use crate::lockfile::{sanitize_target_name, PackageInfo};

#[derive(Debug)]
pub struct BrokenEdge {
    pub from: String,
    pub to: String,
}

pub struct DepGraph {
    graph: DiGraph<String, ()>,
    node_map: HashMap<String, NodeIndex>,
    deps: HashMap<String, Vec<String>>,
}

impl DepGraph {
    pub fn build(packages: &[PackageInfo]) -> Self {
        let mut graph: DiGraph<String, ()> = DiGraph::new();
        let mut node_map: HashMap<String, NodeIndex> = HashMap::new();
        let mut deps: HashMap<String, Vec<String>> = HashMap::new();

        for pkg in packages {
            let idx = graph.add_node(pkg.target_name.clone());
            node_map.insert(pkg.target_name.clone(), idx);
        }

        for pkg in packages {
            let mut pkg_deps = Vec::new();

            let all_deps = pkg
                .dependencies
                .iter()
                .chain(pkg.optional_dependencies.iter());

            for (dep_name, dep_ver) in all_deps {
                let dep_target = sanitize_target_name(&format!("/{}@{}", dep_name, dep_ver));
                let dep_str = format!(":{}", dep_target);

                if !pkg_deps.contains(&dep_str) {
                    pkg_deps.push(dep_str);
                }

                if let (Some(&src_idx), Some(&tgt_idx)) =
                    (node_map.get(&pkg.target_name), node_map.get(&dep_target))
                {
                    graph.add_edge(src_idx, tgt_idx, ());
                }
            }

            deps.insert(pkg.target_name.clone(), pkg_deps);
        }

        DepGraph {
            graph,
            node_map,
            deps,
        }
    }

    pub fn detect_and_break_cycles(&mut self) -> Vec<BrokenEdge> {
        let mut broken = Vec::new();

        let sccs = kosaraju_scc(&self.graph);
        for component in &sccs {
            if component.len() < 2 && !self.graph.contains_edge(component[0], component[0]) {
                continue;
            }
            let mut names: Vec<(NodeIndex, String)> = component
                .iter()
                .map(|&idx| (idx, self.graph[idx].clone()))
                .collect();
            names.sort_by(|a, b| a.1.cmp(&b.1));

            if let Some((first_idx, first_name)) = names.first() {
                let component_set: std::collections::HashSet<NodeIndex> =
                    component.iter().copied().collect();

                let neighbors: Vec<NodeIndex> = self
                    .graph
                    .neighbors(*first_idx)
                    .filter(|n| component_set.contains(n))
                    .collect();

                for neighbor_idx in neighbors {
                    let neighbor_name = self.graph[neighbor_idx].clone();

                    if let Some(dep_list) = self.deps.get_mut(first_name) {
                        let dep_str = format!(":{}", neighbor_name);
                        dep_list.retain(|d| d != &dep_str);
                    }

                    if let Some(edge) = self.graph.find_edge(*first_idx, neighbor_idx) {
                        self.graph.remove_edge(edge);
                    }

                    broken.push(BrokenEdge {
                        from: first_name.clone(),
                        to: neighbor_name,
                    });
                }
            }
        }

        broken
    }

    pub fn get_deps(&self, target_name: &str) -> Vec<String> {
        self.deps.get(target_name).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lockfile::PackageInfo;

    fn make_pkg(name: &str, version: &str, deps: Vec<(&str, &str)>) -> PackageInfo {
        PackageInfo {
            name: name.to_string(),
            version: version.to_string(),
            target_name: sanitize_target_name(&format!("/{}@{}", name, version)),
            tarball_url: None,
            integrity: None,
            dependencies: deps
                .into_iter()
                .map(|(n, v)| (n.to_string(), v.to_string()))
                .collect(),
            optional_dependencies: vec![],
        }
    }

    #[test]
    fn test_no_cycle() {
        let packages = vec![
            make_pkg("a", "1.0", vec![("b", "1.0")]),
            make_pkg("b", "1.0", vec![("c", "1.0")]),
            make_pkg("c", "1.0", vec![]),
        ];

        let mut graph = DepGraph::build(&packages);
        let broken = graph.detect_and_break_cycles();
        assert!(broken.is_empty(), "No cycles should be detected");

        let a_deps = graph.get_deps("a_1.0");
        assert!(a_deps.contains(&":b_1.0".to_string()));
    }

    #[test]
    fn test_simple_cycle() {
        let packages = vec![
            make_pkg("a", "1.0", vec![("b", "1.0")]),
            make_pkg("b", "1.0", vec![("a", "1.0")]),
        ];

        let mut graph = DepGraph::build(&packages);
        let broken = graph.detect_and_break_cycles();

        assert_eq!(broken.len(), 1, "One edge should be broken");
        assert_eq!(broken[0].from, "a_1.0");
        assert_eq!(broken[0].to, "b_1.0");

        let b_deps = graph.get_deps("b_1.0");
        assert!(b_deps.contains(&":a_1.0".to_string()));

        let a_deps = graph.get_deps("a_1.0");
        assert!(!a_deps.contains(&":b_1.0".to_string()));
    }

    #[test]
    fn test_three_node_cycle() {
        let packages = vec![
            make_pkg("a", "1.0", vec![("b", "1.0")]),
            make_pkg("b", "1.0", vec![("c", "1.0")]),
            make_pkg("c", "1.0", vec![("a", "1.0")]),
        ];

        let mut graph = DepGraph::build(&packages);
        let broken = graph.detect_and_break_cycles();

        assert!(!broken.is_empty(), "Cycle should be detected and broken");
        assert_eq!(broken[0].from, "a_1.0");
        assert_eq!(broken[0].to, "b_1.0");
    }
}
