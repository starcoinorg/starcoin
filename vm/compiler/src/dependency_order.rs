// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, ensure, Result};
use move_binary_format::{access::ModuleAccess, file_format::CompiledModule};
use petgraph::graphmap::DiGraphMap;
use std::collections::BTreeMap;

/// Given an array of compiled modules, sort the modules by their dependency orders.
pub fn sort_by_dependency_order<'a>(
    modules: impl IntoIterator<Item = &'a CompiledModule>,
) -> Result<Vec<CompiledModule>> {
    let graph = DependencyGraph::new(modules)?;
    let result = graph.compute_topological_order()?.cloned().collect();
    Ok(result)
}

/// Directed graph capturing dependencies between modules
struct DependencyGraph<'a> {
    /// Set of modules guaranteed to be closed under dependencies
    modules: Vec<&'a CompiledModule>,
    graph: DiGraphMap<ModuleIndex, ()>,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
struct ModuleIndex(usize);

impl<'a> DependencyGraph<'a> {
    /// Construct a dependency graph from a set of `modules`.
    /// Panics if `modules` contains duplicates or is not closed under the depedency relation
    pub fn new(module_iter: impl IntoIterator<Item = &'a CompiledModule>) -> Result<Self> {
        let mut modules = vec![];
        let mut reverse_modules = BTreeMap::new();
        for (i, m) in module_iter.into_iter().enumerate() {
            modules.push(m);
            ensure!(
                reverse_modules
                    .insert(m.self_id(), ModuleIndex(i))
                    .is_none(),
                "Duplicate module found, id: {}",
                m.self_id()
            );
        }

        let mut graph = DiGraphMap::new();
        for module in &modules {
            let module_idx: ModuleIndex = *reverse_modules.get(&module.self_id()).unwrap();
            graph.add_node(module_idx);
            let mut deps = module.immediate_dependencies();
            if !deps.is_empty() {
                // make the function stable.
                deps.sort();
                for dep in deps {
                    if let Some(dep_idx) = reverse_modules.get(&dep) {
                        graph.add_edge(*dep_idx, module_idx, ());
                    }
                }
            }
        }
        Ok(DependencyGraph { modules, graph })
    }

    /// Return an iterator over the modules in `self` in topological order--modules with least deps first.
    /// Fails with an error if `self` contains circular dependencies
    pub fn compute_topological_order(&self) -> Result<impl Iterator<Item = &CompiledModule>> {
        match petgraph::algo::toposort(&self.graph, None) {
            Err(_) => bail!("Circular dependency detected"),
            Ok(ordered_idxs) => Ok(ordered_idxs.into_iter().map(move |idx| self.modules[idx.0])),
        }
    }
}
