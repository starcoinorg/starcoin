// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
};
use move_model::ty::ReferenceKind;
use move_model::{
    ast::Attribute,
    model::{FunctionEnv, GlobalEnv, Loc, ModuleEnv, QualifiedId, StructId},
    symbol::Symbol,
    ty::{PrimitiveType, Type},
};
use std::rc::Rc;

pub(crate) fn run_extended_checks(env: &GlobalEnv) {
    let checker = ExtendedChecker::new(env);
    checker.run();
}

struct ExtendedChecker<'a> {
    env: &'a GlobalEnv,
    /// The id of the module defining error categories
    #[allow(unused)]
    error_category_module: ModuleId,
}

impl<'a> ExtendedChecker<'a> {
    fn new(env: &'a GlobalEnv) -> Self {
        Self {
            env,
            error_category_module: ModuleId::new(
                AccountAddress::ONE,
                Identifier::new("error").unwrap(),
            ),
        }
    }

    // see implementation in `aptos-core`
    // https://github.com/aptos-labs/aptos-core/blob/3af88bc872221c4958e6163660c60bc07bf53d38/aptos-move/framework/src/extended_checks.rs#L123
    fn run(&self) {
        for ref module in self.env.get_modules() {
            if module.is_target() {
                self.check_entry_functions(module);
                self.check_init_module(module);
                self.build_error_map(module);
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Helper functions

impl<'a> ExtendedChecker<'a> {
    fn has_attribute(&self, fun: &FunctionEnv, attr_name: &str) -> bool {
        self.has_attribute_iter(fun.get_attributes().iter(), attr_name)
    }

    fn has_attribute_iter(
        &self,
        mut attrs: impl Iterator<Item = &'a Attribute>,
        attr_name: &str,
    ) -> bool {
        attrs.any(|attr| {
            if let Attribute::Apply(_, name, _) = attr {
                self.env.symbol_pool().string(*name).as_str() == attr_name
            } else {
                false
            }
        })
    }

    #[allow(unused)]
    fn get_runtime_module_id(&self, module: &ModuleEnv<'_>) -> ModuleId {
        let name = module.get_name();
        let addr = name.addr().expect_numerical();
        let name = Identifier::new(self.name_string(name.name()).to_string()).unwrap();
        ModuleId::new(addr, name)
    }

    #[allow(unused)]
    fn name_string(&self, symbol: Symbol) -> Rc<String> {
        self.env.symbol_pool().string(symbol)
    }
}

// -------------------------------------------------------------------------------------------------
// Module Initialization

impl<'a> ExtendedChecker<'a> {
    fn check_init_module(&self, _module: &ModuleEnv) {
        // TODO(simon): implement me.
    }
}

// -------------------------------------------------------------------------------------------------
// Entry Functions

impl<'a> ExtendedChecker<'a> {
    fn check_entry_functions(&self, module: &ModuleEnv) {
        for ref fun in module.get_functions() {
            if !fun.is_entry() {
                continue;
            }

            if self.has_attribute(fun, "legacy_entry_function") {
                continue;
            }

            self.check_transaction_args(&fun.get_id_loc(), &fun.get_parameter_types());
            if fun.get_return_count() > 0 {
                self.env
                    .error(&fun.get_id_loc(), "entry function cannot return values")
            }
        }
    }

    fn check_transaction_args(&self, loc: &Loc, args: &[Type]) {
        for ty in args {
            self.check_transaction_input_type(loc, ty);
        }
    }

    fn check_transaction_input_type(&self, loc: &Loc, ty: &Type) {
        use Type::*;
        match ty {
            Primitive(_) | TypeParameter(_) => {
                // Any primitive type allowed, any parameter expected to instantiate with primitive
            }
            Reference(ReferenceKind::Immutable, bt)
                if matches!(bt.as_ref(), Primitive(PrimitiveType::Signer)) =>
            {
                // Reference to signer allowed
            }
            Vector(ety) => {
                // Vectors are allowed if element type is allowed
                self.check_transaction_input_type(loc, ety)
            }
            Struct(mid, sid, _) if self.is_allowed_input_struct(mid.qualified(*sid)) => {
                // Specific struct types are allowed
            }
            _ => {
                // Everything else is disallowed.
                self.env.error(
                    loc,
                    &format!(
                        "type `{}` is not supported as a parameter type",
                        ty.display(&self.env.get_type_display_ctx())
                    ),
                );
            }
        }
    }

    fn is_allowed_input_struct(&self, qid: QualifiedId<StructId>) -> bool {
        let name = self.env.get_struct(qid).get_full_name_with_address();
        // TODO(simon) find a nice way to keep this in sync with allowed_structs in starcoin-vm
        matches!(
            name.as_str(),
            "0x1::string::String"
                | "0x1::object::Object"
                | "0x1::option::Option"
                | "0x1::fixed_point32::FixedPoint32"
                | "0x1::fixed_point64::FixedPoint64"
        )
    }
}

// -------------------------------------------------------------------------------------------------
// Build errors map

impl<'a> ExtendedChecker<'a> {
    fn build_error_map(&self, _module: &ModuleEnv) {
        // TODO(simon): implement me.
    }
}
