use crate::ast::BinOp;
use ecow::EcoString;
use im::HashMap;
use std::cell::RefCell;
use std::{fmt::Debug, sync::Arc};

use crate::ast::{Assignment, CallArg, CustomType, Definition, Function, Pattern, Statement, TypedExpr};
use crate::type_::{ModuleInterface, Type};
//TODO non-ascii names and upper-case var names.
//TODO i32 widening? Check Gleam expectations.

pub trait Wasmable {
    fn to_wat(&self) -> EcoString;
}

#[derive(Clone, Debug)]
enum WasmType {
    I32,
    ConcreteRef(WasmVar),

}

#[derive(Debug)]
enum WasmTypeSectionEntry {
    PlaceHolder(EcoString),
    Function(WasmFuncDef),
    Struct(WasmStructDef),
}

impl WasmTypeSectionEntry {
    fn public(&self) -> bool {
        match self {
            WasmTypeSectionEntry::Function(f) => f.exported,
            _ => false //TODO for structs etc.
        }
    }
}

impl Wasmable for WasmTypeSectionEntry {
    fn to_wat(&self) -> EcoString {
        match self {
            WasmTypeSectionEntry::PlaceHolder(_) => { panic!() }
            WasmTypeSectionEntry::Function(x) => { x.to_wat() } //TODO see if can be removed!
            WasmTypeSectionEntry::Struct(x) => { x.to_wat() }
        }
    }
}

#[derive(Clone, Debug)]
struct WasmFuncDef {
    info: WasmVar,
    return_type: WasmType,
    exported: bool,
}

impl Wasmable for WasmFuncDef {
    fn to_wat(&self) -> EcoString {
        "".into() //TODO I don't think we need the sections in wat
    }
}

#[derive(Debug, Clone)] //TODO not sure needs clone but eh...
struct WasmStructDef {
    info: WasmVar,
    fields: Vec<(WasmVar, WasmType)>,
}

impl Wasmable for WasmStructDef {
    fn to_wat(&self) -> EcoString {
        let mut acc = format!("(type ${} (struct", self.info.name);
        self.fields.iter().for_each(|f|
            acc.push_str(&mut format!(" (field ${} {})", f.0.name, f.1.to_wat()))
        );
        acc.push_str("))"); //TODO move somewhere else the \n?
        acc.into()
    }
}

impl Wasmable for WasmType {
    fn to_wat(&self) -> EcoString {
        match self {
            WasmType::I32 => "i32".into(),
            WasmType::ConcreteRef(x) => {
                //TODO the ref idx, doesn't work for wat, since functions are gone.....
                // format!("(ref {}  (;{};))", index, x.name).into()
                // but we don't have the full info here hmmmmmmmmmmmmmmmmmm
                format!("(ref ${})", x.name).into()
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct WasmVar {
    //TODO rename to WasmIndex?
    // idx: u32,
    name: EcoString,
}

impl Wasmable for WasmVar {
    fn to_wat(&self) -> EcoString {
        format!("${}", self.name).into()
    }
}

#[derive(Debug)]
struct WasmFunction {
    // info: WasmVar,
    args: Vec<(WasmVar, WasmType)>,
    def: WasmFuncDef,
    body: Vec<WasmInstruction>,
    locals: Vec<(WasmVar, WasmType)>,
}

impl Wasmable for WasmFunction {
    fn to_wat(&self) -> EcoString {
        let export = if self.def.exported { format!(" (export \"{}\")", self.def.info.name) } else { "".to_string() };
        let ret = format!("(result {})", self.def.return_type.to_wat());
        let args = self.args.iter().fold(EcoString::new(), |mut acc, x| {
            acc.push_str(&format!(" (param ${} {})", x.0.name, x.1.to_wat()));
            acc
        });
        let locals = self.locals.iter().fold(
            EcoString::new(),
            |mut acc, (v, type_)| {
                acc.push_str(&mut format!("\n    (local ${} {})", v.name, type_.to_wat()));
                acc
            },
        );
        let body = self.body.iter().map(|x| format!("\n    ({})", x.to_wat())).fold(EcoString::new(), |mut acc, x| {
            acc.push_str(&x);
            acc
        });

        format!("(func ${}{export}{args} {ret}{locals}{body})", self.def.info.name).into()
    }
}

#[derive(Debug)]
enum WasmInstruction {
    LocalGet(WasmVar),
    LocalSet(WasmVar),
    // Const(WasmType, WasmVar),
    Call { func: WasmVar, args: Vec<WasmInstruction> },
    Function(WasmFunction),
    I32Add(Vec<WasmInstruction>),
    I32Sub(Vec<WasmInstruction>),
    I32Const(i32),
    StructNew(WasmVar),
    StructGet(WasmVar, WasmVar), //Type Field
}

impl Wasmable for WasmInstruction {
    fn to_wat(&self) -> EcoString {
        match self {
            WasmInstruction::LocalGet(x) => { format!("local.get ${}", x.name).into() }
            WasmInstruction::LocalSet(x) => { format!("local.set ${}", x.name).into() }
            WasmInstruction::Call { func, args } => {
                format!("call ${}{}", func.name, args.to_wat()).into()
            }
            WasmInstruction::Function(x) => { x.to_wat() }
            WasmInstruction::I32Add(xs) => { format!("i32.add{}", xs.to_wat()).into() }
            WasmInstruction::I32Sub(xs) => { format!("i32.sub{}", xs.to_wat()).into() }
            WasmInstruction::I32Const(x) => { format!("i32.const {x}").into() }
            WasmInstruction::StructNew(x) => { format!("struct.new ${}", x.name).into() }
            WasmInstruction::StructGet(struc, field) => { format!("struct.get ${} ${}", struc.name, field.name).into() }
        }
    }
}

impl Wasmable for Vec<WasmInstruction> {
    fn to_wat(&self) -> EcoString {
        self.iter().map(|x| format!(" ({})", x.to_wat())).reduce(|mut acc, x| {
            acc.push_str(&x);
            acc
        }).unwrap_or(String::from("")).into()
    }
}

pub(crate) struct WasmThing {
    gleam_module: crate::ast::Module<ModuleInterface, Definition<Arc<Type>, TypedExpr, EcoString, EcoString>>,
    wasm_instructions: RefCell<Vec<WasmInstruction>>,
    type_section: RefCell<Vec<WasmTypeSectionEntry>>,
    functions_type_section_index: RefCell<HashMap<EcoString, (u32, u32)>>,
    // pub(crate) wasm_instructions: RefCell<Vec<ModuleField<'static>>>,
    // //AST
    // //Id is pretty private :( identifiers: HashMap<&'a str, Id<'a>>, // Symbol table, but not really, wanted to use for wasm names but unnecessary byte code. Will matter if we do in Gleam  "let x=1; ds(x);"
    // pub(crate) identifiers: HashMap<String, usize>,
    // //globals?
    // pub(crate) known_types: RefCell<HashMap<&'static str, ValType<'static>>>,
    // pub(crate) function_names: HashMap<&'static str, (&'static str, u32)>,
}

impl WasmThing {
    pub fn new(module: crate::ast::Module<ModuleInterface, Definition<Arc<Type>, TypedExpr, EcoString, EcoString>>) -> Self {
        WasmThing {
            gleam_module: module,
            wasm_instructions: RefCell::new(vec![]),
            type_section: RefCell::new(vec![]),
            functions_type_section_index: RefCell::new(Default::default()),
        }
    }
}

// pub(crate) fn known_types() -> RefCell<HashMap<&'static str, ValType<'static>>> {
//     let mut map = HashMap::new();
//     let _ = map.insert("Int", ValType::I32);
//     RefCell::new(map)
// }

impl WasmThing {
    // TODO remember wasm is stack based so arguments before functions :)


    // TODO give <'a> fn new(gleam_module: crate::ast::Module<crate::type_::ModuleInterface, crate::ast::Definition<Arc<crate::type_::Type>, crate::ast::TypedExpr, EcoString, EcoString>>) -> WasmThing<'a> {
    //     WasmThing {
    //         gleam_module,
    //         wasm_instructions: vec![],
    //         identifiers: Default::default(),
    //         known_types: known_types()
    //         // TODO prolly need types imported and a whole thing when getting some more
    //     }
    // }

    pub(crate) fn transform(&self) -> () {
        for gleam_definition in &self.gleam_module.definitions {
            self.transform_gleam_definition(gleam_definition);
        }
    }

    fn transform_gleam_definition(
        &self,
        gleam_expression: &Definition<Arc<Type>, TypedExpr, EcoString, EcoString>,
    ) {
        match gleam_expression {
            Definition::Function(gleam_function) => {
                self.add_gleam_function_to_wasm_instructions(gleam_function);
            }
            Definition::CustomType(gleam_custom_type) => {
                self.add_gleam_custom_type(gleam_custom_type);
            }
            _ => todo!()
        }
    }

    fn add_gleam_custom_type(&self, gleam_custom_type: &CustomType<Arc<Type>>) {
        let name = gleam_custom_type.name.clone();
        let mut struct_name = name.clone();
        struct_name.push_str("_struct");
        let len = self.type_section.borrow().len();
        let type_section_idx: usize = self.type_section.borrow().iter().enumerate()
            .filter_map(|(i, x)| {
                if let WasmTypeSectionEntry::PlaceHolder(huhname) = x {
                    if huhname == &struct_name {
                        return Some(i);
                    }
                }
                return None;
            }
            )
            .nth(0)
            .unwrap_or(len)
            ;

        let fields: Vec<_> = gleam_custom_type.constructors[0].arguments //TODO supports only one constructor :(
            .iter()
            .enumerate()
            .map(|(i, arg)|
                {
                    let name = arg.label.clone().unwrap_or(format!("{i}").into());
                    (WasmVar { name }, self.transform_gleam_type(arg.type_.as_ref()))
                }
            )
            .collect();

        let struct_def = WasmStructDef {
            info: WasmVar { name: struct_name },
            fields: fields.clone(),
        };

        if type_section_idx >= len {
            //TODO while! Maybe? Thiink more
            self.type_section.borrow_mut().push(WasmTypeSectionEntry::PlaceHolder("".into()));
        }
        self.type_section.borrow_mut()[type_section_idx] = WasmTypeSectionEntry::Struct(struct_def.clone());
        // dbg!(self.type_section.borrow());

        let constructor_name = name;
        // constructor_name.push_str("_constructor"); // Oh the type and constructor same name hmmmmm else we dont know we call really
        // TODO or push for func name but put in the hasmap differently
        let constructor_idx = type_section_idx + 1;
        let fun_len = self.functions_type_section_index.borrow().len();
        let _ = self.functions_type_section_index.borrow_mut().insert(constructor_name.clone(), (constructor_idx as u32, fun_len as u32)); //TODO get or insert? Maybe used already? Then need place holder in two places :P

        let var = WasmVar {
            name: constructor_name,
        };

        let constructor_def = WasmFuncDef {
            info: var.clone(),
            // params: fields.iter().map(|x| x.1.clone()).collect(),
            return_type: WasmType::ConcreteRef(struct_def.info.clone()),
            exported: false, //TODO pub structs?
        };

        self.type_section.borrow_mut().push(WasmTypeSectionEntry::Function(constructor_def.clone()));

        let mut instructions = Vec::new();
        for (v, _) in fields.iter() {
            instructions.push(WasmInstruction::LocalGet(v.clone()));
        }
        instructions.push(WasmInstruction::StructNew(struct_def.info));

        let wasm_constructor_instruction = WasmInstruction::Function(
            WasmFunction {
                args: fields.clone(),
                def: constructor_def,
                body: instructions,
                locals: vec![], //TODO check: No processing in constructor?
            }
        );

        self.wasm_instructions
            .borrow_mut()
            .push(wasm_constructor_instruction);
    }

    fn add_gleam_function_to_wasm_instructions(
        &self,
        gleam_function: &Function<Arc<Type>, TypedExpr>,
    ) {
        let name = gleam_function.name.clone();
        let len = self.type_section.borrow().len();
        let fun_len = self.functions_type_section_index.borrow().len();
        let loc: (u32, u32) = *self.functions_type_section_index.borrow_mut().get(&name).unwrap_or(&(len as u32, fun_len as u32));
        let _ = self.functions_type_section_index.borrow_mut().insert(name.clone(), loc);
        let wasm_var = WasmVar { name };

        let result_type = self.transform_gleam_type(gleam_function.return_type.as_ref());
        let mut arguments = Vec::new();
        let mut locals = Vec::new();
        let mut scope: HashMap<EcoString, usize> = HashMap::new();
        for param in &gleam_function.arguments {
            let name = param.names.get_variable_name().unwrap(); //TODO unwrap???
            let _ = scope.insert(name.clone(), scope.len());
            let type_ = self.transform_gleam_type(param.type_.as_ref());
            arguments.push((WasmVar { name: name.clone() }, type_));
        }

        let mut instructions = Vec::new();
        for gleam_statement in gleam_function.body.iter() {
            let (mut instrs, mut lcls) = self.transform_gleam_statement(gleam_statement, &mut scope);
            instructions.append(&mut instrs);
            locals.append(&mut lcls);
        }

        let func_def = WasmFuncDef {
            info: wasm_var,
            // params: arguments.iter().map(|x| x.1.clone()).collect(),
            return_type: result_type,
            exported: gleam_function.public,
        };

        if loc.0 >= len as u32 {
            //TODO while! Maybe? Thiink more
            self.type_section.borrow_mut().push(WasmTypeSectionEntry::PlaceHolder("".into()));
        }
        // dbg!(self.type_section.borrow());
        self.type_section.borrow_mut()[loc.0 as usize] = WasmTypeSectionEntry::Function(func_def.clone()); //TODO grow vec if necess
        // dbg!(self.type_section.borrow());

        let wasm_func = WasmInstruction::Function(
            WasmFunction {
                args: arguments,
                def: func_def,
                body: instructions,
                locals,
            }
        );
        self.wasm_instructions
            .borrow_mut()
            .push(wasm_func);
    }

    fn transform_gleam_statement(
        &self,
        gleam_statement: &Statement<Arc<Type>, TypedExpr>,
        scope: &mut HashMap<EcoString, usize>,
    ) -> (Vec<WasmInstruction>, Vec<(WasmVar, WasmType)>) {
        match gleam_statement {
            Statement::Expression(gleam_expression) => {
                self.transform_gleam_expression(gleam_expression, scope)
            }
            Statement::Assignment(gleam_assignment) => {
                self.transform_gleam_assignment(gleam_assignment, scope)
            }
            _ => todo!(),
        }
    }

    fn transform_gleam_assignment(&self, gleam_assignment: &Assignment<Arc<Type>, TypedExpr>, scope: &mut HashMap<EcoString, usize>) -> (Vec<WasmInstruction>, Vec<(WasmVar, WasmType)>) {
        match &gleam_assignment.pattern {
            Pattern::Variable { name, type_, .. } => {
                let _ = scope.insert(name.clone(), scope.len());
                let locals = vec![(
                    WasmVar { name: name.clone() }, self.transform_gleam_type(type_),
                )];
                let mut instrs = Vec::new();
                let mut val = self.transform_gleam_expression(gleam_assignment.value.as_ref(), scope);
                instrs.append(&mut val.0);
                instrs.push(WasmInstruction::LocalSet(locals[0].0.clone()));
                (instrs, locals)
            }
            _ => todo!()
        }
    }

    fn transform_gleam_expression(
        &self,
        gleam_expression: &TypedExpr,
        scope: &mut HashMap<EcoString, usize>,
    ) -> (Vec<WasmInstruction>, Vec<(WasmVar, WasmType)>) {
        let mut instructions = Vec::new();
        let mut locals = Vec::new();
        match gleam_expression {
            TypedExpr::BinOp {
                name, left, right, ..
            } => {
                let mut op_instrs = Vec::new();
                let mut ls = self.transform_gleam_expression(left.as_ref(), scope);
                op_instrs.append(&mut ls.0);
                locals.append(&mut ls.1);
                let mut rs = self.transform_gleam_expression(right.as_ref(), scope);
                op_instrs.append(&mut rs.0);
                let op = match name {
                    BinOp::AddInt => WasmInstruction::I32Add(op_instrs),
                    BinOp::SubInt => WasmInstruction::I32Sub(op_instrs),
                    _ => todo!()
                };
                instructions.push(op);
                locals.append(&mut rs.1);
            }
            TypedExpr::Var { name, .. } => {
                return (vec![WasmInstruction::LocalGet(WasmVar { name: name.clone() })], vec![]);
            }
            TypedExpr::Int { value, .. } => {
                //TODO type?
                return (vec![WasmInstruction::I32Const(value.parse().unwrap())], vec![]);
            }
            TypedExpr::Call { fun, args, .. } => {
                let mut instrs = Vec::with_capacity(args.len() + 1);
                let mut locals = Vec::new();
                for CallArg { value, .. } in args {
                    // TODO Or this after call?
                    // let mut new_scope = HashMap::new(); panics hehe, vars not in scope..
                    let (mut is, mut ls) = self.transform_gleam_expression(value, scope);
                    instrs.append(&mut is);
                    locals.append(&mut ls);
                }

                let fn_name = if let TypedExpr::Var { name, .. } = fun.as_ref() {
                    //TODO the start end is stupid, besides Var has more info that gets to fn name directly, also it's the loc of the call not the func
                    // self.start_end_names.get(&(location.start,location.end)).unwrap()
                    name
                } else {
                    dbg!(&fun);
                    todo!()
                };

                let call = WasmInstruction::Call {  //TODO tail call use instead? CallReturn :)
                    func: WasmVar {
                        name: fn_name.clone(),
                    },
                    args: instrs,
                };
                // instrs.push(call);
                return (vec![call], locals);
            }
            TypedExpr::RecordAccess { record, label, .. } => {
                let mut instrs = Vec::new();
                let record_name = match record.as_ref() {
                    TypedExpr::Var { name, .. } => {
                        name
                    }
                    _ => { todo!() }
                };
                instrs.push(WasmInstruction::LocalGet(WasmVar { name: record_name.clone() }));
                let mut record_type = record.type_().named_type_name().unwrap().1; //TODO this unwrap!
                record_type.push_str("_struct");
                let struct_var = self.type_section.borrow().iter().filter_map(|x|
                    {
                        match x {
                            WasmTypeSectionEntry::PlaceHolder(name) => {
                                if &record_type == name {
                                    Some(WasmVar {
                                        name: record_type.clone(),
                                    })
                                } else {
                                    None
                                }
                            }
                            WasmTypeSectionEntry::Struct(s) => {
                                if s.info.name == record_type {
                                    Some(s.info.clone())
                                } else {
                                    None
                                }
                            }
                            _ => { None }
                        }
                    }
                ).nth(0);

                let struct_var = match struct_var {
                    None => {
                        self.type_section.borrow_mut().push(WasmTypeSectionEntry::PlaceHolder(record_type.clone()));
                        // dbg!(self.type_section.borrow());
                        WasmVar {
                            name: record_type.clone(),
                        }
                    }
                    Some(x) => x
                };

                let field_var = WasmVar {
                    name: label.clone(),
                };

                instrs.push(WasmInstruction::StructGet(struct_var, field_var));

                return (instrs, locals);
            }
            x => {
                dbg!(x);
                todo!()
            }
        }
        (instructions, locals)
    }

    // fn transform_gleam_bin_op(&self, name: &BinOp) -> WasmInstruction {
    //     match name {
    //         BinOp::AddInt => WasmInstruction::I32Add,
    //         BinOp::SubInt => WasmInstruction::I32Sub,
    //         _ => todo!(),
    //     }
    // }

    fn transform_gleam_type(&self, type_: &Type) -> WasmType {
        match type_ {
            Type::Named { name, .. } =>
                match name.as_str() {
                    "Int" => WasmType::I32,
                    x => {
                        let mut x = x.to_string();
                        x.push_str("_struct");
                        let x = &x;

                        let len = self.type_section.borrow().len();
                        let idx = self.type_section.borrow().iter().enumerate()
                            .filter_map(|(i, entry)| {
                                if let WasmTypeSectionEntry::Struct(WasmStructDef { info, .. }) = entry {
                                    if info.name == x.as_str() {
                                        return Some(i);
                                    }
                                }
                                return None;
                            }
                            )
                            .nth(0).unwrap_or(len); //TODO map so easier, maybe :P

                        if idx == len {
                            self.type_section.borrow_mut().push(WasmTypeSectionEntry::PlaceHolder(EcoString::from(x.clone())));
                        }
                        // dbg!(self.type_section.borrow());

                        let x = EcoString::from(x.clone());

                        WasmType::ConcreteRef(
                            WasmVar {
                                name: x,
                            })
                    }
                }
            _ => todo!() //Prolly a ref, with correct index?,
        }
    }
}

impl Wasmable for WasmThing {
    fn to_wat(&self) -> EcoString {
        // not necess I guess... Maybe for structs.., TODO!
        // let types = self.type_section.borrow().iter().map(|x| x.to_wat())
        //     .reduce(|mut acc, x| {acc.push_str("\n");acc.push_str(&x); acc}).unwrap();

        let types = self.type_section.borrow().iter()
            .map(|x| x.to_wat())
            .reduce(|mut acc, x| {
                if x != "" {
                    acc.push_str("\n");
                    acc.push_str(&x);
                }
                acc
            }
            ).unwrap_or_default();

        let instructions = self.wasm_instructions.borrow().iter().map(|x| x.to_wat())
            .reduce(|mut acc, x| {
                acc.push_str("\n");
                acc.push_str(&x);
                acc
            }).unwrap_or_default();

        let mut module = EcoString::from("(module\n");
        module.push_str(&types);
        if types != "" {
            module.push_str("\n");
        }
        module.push_str(&instructions);
        module.push_str(")");
        module
    }
}

//TODO from i32.add in stack way to rhs & lhs is S-expr? Same with struct.new?
#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fs::File;
    use std::io::Write;
    use std::sync::Arc;
    use ecow::EcoString;
    use im::HashMap;
    use crate::analyse::TargetSupport;
    use crate::ast::{Definition, TypedExpr};
    use crate::type_::{ModuleInterface, Type};
    use crate::warning::{TypeWarningEmitter, WarningEmitter};
    use crate::wasm::{Wasmable, WasmThing};

    fn trying_to_make_module(
        program: &str,
    ) -> crate::ast::Module<ModuleInterface, Definition<Arc<Type>, TypedExpr, EcoString, EcoString>> {
        let parsed = crate::parse::parse_module(program).expect("syntax error");
        let module = parsed.module;
        // println!("{module:?}");
        let ids = crate::uid::UniqueIdGenerator::new();
        let small = ecow::EcoString::from("Welcome");
        let mut hs = HashMap::new();
        let hs2: std::collections::HashMap<EcoString, EcoString> = std::collections::HashMap::new();
        let we = TypeWarningEmitter::new(
            camino::Utf8PathBuf::new(),
            "".into(),
            WarningEmitter::new(Arc::new(crate::warning::VectorWarningEmitterIO::default())),
        );

        let _ = hs.insert(
            crate::type_::PRELUDE_MODULE_NAME.into(),
            crate::type_::build_prelude(&ids),
        );

        let module = crate::analyse::infer_module(
            crate::build::Target::JavaScript,
            &ids,
            module,
            crate::build::Origin::Src,
            &small,
            &hs,
            &we,
            &hs2,
            TargetSupport::NotEnforced,
        )
            .expect("type error?");
        module
    }

    #[test]
    fn wasm_2n() {
        let gleam_module = trying_to_make_module(
            "pub fn add(x: Int, y: Int) -> Int {
            x + y
          }",
        );

        let w = WasmThing {
            gleam_module,
            wasm_instructions: RefCell::new(vec![]),
            type_section: RefCell::new(vec![]),
            functions_type_section_index: RefCell::new(Default::default()),
        };
        w.transform();

        let wat = w.to_wat();
        let mut file = File::create("letstry.wat").unwrap();
        let _ = file.write_all(wat.as_bytes());

        let wasm = wat::parse_str(wat.clone()).unwrap();

        let mut file = File::create("letstry.wasm").unwrap();
        let _ = file.write_all(&wasm);
        insta::assert_snapshot!(wat);
    }

    #[test]
    fn wasm_3nd() {
        let gleam_module = trying_to_make_module(
            "pub fn add(x: Int, y: Int) -> Int {
            let z = 10
            let a = 100
            x+ y + z + a
          }",
        );

        let w = WasmThing {
            gleam_module,
            wasm_instructions: RefCell::new(vec![]),
            type_section: RefCell::new(vec![]),
            functions_type_section_index: RefCell::new(Default::default()),
        };
        w.transform();

        let wat = w.to_wat();
        let mut file = File::create("letstry.wat").unwrap();
        let _ = file.write_all(wat.as_bytes());

        let wasm = wat::parse_str(wat.clone()).unwrap();

        let mut file = File::create("letstry.wasm").unwrap();
        let _ = file.write_all(&wasm);

        insta::assert_snapshot!(wat);
    }

    #[test]
    fn wasm_4nd() {
        let gleam_module = trying_to_make_module(
            "
        pub fn add(x: Int, y: Int) -> Int {
            internal_add(x+1,y)
          }
        fn internal_add(x: Int, y: Int) -> Int {
            x + y
        }
          ",
        );

        //TODO not the nicest wat with the x+1, still legal tho....

        let w = WasmThing {
            gleam_module,
            wasm_instructions: RefCell::new(vec![]),
            type_section: RefCell::new(vec![]),
            functions_type_section_index: RefCell::new(Default::default()),
        };
        w.transform();

        let wat = w.to_wat();
        let mut file = File::create("letstry.wat").unwrap();
        let _ = file.write_all(wat.as_bytes());

        let wasm = wat::parse_str(wat.clone()).unwrap();

        let mut file = File::create("letstry.wasm").unwrap();
        let _ = file.write_all(&wasm);

        insta::assert_snapshot!(wat);
    }

    #[test]
    fn wasm_5nd() {
//TODO pub types!
        let gleam_module = trying_to_make_module(
            "
         type Cat {
  Cat(name: Int, cuteness: Int)
}
        pub fn add(x: Int, y: Int) -> Int {
            let cat1 = Cat(name: x, cuteness: y)
            cat1.cuteness + cat1.name
          }",
        );

        //TODO oh no also not deterministic! WTFFFFFFFFFF! Is it the gleam module? YEs
        // work back from:
        // 29    29 │ 0x7F
        // 30    30 │ 0x03
        // 31    31 │ 0x03
        // 32    32 │ 0x02
        // 33 │+0x02
        // 33    34 │ 0x01
        // 34       │-0x02
        // 35    35 │ 0x07
        // 36    36 │ 0x07
        // 37    37 │ 0x01
        // 38    38 │ 0x03

        // So sort the exports lol, then makes sense if it changes what function is reffered to with a function index lol....


        let w = WasmThing {
            gleam_module,
            wasm_instructions: RefCell::new(vec![]),
            type_section: RefCell::new(vec![]),
            functions_type_section_index: RefCell::new(Default::default()),
        };
        w.transform();

        let wat = w.to_wat();
        let mut file = File::create("letstry.wat").unwrap();
        let _ = file.write_all(wat.as_bytes());

        let wasm = wat::parse_str(wat.clone()).unwrap();

        let mut file = File::create("letstry.wasm").unwrap();
        let _ = file.write_all(&wasm);

        insta::assert_snapshot!(wat);
//
//
//     //TODO: Uncaught (in promise) CompileError: wasm validation error: at offset 43: type mismatch: expression has type i64 but expected structref
//     // but we do get bytes.... not promising! Since encode doesn't catch it....
//     // Ok new error: CompileError: wasm validation error: at offset 46: not a struct type
//     // Yeah cause function does it's own magic (on wasm tools side) to add to the types at top of module, but struct not so much...
//     // Ok now: CompileError: wasm validation error: at offset 68: popping value from empty stack
//     // Lol was using firefox, maybe no GC? Chrome has better errors: WebAssembly.instantiateStreaming(): Compiling function #1 failed: not enough arguments on the stack for struct.get (need 1, got 0) @+68
// // Can also do: wasm2wat -v --enable-gc compiler-core/letstry.wasm
//     // now error (Chrome) is: Compiling function #1 failed: struct.get[0] expected type (ref null 0), found local.get of type structref @+68
// //Ok sure firefox supports it too
//
//     // wasm2wat still thinks it's wrong even with --enable-all: 0000017: error: expected valid result type (got -0x1c)
//     // And the problem is the "0x64 0x00" return type of the constructor... parsed as -0x1c, checked by changing that 0x64 byte, will change te -0x1c error msg, wild!
//     // ex: change to 0x63 will say: "000001f: error: expected valid result type (got -0x1d)"
//     // browsers still parse em...
//     // And spec says it's allowed in return type?
//     // check issues: maybe https://github.com/WebAssembly/wabt/issues/2364 (see also: https://github.com/WebAssembly/wabt/pull/2363)? Lol is ref encoded as 0x6b instead of 0x64, that's be nice haha
//     // or https://github.com/WebAssembly/wabt/issues/2333 weird! Has the enable flag but no support?
//     // also --enable-gc does allow it to process the 0x5f struct type flag..
//     // prolly problem here: https://github.com/WebAssembly/wabt/blob/main/include/wabt/type.h#L47 Oh wel..
//     // Eh no compiled locally with line 47 changed the problem is bigger. Also if I change to 6b in file won't fix with original.
//     // Aaah enable more features: /home/harm/git/wabt/build/wasm2wat --enable-all -v /home/harm/git/gleam/compiler-core/letstry.wasm
//     // new error: 0000056: error: unexpected opcode: 0xfb
//     // Ah crap that's struct.new, well if it's not supported it really is not supported....
//
//     //TODO so the concrete types are fine now, but would like abstract struct when returning an enum variant, then you'd need structref (well non-nullable right, I mean...)
//
//     // dbg!(&gleam_module);
//     // assert!(false);
//     //TODO what the cat type looks like is in module.types
//
//     let w = WasmThing {
//         gleam_module,
//         wasm_instructions: RefCell::new(vec![]),
//         identifiers: Default::default(),
//         known_types: known_types(), // TODO prolly need types imported and a whole thing when getting some more
//         function_names: HashMap::new(),
//     };
//     let res = w.transform().unwrap();
//     let mut file = File::create("letstry.wasm").unwrap();
//
//     let _ = file.write_all(&res);
//     // assert!(false);
    }

    #[test]
    fn wasm_6nd() {
//TODO pub types!
        let gleam_module = trying_to_make_module(
            "
         type Cat {
  Cat(name: Int, cuteness: Int)
}

type Kitten {Kitten(name: Int, age: Int, cuteness: Int) }

        fn add_cat(cat: Cat) -> Int {
    cat.cuteness + cat.name
}

    fn grow(kitten: Kitten) -> Cat {
    Cat(name: kitten.name, cuteness: kitten.cuteness-1)
}

        pub fn add(x: Int, y: Int) -> Int {
            let kitten = Kitten(name: x, cuteness: y, age: 12)
            let cat = grow(kitten)
            add_cat(cat)
          }",
        );


        let w = WasmThing {
            gleam_module,
            wasm_instructions: RefCell::new(vec![]),
            type_section: RefCell::new(vec![]),
            functions_type_section_index: RefCell::new(Default::default()),
        };
        w.transform();

        let wat = w.to_wat();
        let mut file = File::create("letstry.wat").unwrap();
        let _ = file.write_all(wat.as_bytes());

        let wasm = wat::parse_str(wat.clone()).unwrap();

        let mut file = File::create("letstry.wasm").unwrap();
        let _ = file.write_all(&wasm);

        insta::assert_snapshot!(wat);
    }
}