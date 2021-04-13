// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, TypeTag},
};
use serde_generate::{
    dart,
    indent::{IndentConfig, IndentedWriter},
    CodeGeneratorConfig,
};
use starcoin_vm_types::transaction::{
    ArgumentABI, ScriptABI, ScriptFunctionABI, TransactionScriptABI, TypeArgumentABI,
};

use heck::{CamelCase, ShoutySnakeCase};
use std::{
    collections::BTreeMap,
    io::{Result, Write},
    path::PathBuf,
};

/// Output transaction builders and decoders in Java for the given ABIs.
/// Source files will be located in a subdirectory of `install_dir` corresponding to the given
/// package name (if any, otherwise `install_dir` it self).
pub fn write_source_files(
    install_dir: std::path::PathBuf,
    package_name: &str,
    abis: &[ScriptABI],
) -> Result<()> {
    write_script_call_files(install_dir.clone(), "script", abis)?;
    write_helper_file(install_dir.join("lib"), package_name, abis)
}

/// Output transaction helper functions for the given ABIs.
fn write_helper_file(
    install_dir: std::path::PathBuf,
    package_name: &str,
    abis: &[ScriptABI],
) -> Result<()> {
    let mut dir_path = install_dir;
    let parts = package_name.split('.').collect::<Vec<_>>();
    for part in &parts {
        dir_path = dir_path.join(part);
    }
    std::fs::create_dir_all(&dir_path)?;

    let mut file = std::fs::File::create(dir_path.join("Helpers.dart"))?;
    let mut emitter = DartEmitter {
        out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
        package_name,
    };
    emitter.output_preamble()?;
    writeln!(emitter.out, "\nclass Helpers {{")?;
    emitter.out.indent();

    emitter.output_encode_method()?;
    emitter.output_decode_method()?;

    for abi in abis {
        emitter.output_script_encoder_function(abi)?;
    }
    for abi in common::transaction_script_abis(abis).iter() {
        emitter.output_transaction_script_decoder_function(abi)?;
    }
    for abi in common::script_function_abis(abis).iter() {
        emitter.output_script_function_decoder_function(abi)?;
    }

    emitter.output_transaction_script_encoder_map(&common::transaction_script_abis(abis))?;
    emitter.output_script_function_encoder_map(&common::script_function_abis(abis))?;
    for abi in common::transaction_script_abis(abis) {
        emitter.output_code_constant(&abi)?;
    }
    // Must be defined after the constants.
    emitter.output_transaction_script_decoder_map(&common::transaction_script_abis(abis))?;
    emitter.output_script_function_decoder_map(&common::script_function_abis(abis))?;

    emitter.output_decoding_helpers(abis)?;

    emitter.out.unindent();
    writeln!(emitter.out, "\n}}\n")?;

    emitter.output_map()
}

fn write_script_call_files(
    install_dir: std::path::PathBuf,
    package_name: &str,
    abis: &[ScriptABI],
) -> Result<()> {
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert(
        "import".to_string(),
        vec!["../starcoin_types/starcoin_types.dart".to_string()],
    );

    let (transaction_script_abis, script_fun_abis): (Vec<_>, Vec<_>) = abis
        .iter()
        .cloned()
        .partition(|abi| abi.is_transaction_script_abi());

    let mut script_registry: BTreeMap<_, _> = vec![(
        "ScriptCall".to_string(),
        common::make_abi_enum_container(transaction_script_abis.as_slice()),
    )]
    .into_iter()
    .collect();

    let mut script_function_registry: BTreeMap<_, _> = vec![(
        "ScriptFunctionCall".to_string(),
        common::make_abi_enum_container(script_fun_abis.as_slice()),
    )]
    .into_iter()
    .collect();

    script_registry.append(&mut script_function_registry);
    let mut comments: BTreeMap<_, _> = abis
        .iter()
        .map(|abi| {
            let mut paths = package_name
                .split('.')
                .map(String::from)
                .collect::<Vec<_>>();
            paths.push(
                if abi.is_transaction_script_abi() {
                    "ScriptCall"
                } else {
                    "ScriptFunctionCall"
                }
                .to_string(),
            );
            paths.push(abi.name().to_camel_case());
            (paths, prepare_doc_string(abi.doc()))
        })
        .collect();
    comments.insert(
        vec!["ScriptCall".to_string()],
        "Structured representation of a call into a known Move script.".into(),
    );
    comments.insert(
        vec!["ScriptFunctionCall".to_string()],
        "Structured representation of a call into a known Move script function.".into(),
    );

    let config = CodeGeneratorConfig::new(package_name.to_string())
        .with_comments(comments)
        .with_external_definitions(external_definitions)
        .with_serialization(false);
    dart::CodeGenerator::new(&config)
        .output(install_dir, &script_registry)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err)))?;
    Ok(())
}

/// Add minimal escaping for Javadoc.
fn prepare_doc_string(doc: &str) -> String {
    let doc = crate::common::prepare_doc_string(doc);
    // Escape quoted strings and special characters.
    let doc = regex::Regex::new("`([^`]*)`|(>=)|(<=)|(>)|(<)")
        .unwrap()
        .replace_all(&doc, "{@code $1$2$3$4$5}");
    // Replace subsection titles.
    let doc = regex::Regex::new("##* (.*)\n")
        .unwrap()
        .replace_all(&doc, "<p><b>$1</b></p>\n");
    // Simulate lists.
    let doc = regex::Regex::new("\n\\* (.*)")
        .unwrap()
        .replace_all(&doc, "\n<ul><li>$1</li></ul>");
    doc.to_string()
}

/// Shared state for the Java code generator.
struct DartEmitter<'a, T> {
    /// Writer.
    out: IndentedWriter<T>,
    /// Name of the package owning the generated definitions (e.g. "com.my_org.my_package")
    package_name: &'a str,
}

impl<'a, T> DartEmitter<'a, T>
where
    T: Write,
{
    fn output_preamble(&mut self) -> Result<()> {
        writeln!(self.out, "library {};\n", self.package_name)?;
        writeln!(
            self.out,
            r#"
import '../serde/serde.dart';
import '../script/script.dart';
import '../starcoin_types/starcoin_types.dart';
"#,
        )?;
        Ok(())
    }

    fn output_encode_method(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
/**
 * Build a Diem {{@link com.diem.types.Script}} from a structured value {{@link ScriptCall}}.
 *
 * @param call {{@link ScriptCall}} value to encode.
 * @return Encoded script.
 */
static Script encode_script(ScriptCall call) {{
    ScriptEncodingHelper helper = TRANSACTION_SCRIPT_ENCODER_MAP[call.runtimeType.toString()];
    return helper(call);
}}"#
        )?;
        writeln!(
            self.out,
            r#"
/**
 * Build a Diem {{@link com.diem.types.TransactionPayload}} from a structured value {{@link ScriptFunctionCall}}.
 *
 * @param call {{@link ScriptFunctionCall}} value to encode.
 * @return Encoded TransactionPayload.
 */
static TransactionPayload encode_script_function(ScriptFunctionCall call) {{
    ScriptFunctionEncodingHelper helper = SCRIPT_FUNCTION_ENCODER_MAP[call.runtimeType.toString()];
    return helper(call);
}}"#
        )
    }

    fn output_decode_method(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
/**
 * Try to recognize a Diem {{@link com.diem.types.Script}} and convert it into a structured value {{@code ScriptCall}}.
 *
 * @param script {{@link com.diem.types.Script}} values to decode.
 * @return Decoded {{@link ScriptCall}} value.
 */
static ScriptCall decode_script(Script script)  {{
    TransactionScriptDecodingHelper helper = TRANSACTION_SCRIPT_DECODER_MAP[script.code];
    if (helper == null) {{
        throw new Exception("Unknown script bytecode");
    }}
    return helper(script);
}}
"#
        )?;

        writeln!(
            self.out,
            r#"
/**
 * Try to recognize a Diem {{@link com.diem.types.TransactionPayload}} and convert it into a structured value {{@code ScriptFunctionCall}}.
 *
 * @param payload {{@link com.diem.types.TransactionPayload}} values to decode.
 * @return Decoded {{@link ScriptFunctionCall}} value.
 */
static ScriptFunctionCall decode_script_function_payload(TransactionPayload payload) {{
    if (payload is TransactionPayloadScriptFunctionItem) {{
        ScriptFunction script = (payload as TransactionPayloadScriptFunctionItem).value;
        ScriptFunctionDecodingHelper helper = SCRIPT_FUNCTION_DECODER_MAP[script.module.name.value + script.function.value];
        if (helper == null) {{
            throw new Exception("Unknown script function");
        }}
        return helper(payload);
    }} else {{
        throw new Exception("Unknown transaction payload");
    }}
}}
"#
        )
    }

    fn emit_transaction_script_encoder(
        &mut self,
        abi: &TransactionScriptABI,
        quoted_type_params: Vec<String>,
        quoted_type_params_doc: Vec<String>,
        quoted_params: Vec<String>,
        quoted_params_doc: Vec<String>,
    ) -> Result<()> {
        writeln!(
            self.out,
            "\n{}static Script encode_{}_script({}) {{",
            Self::quote_doc(
                abi.doc(),
                [quoted_type_params_doc, quoted_params_doc].concat(),
                "Encoded {@link com.diem.types.Script} value.",
            ),
            abi.name(),
            [quoted_type_params, quoted_params].concat().join(", ")
        )?;
        self.out.indent();
        writeln!(
            self.out,
            r#"Script.Builder builder = new Script.Builder();
builder.code = new Bytes({}_CODE);
builder.ty_args = [{}];
builder.args = [{}];
return builder.build();"#,
            abi.name().to_shouty_snake_case(),
            Self::quote_type_arguments(abi.ty_args()),
            Self::quote_arguments(abi.args()),
        )?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn emit_script_function_encoder(
        &mut self,
        abi: &ScriptFunctionABI,
        quoted_type_params: Vec<String>,
        quoted_type_params_doc: Vec<String>,
        quoted_params: Vec<String>,
        quoted_params_doc: Vec<String>,
    ) -> Result<()> {
        writeln!(
            self.out,
            "\n{}static TransactionPayload encode_{}_script_function({}) {{",
            Self::quote_doc(
                abi.doc(),
                [quoted_type_params_doc, quoted_params_doc].concat(),
                "Encoded {@link com.diem.types.TransactionPayload} value.",
            ),
            abi.name(),
            [quoted_type_params, quoted_params].concat().join(", ")
        )?;
        self.out.indent();
        writeln!(
            self.out,
            r#"var ty_args = [{}];
var args = [{}];
var function = {};
var module = {};

return TransactionPayloadScriptFunctionItem(ScriptFunction(module,function,ty_args,args));"#,
            Self::quote_type_arguments(abi.ty_args()),
            Self::quote_arguments(abi.args()),
            Self::quote_identifier(abi.name()),
            Self::quote_module_id(abi.module_name()),
        )?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_script_encoder_function(&mut self, abi: &ScriptABI) -> Result<()> {
        let quoted_type_params: Vec<_> = abi
            .ty_args()
            .iter()
            .map(|ty_arg| format!("TypeTag {}", ty_arg.name()))
            .collect();
        let quoted_type_params_doc: Vec<_> = abi
            .ty_args()
            .iter()
            .map(|ty_arg| format!("{} {{TypeTag}} value", ty_arg.name()))
            .collect();
        let quoted_params: Vec<_> = abi
            .args()
            .iter()
            .map(|arg| format!("{} {}", Self::quote_type(arg.type_tag()), arg.name()))
            .collect();
        let quoted_params_doc: Vec<_> = abi
            .args()
            .iter()
            .map(|arg| {
                format!(
                    "{} {{ {}}} value",
                    arg.name(),
                    Self::quote_type(arg.type_tag())
                )
            })
            .collect();
        match abi {
            ScriptABI::TransactionScript(abi) => self.emit_transaction_script_encoder(
                abi,
                quoted_type_params,
                quoted_type_params_doc,
                quoted_params,
                quoted_params_doc,
            ),
            ScriptABI::ScriptFunction(abi) => self.emit_script_function_encoder(
                abi,
                quoted_type_params,
                quoted_type_params_doc,
                quoted_params,
                quoted_params_doc,
            ),
        }
    }

    fn output_transaction_script_decoder_function(
        &mut self,
        abi: &TransactionScriptABI,
    ) -> Result<()> {
        writeln!(
            self.out,
            "\nstatic ScriptCall decode_{}_script(Script {}script) throws Exception, IndexOutOfBoundsException {{",
            abi.name(),
            // prevent warning "unused variable"
            if abi.ty_args().is_empty() && abi.args().is_empty() {
                "_"
            } else {
                ""
            }
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "ScriptCall.{0}.Builder builder = new ScriptCall.{0}.Builder();",
            abi.name().to_camel_case(),
        )?;
        for (index, ty_arg) in abi.ty_args().iter().enumerate() {
            writeln!(
                self.out,
                "builder.{} = script.ty_args.get({});",
                ty_arg.name(),
                index,
            )?;
        }
        for (index, arg) in abi.args().iter().enumerate() {
            writeln!(
                self.out,
                "builder.{} = Helpers.decode_{}_argument(script.args.get({}));",
                arg.name(),
                common::mangle_type(arg.type_tag()),
                index,
            )?;
        }
        writeln!(self.out, "return builder.build();")?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_script_function_decoder_function(&mut self, abi: &ScriptFunctionABI) -> Result<()> {
        // `payload` is always used, so don't need to add `"_"` prefix
        writeln!(
            self.out,
            "\nstatic ScriptFunctionCall decode_{}_script_function(TransactionPayload payload) {{",
            abi.name(),
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "if(!(payload is TransactionPayloadScriptFunctionItem)) {{
                throw new Exception(\"Transaction payload not a Script Function\");
        }}"
        )?;
        writeln!(
            self.out,
            "ScriptFunction {}script = (payload as TransactionPayloadScriptFunctionItem).value;",
            // prevent warning "unused variable"
            if abi.ty_args().is_empty() && abi.args().is_empty() {
                "_"
            } else {
                ""
            }
        )?;
        // writeln!(
        //     self.out,
        //     "ScriptFunctionCall.{0}.Builder builder = new ScriptFunctionCall.{0}.Builder();",
        //     abi.name().to_camel_case(),
        // )?;
        for (index, ty_arg) in abi.ty_args().iter().enumerate() {
            writeln!(
                self.out,
                "var {} = script.ty_args[{}];",
                ty_arg.name(),
                index,
            )?;
        }
        for (index, arg) in abi.args().iter().enumerate() {
            writeln!(
                self.out,
                "var {} = Helpers.decode_{}_argument(script.args[{}]);",
                arg.name(),
                common::mangle_type(arg.type_tag()),
                index,
            )?;
        }
        writeln!(
            self.out,
            "return ScriptFunctionCall{0}Item(",
            abi.name().to_camel_case()
        )?;
        self.out.indent();

        for (_, ty_arg) in abi.ty_args().iter().enumerate() {
            writeln!(self.out, "{},", ty_arg.name(),)?;
        }
        for (_, arg) in abi.args().iter().enumerate() {
            writeln!(self.out, "{},", arg.name(),)?;
        }
        self.out.unindent();
        writeln!(self.out, ");")?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_transaction_script_encoder_map(
        &mut self,
        abis: &[TransactionScriptABI],
    ) -> Result<()> {
        writeln!(
            self.out,
            r#"

static final Map<String, ScriptEncodingHelper> TRANSACTION_SCRIPT_ENCODER_MAP = initTransactionScriptEncoderMap();

static Map<String, ScriptEncodingHelper> initTransactionScriptEncoderMap() {{"#
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "Map<String, ScriptEncodingHelper> map = new Map();"
        )?;
        for abi in abis {
            let params = std::iter::empty()
                .chain(abi.ty_args().iter().map(TypeArgumentABI::name))
                .chain(abi.args().iter().map(ArgumentABI::name))
                .map(|name| format!("obj.{}", name))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(
                self.out,
                "map[(ScriptCall{0}Item).toString()]=(call) {{
    ScriptCall{0}Item obj = (ScriptCall{0}Item)call;
    return Helpers.encode_{1}_script({2});
}};",
                abi.name().to_camel_case(),
                abi.name(),
                params,
            )?;
        }
        writeln!(self.out, "return map;")?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_script_function_encoder_map(&mut self, abis: &[ScriptFunctionABI]) -> Result<()> {
        writeln!(
            self.out,
            r#"

static final Map<String, ScriptFunctionEncodingHelper> SCRIPT_FUNCTION_ENCODER_MAP = initScriptFunctionEncoderMap();

static Map<String, ScriptFunctionEncodingHelper> initScriptFunctionEncoderMap() {{"#
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "Map<String, ScriptFunctionEncodingHelper> map = new Map();"
        )?;
        for abi in abis {
            let params = std::iter::empty()
                .chain(abi.ty_args().iter().map(TypeArgumentABI::name))
                .chain(abi.args().iter().map(ArgumentABI::name))
                .map(|name| format!("obj.{}", name))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(
                self.out,
                "map[(ScriptFunctionCall{0}Item).toString()]= (call) {{
    ScriptFunctionCall{0}Item obj = call as ScriptFunctionCall{0}Item;
    return Helpers.encode_{1}_script_function({2});
}};",
                abi.name().to_camel_case(),
                abi.name(),
                params,
            )?;
        }
        writeln!(self.out, "return map;")?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_transaction_script_decoder_map(
        &mut self,
        abis: &[TransactionScriptABI],
    ) -> Result<()> {
        writeln!(
            self.out,
            r#"

static final Map<Bytes, TransactionScriptDecodingHelper> TRANSACTION_SCRIPT_DECODER_MAP = initTransactionScriptDecoderMap();

static Map<Bytes, TransactionScriptDecodingHelper> initTransactionScriptDecoderMap() {{"#
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "Map<Bytes, TransactionScriptDecodingHelper> map = new Map();"
        )?;
        for abi in abis {
            writeln!(
                self.out,
                "map[new Bytes({}_CODE)]=(script) => Helpers.decode_{}_script(script);",
                abi.name().to_shouty_snake_case(),
                abi.name()
            )?;
        }
        writeln!(self.out, "return map;")?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_script_function_decoder_map(&mut self, abis: &[ScriptFunctionABI]) -> Result<()> {
        writeln!(
            self.out,
            r#"

static final Map<String, ScriptFunctionDecodingHelper> SCRIPT_FUNCTION_DECODER_MAP = initDecoderMap();

static Map<String, ScriptFunctionDecodingHelper> initDecoderMap() {{"#
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "Map<String, ScriptFunctionDecodingHelper> map = new Map();"
        )?;
        for abi in abis {
            writeln!(
                self.out,
                "map[\"{0}{1}\"]=(payload) => Helpers.decode_{1}_script_function(payload);",
                abi.module_name().name(),
                abi.name()
            )?;
        }
        writeln!(self.out, "return map;")?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_map(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
typedef ScriptFunctionEncodingHelper = TransactionPayload Function(
      ScriptFunctionCall call);
            
typedef ScriptFunctionDecodingHelper = ScriptFunctionCall Function(
      TransactionPayload call);

typedef TransactionScriptDecodingHelper = ScriptCall Function(
      Script script);

typedef ScriptEncodingHelper = Script Function(
      ScriptCall call);
"#
        )
    }

    fn output_decoding_helpers(&mut self, abis: &[ScriptABI]) -> Result<()> {
        let required_types = common::get_required_decoding_helper_types(abis);
        for required_type in required_types {
            self.output_decoding_helper(required_type)?;
        }
        Ok(())
    }

    fn output_decoding_helper(&mut self, type_tag: &TypeTag) -> Result<()> {
        use TypeTag::*;
        let (constructor, expr) = match type_tag {
            Bool => ("Bool", String::new()),
            U8 => ("U8", String::new()),
            U64 => ("U64", String::new()),
            U128 => ("U128", String::new()),
            Address => ("Address", String::new()),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => ("U8Vector", String::new()),
                _ => common::type_not_allowed(type_tag),
            },
            Struct(_) | Signer => common::type_not_allowed(type_tag),
        };
        writeln!(
            self.out,
            r#"
static {} decode_{}_argument(TransactionArgument arg) {{
    if (!(arg is TransactionArgument{}Item)) {{
        throw new Exception("Was expecting a {} argument");
    }}
    return (arg as TransactionArgument{}Item).value{};
}}
"#,
            Self::quote_type(type_tag),
            common::mangle_type(type_tag),
            constructor,
            constructor,
            constructor,
            expr,
        )
    }

    fn output_code_constant(&mut self, abi: &TransactionScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\nstatic byte[] {}_CODE = {{{}}};",
            abi.name().to_shouty_snake_case(),
            abi.code()
                .iter()
                .map(|x| format!("{}", *x as i8))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        Ok(())
    }

    fn quote_identifier(ident: &str) -> String {
        format!("new Identifier(\"{}\")", ident)
    }

    fn quote_address(address: &AccountAddress) -> String {
        format!(
            "AccountAddress([ {} ])",
            address
                .to_vec()
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn quote_module_id(module_id: &ModuleId) -> String {
        format!(
            "new ModuleId({}, {})",
            Self::quote_address(module_id.address()),
            Self::quote_identifier(module_id.name().as_str())
        )
    }

    fn quote_doc<I>(doc: &str, params_doc: I, return_doc: &str) -> String
    where
        I: IntoIterator<Item = String>,
    {
        let mut doc = prepare_doc_string(doc) + "\n";
        for text in params_doc {
            doc = format!("{}\n@param {}", doc, text);
        }
        doc = format!("{}\n@return {}", doc, return_doc);
        let text = textwrap::indent(&doc, " * ").replace("\n\n", "\n *\n");
        format!("/**\n{}\n */\n", text)
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
            .map(|arg| Self::quote_transaction_argument(arg.type_tag(), arg.name()))
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
                _ => common::type_not_allowed(type_tag),
            },

            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
    }

    fn quote_transaction_argument(type_tag: &TypeTag, name: &str) -> String {
        use TypeTag::*;
        match type_tag {
            Bool => format!("new TransactionArgumentBoolItem({})", name),
            U8 => format!("new TransactionArgumentU8Item({})", name),
            U64 => format!("new TransactionArgumentU64Item({})", name),
            U128 => format!("new TransactionArgumentU128Item({})", name),
            Address => format!("new TransactionArgumentAddressItem({})", name),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => format!("new TransactionArgumentU8VectorItem({})", name),
                _ => common::type_not_allowed(type_tag),
            },

            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
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
        package_name: &str,
        abis: &[ScriptABI],
    ) -> std::result::Result<(), Self::Error> {
        write_source_files(self.install_dir.clone(), package_name, abis)?;
        Ok(())
    }
}
