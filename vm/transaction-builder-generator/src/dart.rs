// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::type_not_allowed;
use libra_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;

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
        r#"
import 'starcoin/starcoin.dart';
import 'serde/serde.dart';
import 'dart:typed_data';
"#,
    )?;
    Ok(())
}

fn output_builder(out: &mut dyn Write, abi: &ScriptABI) -> Result<()> {
    writeln!(
        out,
        "\n{}static Script encode_{}_script({}) {{",
        quote_doc(abi.doc()),
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
    var code = new Bytes(Uint8List.fromList({}));
    var ty_args = List<TypeTag>.filled(1,{});
    var args = List<TransactionArgument>.filled(1,{});
    var script = new Script(code,ty_args,args);
    return script;
}}"#,
        quote_code(abi.code()),
        quote_type_arguments(abi.ty_args()),
        quote_arguments(abi.args()),
    )?;
    Ok(())
}

fn quote_doc(doc: &str) -> String {
    let doc = crate::common::prepare_doc_string(doc);
    let text = textwrap::fill(&doc, 86);
    format!("/**\n{} */\n", textwrap::indent(&text, " * "))
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

fn quote_type_arguments(ty_args: &[TypeArgumentABI]) -> String {
    ty_args
        .iter()
        .map(|ty_arg| ty_arg.name().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn quote_arguments(args: &[ArgumentABI]) -> String {
    args.iter()
        .map(|arg| make_transaction_argument(arg.type_tag(), arg.name()))
        .collect::<Vec<_>>()
        .join(", ")
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
        Address => format!("new TransactionArgumentAddress({})", name),
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
