use anyhow::Result;
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{Bytecode, CompiledScript};
use move_binary_format::file_format_common::{VERSION_3, VERSION_4, VERSION_6};
use move_binary_format::CompiledModule;

pub struct ModuleBytecodeDowngrader;
impl ModuleBytecodeDowngrader {
    pub fn to_v3(m: &CompiledModule) -> Result<Vec<u8>> {
        match m.version {
            VERSION_4 => Self::from_v4_to_v3(m),
            VERSION_3 => {
                let mut bytes = vec![];
                m.serialize_for_version(Some(VERSION_3), &mut bytes)?;
                Ok(bytes)
            }
            _ => anyhow::bail!("unsupport module bytecode version {}", m.version),
        }
    }
    pub fn from_v4_to_v3(m: &CompiledModule) -> Result<Vec<u8>> {
        anyhow::ensure!(m.version() == VERSION_4, "bytecode version is not v4");

        for fd in &m.function_defs {
            if let Some(cu) = &fd.code {
                for c in &cu.code {
                    if matches!(
                        c,
                        Bytecode::VecLen(_)
                            | Bytecode::VecPack(_, _)
                            | Bytecode::VecUnpack(_, _)
                            | Bytecode::VecSwap(_)
                            | Bytecode::VecPopBack(_)
                            | Bytecode::VecPushBack(_)
                            | Bytecode::VecImmBorrow(_)
                            | Bytecode::VecMutBorrow(_)
                    ) {
                        anyhow::bail!(
                            "module {:?} contains vec bytecodes introduced in v4 bytecode version",
                            m.self_id()
                        );
                    }
                }
            }
        }

        let mut bytes = vec![];
        m.serialize_for_version(Some(VERSION_3), &mut bytes)?;
        Ok(bytes)
    }

    pub fn to_v4(m: &CompiledModule) -> Result<Vec<u8>> {
        match m.version {
            VERSION_6 => Self::from_v6_to_v4(m),
            VERSION_4 => {
                let mut bytes = vec![];
                m.serialize_for_version(Some(VERSION_4), &mut bytes)?;
                Ok(bytes)
            }
            _ => anyhow::bail!("unsupport module bytecode version {}", m.version),
        }
    }
    pub fn from_v6_to_v4(m: &CompiledModule) -> Result<Vec<u8>> {
        anyhow::ensure!(m.version() == VERSION_6, "bytecode version is not v6");

        for fd in &m.function_defs {
            if let Some(cu) = &fd.code {
                for c in &cu.code {
                    if matches!(
                        c,
                        Bytecode::LdU16(_)
                            | Bytecode::LdU32(_)
                            | Bytecode::LdU256(_)
                            | Bytecode::CastU16
                            | Bytecode::CastU32
                            | Bytecode::CastU256
                    ) {
                        anyhow::bail!(
                            "module {:?} contains bytecodes introduced in v6 bytecode version",
                            m.self_id()
                        );
                    }
                }
            }
        }

        let mut bytes = vec![];
        m.serialize_for_version(Some(VERSION_4), &mut bytes)?;
        Ok(bytes)
    }
}

pub struct ScriptBytecodeDowgrader;
impl ScriptBytecodeDowgrader {
    pub fn from_v4_to_v3(s: &CompiledScript) -> Result<Vec<u8>> {
        anyhow::ensure!(s.version == VERSION_4, "bytecode version is not v4");

        for c in &s.code.code {
            if matches!(
                c,
                Bytecode::VecLen(_)
                    | Bytecode::VecPack(_, _)
                    | Bytecode::VecUnpack(_, _)
                    | Bytecode::VecSwap(_)
                    | Bytecode::VecPopBack(_)
                    | Bytecode::VecPushBack(_)
                    | Bytecode::VecImmBorrow(_)
                    | Bytecode::VecMutBorrow(_)
            ) {
                anyhow::bail!("script contains vec bytecodes introduced in v4 bytecode version",);
            }
        }

        let mut bytes = vec![];
        s.serialize_for_version(Some(VERSION_3), &mut bytes)?;
        Ok(bytes)
    }

    pub fn from_v6_to_v4(s: &CompiledScript) -> Result<Vec<u8>> {
        anyhow::ensure!(s.version == VERSION_6, "bytecode version is not v6");

        for c in &s.code.code {
            if matches!(
                c,
                Bytecode::LdU16(_)
                    | Bytecode::LdU32(_)
                    | Bytecode::LdU256(_)
                    | Bytecode::CastU16
                    | Bytecode::CastU32
                    | Bytecode::CastU256
            ) {
                anyhow::bail!("script contains bytecodes introduced in v6 bytecode version",);
            }
        }

        let mut bytes = vec![];
        s.serialize_for_version(Some(VERSION_4), &mut bytes)?;
        Ok(bytes)
    }
}
