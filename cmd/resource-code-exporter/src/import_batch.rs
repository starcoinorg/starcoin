use bcs_ext;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_statedb::{ChainStateDB, ChainStateReader, ChainStateWriter};
use starcoin_types::state_set::ChainStateSet;
use std::path::Path;
use starcoin_types::access_path::AccessPath;

/// Import ChainStateSet from BCS file to a new statedb with batch processing
/// This is a separate module for batch import functionality
pub fn import_from_statedb_batch(
    statedb: &ChainStateDB,
    bcs_path: &Path,
    expect_state_root_hash: Option<HashValue>,
    batch_size: usize,
) -> anyhow::Result<()> {
    info!("Starting batch import from: {}", bcs_path.display());

    // Read BCS file
    let bcs_data = std::fs::read(bcs_path)?;
    let chain_state_set: ChainStateSet = bcs_ext::from_bytes(&bcs_data)?;

    info!("Loaded {} account states from BCS file", chain_state_set.len());

    // Process in batches to avoid memory issues
    let state_sets: Vec<_> = chain_state_set.state_sets().to_vec();
    let total_batches = (state_sets.len() + batch_size - 1) / batch_size;

    for (batch_idx, chunk) in state_sets.chunks(batch_size).enumerate() {
        info!("Processing batch {}/{} ({} accounts)", batch_idx + 1, total_batches, chunk.len());
        
        // Create a temporary ChainStateSet for this batch
        let batch_chain_state_set = ChainStateSet::new(chunk.to_vec());
        
        // Apply this batch
        match statedb.apply(batch_chain_state_set) {
            Ok(_) => {
                info!("Successfully processed batch {}/{}", batch_idx + 1, total_batches);
            }
            Err(e) => {
                // Try to identify which account caused the error
                for (account_idx, (address, _)) in chunk.iter().enumerate() {
                    info!("Failed account in batch {}: {} (index {})", batch_idx + 1, address, account_idx);
                }
                return Err(anyhow::anyhow!(
                    "Failed to apply batch {}/{}: {}",
                    batch_idx + 1,
                    total_batches,
                    e
                ));
            }
        }
    }

    // Verify the final state root if provided
    if let Some(expected_root) = expect_state_root_hash {
        let actual_root = statedb.state_root();
        if actual_root != expected_root {
            return Err(anyhow::anyhow!(
                "State root mismatch: expected {}, got {}",
                expected_root,
                actual_root
            ));
        }
    }

    info!("Batch import completed successfully");
    Ok(())
}

/// Import ChainStateSet from BCS file with adaptive batch size based on data size
pub fn import_from_statedb_adaptive(
    statedb: &ChainStateDB,
    bcs_path: &Path,
    expect_state_root_hash: Option<HashValue>,
) -> anyhow::Result<()> {
    info!("Starting adaptive batch import from: {}", bcs_path.display());

    // Read BCS file
    let bcs_data = std::fs::read(bcs_path)?;
    let chain_state_set: ChainStateSet = bcs_ext::from_bytes(&bcs_data)?;

    let total_accounts = chain_state_set.len();
    info!("Loaded {} account states from BCS file", total_accounts);

    // Determine batch size based on data size
    let batch_size = if total_accounts > 50000 {
        500  // Small batches for very large datasets
    } else if total_accounts > 10000 {
        1000 // Medium batches for large datasets
    } else {
        2000 // Large batches for smaller datasets
    };

    info!("Using batch size: {} for {} accounts", batch_size, total_accounts);

    // Process in batches
    let state_sets: Vec<_> = chain_state_set.state_sets().to_vec();
    let total_batches = (state_sets.len() + batch_size - 1) / batch_size;

    for (batch_idx, chunk) in state_sets.chunks(batch_size).enumerate() {
        info!("Processing batch {}/{} ({} accounts)", batch_idx + 1, total_batches, chunk.len());
        
        // Create a temporary ChainStateSet for this batch
        let batch_chain_state_set = ChainStateSet::new(chunk.to_vec());
        
        // Apply this batch with retry logic
        let mut retry_count = 0;
        const MAX_RETRIES: usize = 3;
        
        loop {
            match statedb.apply(batch_chain_state_set.clone()) {
                Ok(_) => {
                    info!("Successfully processed batch {}/{}", batch_idx + 1, total_batches);
                    break;
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= MAX_RETRIES {
                        // Try to identify which account caused the error
                        for (account_idx, (address, _)) in chunk.iter().enumerate() {
                            info!("Failed account in batch {}: {} (index {})", batch_idx + 1, address, account_idx);
                        }
                        return Err(anyhow::anyhow!(
                            "Failed to apply batch {}/{} after {} retries: {}",
                            batch_idx + 1,
                            total_batches,
                            MAX_RETRIES,
                            e
                        ));
                    }
                    info!("Retry {}/{} for batch {}/{}", retry_count, MAX_RETRIES, batch_idx + 1, total_batches);
                }
            }
        }
    }

    // Verify the final state root if provided
    if let Some(expected_root) = expect_state_root_hash {
        let actual_root = statedb.state_root();
        if actual_root != expected_root {
            return Err(anyhow::anyhow!(
                "State root mismatch: expected {}, got {}",
                expected_root,
                actual_root
            ));
        }
    }

    info!("Adaptive batch import completed successfully");
    Ok(())
}

pub fn debug_apply_batch_accounts(
    statedb: &ChainStateDB,
    bcs_path: &Path,
    batch_size: usize,
    batch_index: usize, // 0-based
) -> anyhow::Result<()> {
    info!("Debugging batch {} from: {}", batch_index, bcs_path.display());
    let bcs_data = std::fs::read(bcs_path)?;
    let chain_state_set: ChainStateSet = bcs_ext::from_bytes(&bcs_data)?;
    let state_sets: Vec<_> = chain_state_set.state_sets().to_vec();
    let start = batch_index * batch_size;
    let end = ((batch_index + 1) * batch_size).min(state_sets.len());
    info!("Batch {} range: [{}..{})", batch_index, start, end);
    
    for (i, (address, account_state_set)) in state_sets[start..end].iter().enumerate() {
        let batch_idx = start + i;
        // info!("Processing account {} (batch idx {}): {}", address, batch_idx, address);
        
        // Create a new ChainStateDB instance for this account to avoid affecting the original
        let test_statedb = statedb.fork();
        
        // Create a single-account ChainStateSet for testing
        let single_account_chain_state_set = ChainStateSet::new(vec![(*address, account_state_set.clone())]);
        
        // Try to apply this single account and catch any missing node errors
        match test_statedb.apply(single_account_chain_state_set) {
            Ok(_) => {
                //info!("✓ Account {} (batch idx {}) - applied successfully", address, batch_idx);
            }
            Err(e) => {
                error!("[MISSING NODE] Account: {} (batch idx {}), Error: {}", address, batch_idx, e);
                
                // Log the account's code and resource information for debugging
                if let Some(code_set) = account_state_set.code_set() {
                    info!("Account {} has {} code modules", address, code_set.len());
                    for (module_name, _) in code_set.iter() {
                        error!("  - Code module: {:?}", module_name);
                    }
                }
                
                if let Some(resource_set) = account_state_set.resource_set() {
                    info!("Account {} has {} resources", address, resource_set.len());
                    for (struct_tag, _) in resource_set.iter() {
                        error!("  - Resource: {:?}", bcs_ext::from_bytes::<AccessPath>(struct_tag));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Debug function to test commit phase missing node errors by directly manipulating StateTree
pub fn debug_commit_phase_missing_node_direct(
    statedb: &ChainStateDB,
    bcs_path: &Path,
    batch_size: usize,
    batch_index: usize, // 0-based
) -> anyhow::Result<()> {
    info!("Debugging commit phase directly for batch {} from: {}", batch_index, bcs_path.display());
    let bcs_data = std::fs::read(bcs_path)?;
    let chain_state_set: ChainStateSet = bcs_ext::from_bytes(&bcs_data)?;
    let state_sets: Vec<_> = chain_state_set.state_sets().to_vec();
    let start = batch_index * batch_size;
    let end = ((batch_index + 1) * batch_size).min(state_sets.len());
    info!("Batch {} range: [{}..{})", batch_index, start, end);
    
    // Create a new ChainStateDB instance for testing
    let test_statedb = statedb.fork();
    
    // Create a batch ChainStateSet for testing
    let batch_chain_state_set = ChainStateSet::new(state_sets[start..end].to_vec());
    
    // Try to apply this batch and catch any missing node errors
    match test_statedb.apply(batch_chain_state_set) {
        Ok(_) => {
            info!("✓ Batch {} - applied successfully", batch_index);
            
            // Now try to commit explicitly to see if the error occurs here
            match test_statedb.commit() {
                Ok(new_root) => {
                    info!("✓ Batch {} - committed successfully, new root: {}", batch_index, new_root);
                }
                Err(e) => {
                    error!("[COMMIT MISSING NODE] Batch: {} (accounts {}..{}), Error: {}", 
                           batch_index, start, end, e);
                    
                    // Try to identify which accounts might be causing the commit issue
                    for (i, (address, account_state_set)) in state_sets[start..end].iter().enumerate() {
                        let batch_idx = start + i;
                        info!("Account in failed commit batch: {} (batch idx {})", address, batch_idx);
                        
                        if let Some(code_set) = account_state_set.code_set() {
                            info!("  - Has {} code modules", code_set.len());
                        }
                        
                        if let Some(resource_set) = account_state_set.resource_set() {
                            info!("  - Has {} resources", resource_set.len());
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("[APPLY MISSING NODE] Batch: {} (accounts {}..{}), Error: {}", 
                   batch_index, start, end, e);
            
            // Try to identify which accounts might be causing the apply issue
            for (i, (address, account_state_set)) in state_sets[start..end].iter().enumerate() {
                let batch_idx = start + i;
                info!("Account in failed apply batch: {} (batch idx {})", address, batch_idx);
                
                if let Some(code_set) = account_state_set.code_set() {
                    info!("  - Has {} code modules", code_set.len());
                }
                
                if let Some(resource_set) = account_state_set.resource_set() {
                    info!("  - Has {} resources", resource_set.len());
                }
            }
        }
    }
    
    Ok(())
}

/// Debug function to test commit phase missing node errors
pub fn debug_commit_phase_missing_node(
    statedb: &ChainStateDB,
    bcs_path: &Path,
    batch_size: usize,
    batch_index: usize, // 0-based
) -> anyhow::Result<()> {
    info!("Debugging commit phase for batch {} from: {}", batch_index, bcs_path.display());
    let bcs_data = std::fs::read(bcs_path)?;
    let chain_state_set: ChainStateSet = bcs_ext::from_bytes(&bcs_data)?;
    let state_sets: Vec<_> = chain_state_set.state_sets().to_vec();
    let start = batch_index * batch_size;
    let end = ((batch_index + 1) * batch_size).min(state_sets.len());
    info!("Batch {} range: [{}..{})", batch_index, start, end);
    
    // Create a new ChainStateDB instance for testing
    let test_statedb = statedb.fork();
    
    // Create a batch ChainStateSet for testing
    let batch_chain_state_set = ChainStateSet::new(state_sets[start..end].to_vec());
    
    // Try to apply this batch and catch any missing node errors
    match test_statedb.apply(batch_chain_state_set) {
        Ok(_) => {
            info!("✓ Batch {} - applied successfully", batch_index);
            
            // Now try to commit explicitly to see if the error occurs here
            match test_statedb.commit() {
                Ok(new_root) => {
                    info!("✓ Batch {} - committed successfully, new root: {}", batch_index, new_root);
                }
                Err(e) => {
                    error!("[COMMIT MISSING NODE] Batch: {} (accounts {}..{}), Error: {}", 
                           batch_index, start, end, e);
                    
                    // Try to identify which accounts might be causing the commit issue
                    for (i, (address, account_state_set)) in state_sets[start..end].iter().enumerate() {
                        let batch_idx = start + i;
                        info!("Account in failed commit batch: {} (batch idx {})", address, batch_idx);
                        
                        if let Some(code_set) = account_state_set.code_set() {
                            info!("  - Has {} code modules", code_set.len());
                        }
                        
                        if let Some(resource_set) = account_state_set.resource_set() {
                            info!("  - Has {} resources", resource_set.len());
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("[APPLY MISSING NODE] Batch: {} (accounts {}..{}), Error: {}", 
                   batch_index, start, end, e);
            
            // Try to identify which accounts might be causing the apply issue
            for (i, (address, account_state_set)) in state_sets[start..end].iter().enumerate() {
                let batch_idx = start + i;
                info!("Account in failed apply batch: {} (batch idx {})", address, batch_idx);
                
                if let Some(code_set) = account_state_set.code_set() {
                    info!("  - Has {} code modules", code_set.len());
                }
                
                if let Some(resource_set) = account_state_set.resource_set() {
                    info!("  - Has {} resources", resource_set.len());
                }
            }
        }
    }
    
    Ok(())
}