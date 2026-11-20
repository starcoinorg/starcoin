//! String parameters JSON-RPC callers send: type tags, struct tags, transaction arguments,
//! function and module ids and access paths, parsed through the RPC view types, then rendered
//! back (RPC echoes them) and, for type tags, resolved to an ABI against the stdlib modules as
//! `contract.resolve_struct_tag` and pubsub event decoding do.
#![no_main]

use bcs_ext::BCSCodec;
use libfuzzer_sys::fuzz_target;
use starcoin_abi_resolver::ABIResolver;
use starcoin_fuzz::StdlibStateView;
use starcoin_rpc_api::types::{
    ByteCodeOrScriptFunction, FunctionIdView, ModuleIdView, StructTagView, TransactionArgumentView,
    TypeTagView,
};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::language_storage::TypeTag;
use starcoin_vm_types::state_store::state_key::StateKey;
use std::str::FromStr;

thread_local! {
    static STATE: &'static StdlibStateView = Box::leak(Box::new(StdlibStateView::load()));
}

fn stdlib_state() -> &'static StdlibStateView {
    STATE.with(|state| *state)
}

fn resolve(type_tag: &TypeTag) {
    type_tag.encode().expect("a parsed type tag must encode");
    let _ = ABIResolver::new(stdlib_state()).resolve_type_tag(type_tag);
}

fuzz_target!(|data: &[u8]| {
    let Some((selector, bytes)) = data.split_first() else {
        return;
    };
    let Ok(s) = std::str::from_utf8(bytes) else {
        return;
    };
    match selector % 7 {
        0 => {
            if let Ok(t) = TypeTagView::from_str(s) {
                t.to_string();
                resolve(&t.0);
            }
        }
        1 => {
            if let Ok(t) = StructTagView::from_str(s) {
                t.to_string();
                let _ = ABIResolver::new(stdlib_state()).resolve_struct_tag(&t.0);
            }
        }
        2 => {
            if let Ok(tags) = starcoin_vm_types::parser::parse_type_tags(s) {
                tags.iter().for_each(resolve);
            }
        }
        3 => {
            if let Ok(arg) = TransactionArgumentView::from_str(s) {
                arg.to_string();
                arg.0.encode().expect("a parsed argument must encode");
            }
            let _ = starcoin_types::transaction::parse_transaction_argument_advance(s);
        }
        4 => {
            if let Ok(f) = FunctionIdView::from_str(s) {
                f.to_string();
                let _ = ABIResolver::new(stdlib_state())
                    .resolve_function(&f.0.module, f.0.function.as_ident_str());
            }
            let _ = ByteCodeOrScriptFunction::from_str(s);
        }
        5 => {
            if let Ok(m) = ModuleIdView::from_str(s) {
                m.to_string();
            }
        }
        _ => {
            if let Ok(p) = AccessPath::from_str(s) {
                p.to_string();
                StateKey::AccessPath(p)
                    .encode()
                    .expect("a parsed access path must encode");
            }
        }
    }
});
