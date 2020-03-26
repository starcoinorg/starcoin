use compiler::Compiler;

pub struct StarcoinCompiler {}

impl StarcoinCompiler {
    /// Compile script with account address 0x0
    pub fn compile_script(code: &str) -> Vec<u8> {
        unimplemented!()
//        let compiler = Compiler {
//            ..Compiler::default()
//        };
//        compiler.into_script_blob(code).unwrap()
    }
}
