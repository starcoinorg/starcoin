// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::type_not_allowed;
use move_core_types::language_storage::TypeTag;
use starcoin_vm_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};

use std::{
    io::{Result, Write},
    path::PathBuf,
};

/// Output transaction builders in Java for the given ABIs.
pub fn output(out: &mut dyn Write, abis: &[ScriptABI], class_name: &str) -> Result<()> {
    output_preamble(out)?;
    writeln!(out, "\nclass {} {{\n", class_name)?;
    for abi in abis {
        output_builder(out, abi)?;
    }
    writeln!(out, "\n}}\n")?;
    Ok(())
}

fn output_preamble(out: &mut dyn Write) -> Result<()> {
    writeln!(
        out,
        r#"import 'starcoin/starcoin.dart';
import 'serde/serde.dart';
import 'dart:typed_data';
"#,
    )?;
    Ok(())
}

fn output_builder(out: &mut dyn Write, abi: &ScriptABI) -> Result<()> {
    writeln!(
        out,
        "\n\tstatic Script encode_{}_script({}) {{",
        abi.name(),
        [
            quote_type_parameters(abi.ty_args()),
            quote_parameters(abi.args()),
        ]
        .concat()
        .join(", ")
    )?;
    writeln!(
        out,
        r#"
    var code = new Bytes(Uint8List.fromList({}));"#,
        quote_code(abi.code()),
    )?;
    if abi.ty_args().is_empty() {
        writeln!(out, "\t\tvar ty_args = List<TypeTag>();")?;
    } else {
        writeln!(
            out,
            "\t\tList<TypeTag> ty_args = new List<TypeTag>({});",
            abi.ty_args().len(),
        )?;
        for (index, arg) in abi.ty_args().iter().enumerate() {
            writeln!(out, "\t\tty_args[{}] = {};", index, arg.name())?;
        }
    }

    if abi.args().is_empty() {
        writeln!(out, "\t\tvar args = List<TransactionArgument>();")?;
    } else {
        writeln!(
            out,
            "\t\tList<TransactionArgument> args = new List<TransactionArgument>({});",
            abi.args().len(),
        )?;
        for (index, arg) in abi.args().iter().enumerate() {
            writeln!(
                out,
                "\t\targs[{}] = {};",
                index,
                make_transaction_argument(arg.type_tag(), arg.name())
            )?;
        }
    }

    writeln!(
        out,
        r#"
    var script = new Script(code,ty_args,args);
    return script;
  }}"#
    )?;
    Ok(())
}

fn quote_type_parameters(ty_args: &[TypeArgumentABI]) -> Vec<String> {
    ty_args
        .iter()
        .map(|ty_arg| format!("TypeTag {}", ty_arg.name()))
        .collect()
}

fn quote_parameters(args: &[ArgumentABI]) -> Vec<String> {
    args.iter()
        .map(|arg| format!("{} {}", quote_type(arg.type_tag()), arg.name()))
        .collect()
}

fn quote_code(code: &[u8]) -> String {
    format!(
        "[{}]",
        code.iter()
            .map(|x| format!("{}", *x as i8))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn quote_type(type_tag: &TypeTag) -> String {
    use TypeTag::*;
    match type_tag {
        Bool => "bool".into(),
        U8 => "int".into(),
        U64 => "int".into(),
        U128 => "Int128".into(),
        Address => "AccountAddress".into(),
        Vector(type_tag) => match type_tag.as_ref() {
            U8 => "Bytes".into(),
            _ => type_not_allowed(type_tag),
        },

        Struct(_) | Signer => type_not_allowed(type_tag),
    }
}

fn make_transaction_argument(type_tag: &TypeTag, name: &str) -> String {
    use TypeTag::*;
    match type_tag {
        Bool => format!("new TransactionArgumentBoolItem({})", name),
        U8 => format!("new TransactionArgumentU8Item({})", name),
        U64 => format!("new TransactionArgumentU64Item({})", name),
        U128 => format!("new TransactionArgumentU128Item({})", name),
        Address => format!("new TransactionArgumentAddressItem({})", name),
        Vector(type_tag) => match type_tag.as_ref() {
            U8 => format!("new TransactionArgumentU8VectorItem({})", name),
            _ => type_not_allowed(type_tag),
        },

        Struct(_) | Signer => type_not_allowed(type_tag),
    }
}

pub struct Installer {
    install_dir: PathBuf,
}

impl Installer {
    pub fn new(install_dir: PathBuf) -> Self {
        Installer { install_dir }
    }
}

impl crate::SourceInstaller for Installer {
    type Error = Box<dyn std::error::Error>;

    fn install_transaction_builders(
        &self,
        name: &str,
        abis: &[ScriptABI],
    ) -> std::result::Result<(), Self::Error> {
        let parts = name.split('.').collect::<Vec<_>>();
        let mut dir_path = self.install_dir.clone();
        dir_path = dir_path.join("lib");
        let class_name = parts.last().unwrap().to_string();

        std::fs::create_dir_all(&dir_path)?;

        let mut file = std::fs::File::create(dir_path.join(class_name.clone() + ".dart"))?;
        output(&mut file, abis, &class_name)?;
        Ok(())
    }
}
