use core::hash::Hash;

use std::{
    borrow::Borrow, cell::RefCell, cmp::Ordering, collections::HashMap, option::Option, sync::Arc,
};

use ecow::EcoString;

use crate::{
    ast::{Pattern, Publicity, RecordConstructor, TypedClause, TypedExpr},
    type_::{Deprecation, Type, TypeVar, ValueConstructor, ValueConstructorVariant},
};

use DecisionTree::*;

pub(crate) struct DecisionTreeGenerator<'a> {
    subject_values: &'a [TypedExpr],
    clauses: &'a [TypedClause],
    variant_count: HashMap<(EcoString, EcoString), Vec<RecordConstructor<Arc<Type>>>>,
}

impl<'a> DecisionTreeGenerator<'a> {
    pub(crate) fn new(
        subject_values: &'a [TypedExpr],
        clauses: &'a [TypedClause],
        variant_count: HashMap<(EcoString, EcoString), Vec<RecordConstructor<Arc<Type>>>>,
    ) -> Self {
        DecisionTreeGenerator {
            subject_values,
            clauses,
            variant_count,
        }
    }

    pub(crate) fn to_tree(&self) -> DecisionTree {
        // Build the matrix!
        let hs = self.subject_values.to_vec();
        let mut actions_and_env = Vec::new();
        let patterns = self.clauses.iter().fold(Vec::new(), |mut acc, c| {
            let action_and_env = (c.then.clone(), HashMap::new());
            acc.push(c.pattern.clone());
            actions_and_env.push(action_and_env.clone());
            for p in &c.alternative_patterns {
                acc.push(p.clone());
                actions_and_env.push(action_and_env.clone());
            }
            acc
        });
        //Patterns and actions seem to be correctly matched here.
        // dbg!(&actions_and_env[0]);
        // dbg!(&actions_and_env[1]);
        // dbg!(&patterns[0]);

        // let actions_and_env = self
        //     .clauses
        //     .iter()
        //     .map(|c| (c.then.clone(), HashMap::new()))
        //     .collect();
        let matrix = PatternMatrix {
            hs,
            patterns,
            actions_and_env,
        };
        compile_tree(matrix, self.variant_count.clone())
    }
}

fn compile_tree(
    mut matrix: PatternMatrix,
    variant_count: HashMap<(EcoString, EcoString), Vec<RecordConstructor<Arc<Type>>>>,
) -> DecisionTree {
    // P is empty
    if matrix.patterns.first().is_none() {
        return Unreachable;
    }

    // println!("compiling tree!");
    let p_len = matrix.patterns[0].len();
    assert!(matrix.patterns.iter().all(|ps| ps.len() == p_len));

    //Match always succeeds, is_empty uneccesary but clear

    //Two below seem to be going wrong.
    // dbg!(&matrix.patterns[0]);
    // dbg!(&matrix.actions_and_env.len());
    if matrix.patterns[0].is_empty()
        || matrix.patterns[0].iter().all(|p| match p {
            Pattern::Discard { .. } | Pattern::Variable { .. } => true, //TODO well I mean...
            _ => false,
        })
    {
        let (branch, mut bindings) = matrix.actions_and_env[0].clone();

        matrix.patterns[0].iter().enumerate().for_each(|(i, p)| {
            match p {
                Pattern::Variable { name, .. } => {
                    if !bindings.contains_key(name) {
                        //TODO wellllllllllll Hmmmm Maybe just for list.... Else scop issues?
                        let _ = bindings.insert(name.clone(), Binding::Expr(matrix.hs[i].clone()));
                    }
                    // Ok kinda fun but! Should be the tail etc from before! So do get the tags here before somehow that should have the right logic right except if started as list since then newer overwrites, fuck!
                    // also seems like we get the wrong clause weird!
                    // .. But never here
                }
                _ => (),
            }
        });

        return Success { branch, bindings };
    }

    // "If there exists a variable pattern p-l_i and the previous rows contain no var patterns add it to the env.
    // TODO wait don't I do this above already?
    for (idx, row) in matrix.patterns.iter().enumerate() {
        for (i, p) in row.iter().enumerate() {
            match p {
                Pattern::Variable { name, .. } => {
                    // let hs_len = matrix.hs.len();
                    // println!("row: {idx}, pattern: {i}, subject count: {hs_len}");
                    // let _ = matrix.actions_and_env[idx]
                    //     .1
                    //     .insert(name.clone(), matrix.hs[i].clone());
                    ()
                }
                _ => (),
            }
        }
    }

    // OR patterns already removed

    // Build an actual switch node.
    //1 Pick column
    let i = pick_column(&matrix, variant_count.clone());

    //2 Identify necessary branches
    let type_ = matrix.hs[i].type_();
    let mut tags = Vec::new();
    let type_: &Type = type_.borrow();
    //TODO check not same case multiple times!
    get_tags(type_, &mut matrix, i, &mut tags, &variant_count);
    // dbg!(&tags);

    //3 Compile the decision sub-trees corresponding to each branch
    let mut cases = Vec::new();
    for tag in &tags {
        //TODO trying this so ignore earlier stuff, yeah
        // let mut hs = Vec::new();
        // let  _ = &matrix.hs[tag_idx..].clone_into(&mut hs);

        // let matrix = PatternMatrix {
        //     hs,
        //     patterns,
        //     actions_and_env,
        // };

        let branch = Box::new(compile_branch(
            i,
            tag,
            matrix.clone(),
            variant_count.clone(),
        ));
        let case = match tag {
            Tag::Constructor(ref c) => Case::ConstructorEquality {
                constructor: c.clone(),
            },
            Tag::T => Case::Default,
            Tag::List {
                head_element: None,
                tail: None,
            } => Case::EmptyList,
            Tag::List {
                head_element: _,
                tail: _,
            } => Case::List,
            x => {
                println!("{x:?}");
                todo!()
            }
        };
        cases.push((case.clone(), branch));
        if let Case::Default = case {
            break;
        }
    }

    //4 Assemble into single tree of switch nodes
    Switch {
        discriminant: matrix.hs[i].clone(),
        cases,
    }
}

fn get_tags(
    type_: &Type,
    matrix: &mut PatternMatrix,
    i: usize,
    tags: &mut Vec<Tag>,
    variant_count: &HashMap<(EcoString, EcoString), Vec<RecordConstructor<Arc<Type>>>>,
) {
    match type_ {
        Type::Named { module, name, .. } => {
            let len = matrix.patterns.len();
            for row_idx in 0..len {
                match &matrix.patterns[row_idx][i] {
                    Pattern::Constructor { name, .. } => {
                        let _ = tags.push(Tag::Constructor(name.clone()));
                    }
                    Pattern::Variable { name, .. } => {
                        let val = matrix.hs[i].clone();
                        // dbg!((name.clone(), Binding::Expr(val.clone())));
                        // let _ = matrix.actions_and_env[row_idx].1.insert(name.clone(), Binding::Expr(val));
                        // dbg!(&matrix.actions_and_env[row_idx].1);
                        let _ = tags.push(Tag::T);
                    }
                    Pattern::List {
                        location,
                        elements,
                        tail,
                        type_,
                    } => {
                        // if elements.len() != 1 {
                        // println!("\n{elements:?}\n{tail:?}\n");

                        //     todo!();
                        // }
                        if elements.len() == 1 {
                            let _ = tags.push(Tag::List {
                                head_element: Some(elements[0].clone()),
                                tail: tail.clone(),
                            });
                        } else if elements.len() == 0 && tail.is_none() {
                            //Empty list.
                            let _ = tags.push(Tag::List {
                                head_element: None,
                                tail: tail.clone(),
                            });
                        } else {
                            todo!()
                        }
                    }
                    Pattern::Discard {
                        name,
                        location,
                        type_,
                    } => {
                        let _ = tags.push(Tag::T);
                    }
                    x => {
                        println!("{x:?}");
                        panic!()
                    } //unreachable...
                }
            }
            // println!("Guh.... module: {module:?}, name:{name:?}"); // gleam, list???
            if module.as_str() != "gleam" {
                if (*variant_count.get(&(module.clone(), name.clone())).unwrap()).len() == 0 {
                    todo!(); //Constant type? Like maybe Int has zero constructors? Then add a c case. Like in table 1.
                }
                if tags.len()
                    != (*variant_count.get(&(module.clone(), name.clone())).unwrap()).len()
                {
                    let _ = tags.push(Tag::T);
                }
            } else {
                // TODO
                // List default case!
            }
        }
        Type::Var { type_ } => {
            let type_: &RefCell<TypeVar> = type_.borrow();
            let type_: &TypeVar = &*type_.borrow();

            match type_ {
                TypeVar::Link { type_ } => {
                    let type_ = type_.borrow();
                    get_tags(type_, matrix, i, tags, variant_count);
                }
                _ => {
                    let _ = tags.push(Tag::T);
                } //Hmm unbound type vars...
            }
        }
        Type::Tuple { .. } => {
            let _ = tags.push(Tag::T);
        }
        x => {
            println!("{x:?}");
            todo!()
        } //unreachable ... Nope explicit panic lol
    };
}

fn compile_branch(
    i: usize,
    tag: &Tag,
    matrix: PatternMatrix,
    variant_count: HashMap<(EcoString, EcoString), Vec<RecordConstructor<Arc<Type>>>>,
) -> DecisionTree {
    let mut new_matrix = PatternMatrix {
        hs: Vec::new(),
        patterns: Vec::new(),
        actions_and_env: Vec::new(),
    };
    // H output expand
    // dbg!(&tag);
    for (j, h) in matrix.hs.iter().enumerate() {
        if i == j {
            // println!("h:\n {h:?}\nt: {tag:?}\n");
            // dbg!(h);
            match h {
                TypedExpr::Var { constructor, .. } => {
                    match tag.clone() {
                        Tag::Constructor(c_name) => {
                            //No field map? Nope variant is local variable.
                            let c_t: &Type = constructor.type_.borrow();
                            // println!("constructor: {constructor:?}\nconstructor type {c_t:?}\n");
                            // if any in column are constructor pattern at i! else make it empty! But hs alrady too short wtf
                            // patterns is too long........ TODO! Was adding too many lol
                            match c_t {
                                Type::Var { type_ } => {
                                    let type_: &RefCell<TypeVar> = type_.borrow();
                                    let type_: &TypeVar = &*type_.borrow();
                                    match type_ {
                                        TypeVar::Link { type_ } => match type_.borrow() {
                                            Type::Named {
                                                publicity,
                                                package,
                                                module,
                                                name,
                                                args,
                                            } => {
                                                let constructors = variant_count
                                                    .get(&(module.clone(), name.clone()))
                                                    .unwrap();
                                                // dbg!(&module);
                                                // dbg!(&name);
                                                // dbg!(&c_name);
                                                let constructor = constructors
                                                    .iter()
                                                    .find(|c| c.name.as_str() == c_name.as_str())
                                                    .unwrap();
                                                // dbg!(&constructor.arguments.len());
                                                // dbg!(&constructor);
                                                for (index, arg) in
                                                    constructor.arguments.iter().enumerate()
                                                {
                                                    let label = match &arg.label {
                                                        Some((l, _)) => l.clone(),
                                                        None => EcoString::from(format!("{index}")),
                                                    };
                                                    // dbg!(&label);
                                                    new_matrix.hs.push(TypedExpr::RecordAccess {
                                                        location: h.location(),
                                                        typ: arg.type_.clone(),
                                                        label,
                                                        index: index as u64,
                                                        record: Box::new(h.clone()),
                                                    });
                                                }
                                            }
                                            _ => todo!(),
                                        },
                                        _ => todo!(),
                                    }
                                }
                                _ => todo!(),
                            }
                        }
                        Tag::List { head_element, tail } => {
                            // dbg!(&tag);
                            // println!("{tag:?}");
                            // dbg!(&head_element);
                            // dbg!(tail);
                            match (head_element, tail) {
                                (None, None) => {
                                    //Empty list.
                                    // one columns jus head?
                                    // println!("elem no tail! {head_element:?}");
                                    continue;
                                }
                                (Some(_), None) => {
                                    // println!("elem! {head_element:?}");
                                    // two columns, head and tail of subject value at i old
                                    match &matrix.hs[i] {
                                        TypedExpr::List { elements, .. } => {
                                            println!("This ok then?");
                                            new_matrix.hs.push(elements[0].clone());
                                        }
                                        _ => panic!(), //well now
                                    }
                                }
                                (Some(p1), Some(p2)) => {
                                    let p2 = p2.as_ref();
                                    match (p1, p2) {
                                        (
                                            Pattern::Variable {
                                                location: loc1,
                                                name: name1,
                                                type_: type1,
                                            },
                                            Pattern::Variable {
                                                location: loc2,
                                                name: name2,
                                                type_: type2,
                                            },
                                        ) => {
                                            new_matrix.hs.push(TypedExpr::Var {
                                                location: loc1.clone(),
                                                constructor: ValueConstructor {
                                                    publicity: Publicity::Internal,
                                                    deprecation: Deprecation::NotDeprecated,
                                                    variant:
                                                        ValueConstructorVariant::LocalVariable {
                                                            location: loc1.clone(),
                                                        },
                                                    type_: type1.clone(),
                                                },
                                                name: name1.clone(),
                                            });
                                            new_matrix.hs.push(TypedExpr::Var {
                                                location: loc2.clone(),
                                                constructor: ValueConstructor {
                                                    publicity: Publicity::Internal,
                                                    deprecation: Deprecation::NotDeprecated,
                                                    variant:
                                                        ValueConstructorVariant::LocalVariable {
                                                            location: loc2.clone(),
                                                        },
                                                    type_: type2.clone(),
                                                },
                                                name: name2.clone(),
                                            });
                                        }
                                        (_, _) => panic!(),
                                    }
                                }
                                _ => panic!(),
                            }
                        }
                        Tag::T => {
                            //() //Do nothing? TODO could be wrong! Bu I think right
                            // No h for this column.
                            // dbg!("T hs");
                            continue;
                        }
                        t => {
                            println!("{t:?}");
                            todo!()
                        }
                    }
                }
                TypedExpr::Tuple { elems, .. } => {
                    for elem in elems {
                        new_matrix.hs.push(elem.clone());
                    }
                }
                TypedExpr::RecordAccess {
                    location,
                    typ,
                    label,
                    index,
                    record,
                } => {
                    match tag.clone() {
                        Tag::Constructor(c_name) => {
                            //No field map? Nope variant is local variable.
                            let c_t: &Type = typ.borrow();
                            // println!("constructor: {constructor:?}\nconstructor type {c_t:?}\n");
                            // if any in column are constructor pattern at i! else make it empty! But hs alrady too short wtf
                            // patterns is too long........ TODO! Was adding too many lol
                            match c_t {
                                Type::Var { type_ } => {
                                    let type_: &RefCell<TypeVar> = type_.borrow();
                                    let type_: &TypeVar = &*type_.borrow();
                                    match type_ {
                                        TypeVar::Link { type_ } => match type_.borrow() {
                                            Type::Named {
                                                publicity,
                                                package,
                                                module,
                                                name,
                                                args,
                                            } => {
                                                let constructors = variant_count
                                                    .get(&(module.clone(), name.clone()))
                                                    .unwrap();
                                                dbg!(&module);
                                                dbg!(&name);
                                                dbg!(&c_name);
                                                let constructor = constructors
                                                    .iter()
                                                    .find(|c| c.name.as_str() == c_name.as_str())
                                                    .unwrap();
                                                // dbg!(&constructor.arguments.len());
                                                // dbg!(&constructor);
                                                for (index, arg) in
                                                    constructor.arguments.iter().enumerate()
                                                {
                                                    let label = match &arg.label {
                                                        Some((l, _)) => l.clone(),
                                                        None => EcoString::from(format!("{index}")),
                                                    };
                                                    // dbg!(&label);
                                                    new_matrix.hs.push(TypedExpr::RecordAccess {
                                                        location: h.location(),
                                                        typ: arg.type_.clone(),
                                                        label,
                                                        index: index as u64,
                                                        record: Box::new(h.clone()),
                                                    });
                                                }
                                            }
                                            _ => todo!(),
                                        },
                                        _ => todo!(),
                                    }
                                }
                                Type::Named {
                                    publicity,
                                    package,
                                    module,
                                    name,
                                    args,
                                } => {
                                    //TODO waaaaay too duplicated from above!
                                    let constructors =
                                        variant_count.get(&(module.clone(), name.clone())).unwrap();
                                    // dbg!(&module);
                                    // dbg!(&name);
                                    // dbg!(&c_name);
                                    let constructor = constructors
                                        .iter()
                                        .find(|c| c.name.as_str() == c_name.as_str())
                                        .unwrap();
                                    // dbg!(&constructor.arguments.len());
                                    // dbg!(&constructor);
                                    for (index, arg) in constructor.arguments.iter().enumerate() {
                                        let label = match &arg.label {
                                            Some((l, _)) => l.clone(),
                                            None => EcoString::from(format!("{index}")),
                                        };
                                        // dbg!(&label);
                                        new_matrix.hs.push(TypedExpr::RecordAccess {
                                            location: h.location(),
                                            typ: arg.type_.clone(),
                                            label,
                                            index: index as u64,
                                            record: Box::new(h.clone()),
                                        });
                                    }
                                }
                                x => {
                                    dbg!(x);
                                    todo!()
                                }
                            }
                        }
                        Tag::List { head_element, tail } => {
                            // dbg!(&tag);
                            // println!("{tag:?}");
                            // dbg!(&head_element);
                            // dbg!(tail);
                            match (head_element, tail) {
                                (None, None) => {
                                    //Empty list.
                                    // one columns jus head?
                                    // println!("elem no tail! {head_element:?}");
                                    continue;
                                }
                                (Some(_), None) => {
                                    // println!("elem! {head_element:?}");
                                    // two columns, head and tail of subject value at i old
                                    match &matrix.hs[i] {
                                        TypedExpr::List { elements, .. } => {
                                            println!("This ok then?");
                                            new_matrix.hs.push(elements[0].clone());
                                        }
                                        _ => panic!(), //well now
                                    }
                                }
                                (Some(p1), Some(p2)) => {
                                    let p2 = p2.as_ref();
                                    match (p1, p2) {
                                        (
                                            Pattern::Variable {
                                                location: loc1,
                                                name: name1,
                                                type_: type1,
                                            },
                                            Pattern::Variable {
                                                location: loc2,
                                                name: name2,
                                                type_: type2,
                                            },
                                        ) => {
                                            new_matrix.hs.push(TypedExpr::Var {
                                                location: loc1.clone(),
                                                constructor: ValueConstructor {
                                                    publicity: Publicity::Internal,
                                                    deprecation: Deprecation::NotDeprecated,
                                                    variant:
                                                        ValueConstructorVariant::LocalVariable {
                                                            location: loc1.clone(),
                                                        },
                                                    type_: type1.clone(),
                                                },
                                                name: name1.clone(),
                                            });
                                            new_matrix.hs.push(TypedExpr::Var {
                                                location: loc2.clone(),
                                                constructor: ValueConstructor {
                                                    publicity: Publicity::Internal,
                                                    deprecation: Deprecation::NotDeprecated,
                                                    variant:
                                                        ValueConstructorVariant::LocalVariable {
                                                            location: loc2.clone(),
                                                        },
                                                    type_: type2.clone(),
                                                },
                                                name: name2.clone(),
                                            });
                                        }
                                        (_, _) => panic!(),
                                    }
                                }
                                _ => panic!(),
                            }
                        }
                        Tag::T => {
                            //() //Do nothing? TODO could be wrong! Bu I think right
                            // No h for this column.
                            // dbg!("T hs");
                            continue;
                        }
                        t => {
                            println!("{t:?}");
                            todo!()
                        }
                    }
                }
                x => {
                    dbg!(x);
                    todo!()
                }
            }
        } else {
            new_matrix.hs.push(h.clone())
        }
    }

    //f output for expand, could be one loop with above but eh.
    'row: for (row_idx, row) in matrix.patterns.iter().enumerate() {
        let mut new_actions_and_env = matrix.actions_and_env[row_idx].clone();
        // dbg!(&new_actions_and_env.0);
        let mut new_row = Vec::new();
        'pattern: for j in 0..row.len() {
            if i == j {
                //TODO env!?
                // let p = &matrix.patterns[row_idx][j];
                // dbg!(p);
                // println!("p:\n {p:?}\n");
                match &matrix.patterns[row_idx][j] {
                    // Pattern::Int { location, value } => todo!(),
                    // Pattern::Float { location, value } => todo!(),
                    // Pattern::String { location, value } => todo!(),
                    //TODO variable and discard should be same.....
                    Pattern::Variable {
                        location,
                        name,
                        type_,
                    } => {
                        let _ = new_actions_and_env.1.insert(name.clone(), Binding::Expr((&matrix.hs[j]).clone()));
                        match &matrix.hs[j] {
                            TypedExpr::Var { constructor, .. } => match tag {
                                Tag::Constructor(c_name) => {
                                    let (module, name) = match constructor.type_.borrow() {
                                        Type::Named { name, module, .. } => {
                                            (module.clone(), name.clone())
                                        }
                                        Type::Var { type_ } => {
                                            let type_: &RefCell<TypeVar> = type_.borrow();
                                            let type_: &TypeVar = &*type_.borrow();
                                            match type_ {
                                                TypeVar::Unbound { id } => todo!(),
                                                TypeVar::Link { type_ } => match type_.borrow() {
                                                    Type::Named {
                                                        publicity,
                                                        package,
                                                        module,
                                                        name,
                                                        args,
                                                    } => (module.clone(), name.clone()),
                                                    Type::Fn { args, retrn } => todo!(),
                                                    Type::Var { type_ } => todo!(),
                                                    Type::Tuple { elems } => todo!(),
                                                },
                                                TypeVar::Generic { id } => todo!(),
                                            }
                                        }
                                        _ => {
                                            // dbg!(oops);
                                            todo!()
                                        }
                                    };

                                    let constructors =
                                        variant_count.get(&(module.clone(), name.clone())).unwrap();

                                    let constructor = constructors
                                        .iter()
                                        .find(|c| c.name.as_str() == c_name.as_str())
                                        .unwrap();
                                    for arg in &constructor.arguments {
                                        new_row.push(Pattern::Discard {
                                            name: EcoString::new(),
                                            location: location.clone(),
                                            type_: arg.type_.clone(),
                                        });
                                    }
                                }
                                Tag::T => {
                                    // No new pattern!
                                    //But what about the corresponding h? An hs per row?
                                    continue 'pattern;
                                    // Just nothing? Sure it's the default
                                }
                                _ => todo!(),
                            },
                            TypedExpr::RecordAccess {
                                location,
                                typ,
                                label,
                                index,
                                record,
                            } => {
                                match tag.clone() {
                                    Tag::Constructor(c_name) => {
                                        let (module, name) = match typ.borrow() {
                                            Type::Named { name, module, .. } => {
                                                (module.clone(), name.clone())
                                            }
                                            Type::Var { type_ } => {
                                                let type_: &RefCell<TypeVar> = type_.borrow();
                                                let type_: &TypeVar = &*type_.borrow();
                                                match type_ {
                                                    TypeVar::Unbound { id } => todo!(),
                                                    TypeVar::Link { type_ } => match type_.borrow() {
                                                        Type::Named {
                                                            publicity,
                                                            package,
                                                            module,
                                                            name,
                                                            args,
                                                        } => (module.clone(), name.clone()),
                                                        Type::Fn { args, retrn } => todo!(),
                                                        Type::Var { type_ } => todo!(),
                                                        Type::Tuple { elems } => todo!(),
                                                    },
                                                    TypeVar::Generic { id } => todo!(),
                                                }
                                            }
                                            _ => {
                                                // dbg!(oops);
                                                todo!()
                                            }
                                        };
    
                                        let constructors =
                                            variant_count.get(&(module.clone(), name.clone())).unwrap();
    
                                        let constructor = constructors
                                            .iter()
                                            .find(|c| c.name.as_str() == c_name.as_str())
                                            .unwrap();
                                        for arg in &constructor.arguments {
                                            new_row.push(Pattern::Discard {
                                                name: EcoString::new(),
                                                location: location.clone(),
                                                type_: arg.type_.clone(),
                                            });
                                        }

                                    }
                                    Tag::T => continue 'pattern,
                                    _ => todo!()
                                }
                                // new_row.push(Pattern::Discard {
                                //     name: EcoString::new(),
                                //     location: location.clone(),
                                //     type_: typ.clone(),
                                // });
                                // //TODO  How many depends on type, else put above back
                                // dbg!(tag);
                                // todo!()
                            }
                            e => {
                                dbg!(e);
                                todo!()
                            }
                        }
                    }
                    // Pattern::VarUsage { location, name, constructor, type_ } => todo!(),
                    // Pattern::Assign { name, location, pattern } => todo!(),
                    Pattern::Discard {
                        name,
                        location,
                        type_,
                    } => {
                        match &matrix.hs[j] {
                            TypedExpr::Var { constructor, .. } => match tag {
                                Tag::Constructor(c_name) => {
                                    let (module, name) = match constructor.type_.borrow() {
                                        Type::Named { name, module, .. } => (name, module),
                                        _ => todo!(),
                                    };
                                    let constructors =
                                        variant_count.get(&(module.clone(), name.clone())).unwrap();

                                    let constructor = constructors
                                        .iter()
                                        .find(|c| c.name.as_str() == c_name.as_str())
                                        .unwrap();
                                    for arg in &constructor.arguments {
                                        new_row.push(Pattern::Discard {
                                            name: EcoString::new(),
                                            location: location.clone(),
                                            type_: arg.type_.clone(),
                                        });
                                    }
                                }
                                Tag::T => {
                                    continue 'pattern;
                                }
                                _ => {
                                    todo!();
                                }
                            },
                            TypedExpr::Tuple { elems, .. } => {
                                for _elem in elems {
                                    new_row.push(Pattern::Discard {
                                        name: name.clone(),
                                        location: location.clone(),
                                        type_: type_.clone(),
                                    }); //TODO name & types get them from elem!
                                }
                            }
                            TypedExpr::RecordAccess {
                                location,
                                typ,
                                label,
                                index,
                                record,
                            } => {
                                // dbg!("me?");
                                //TODO I guess?
                                // new_row.push(Pattern::Discard {
                                //     name: EcoString::new(),
                                //     location: location.clone(),
                                //     type_: typ.clone(),
                                // });
                                match tag.clone() {
                                    Tag::Constructor(c_name) => {
                                        let (module, name) = match typ.borrow() {
                                            Type::Named {
                                                publicity,
                                                package,
                                                module,
                                                name,
                                                args,
                                            } => (module.clone(), name.clone()),
                                            _ => todo!(),
                                        };

                                        let constructors = variant_count
                                            .get(&(module.clone(), name.clone()))
                                            .unwrap();

                                        let constructor = constructors
                                            .iter()
                                            .find(|c| c.name.as_str() == c_name.as_str())
                                            .unwrap();
                                        for arg in &constructor.arguments {
                                            new_row.push(Pattern::Discard {
                                                name: EcoString::new(),
                                                location: location.clone(),
                                                type_: arg.type_.clone(),
                                            });
                                        }
                                    }
                                    Tag::T => {
                                        // dbg!(typ);
                                        // match typ.borrow() {
                                        //     Type::Named { args, .. } => {
                                        //         for arg in args {
                                        //             new_row.push(Pattern::Discard {
                                        //                 name: EcoString::new(),
                                        //                 location: location.clone(),
                                        //                 type_: arg.clone(),
                                        //             });
                                        //         }
                                        //     }
                                        //     t => {dbg!(t);todo!()},
                                        // }
                                        continue 'pattern;
                                    }
                                    tag => {
                                        dbg!(tag);
                                        todo!()
                                    }
                                }

                                // continue 'pattern;
                            }
                            e => {
                                dbg!(e);
                                todo!()
                            }
                        }
                    }
                    Pattern::List {
                        elements: pattern_elements,
                        tail: pattern_tail,
                        ..
                    } => {
                        // dbg!(&tag);
                        match tag {
                            Tag::List { head_element, tail } => {
                                match (head_element, tail, pattern_tail) {
                                    (None, None, None) => {
                                        // println!("wrong?");
                                        // continue; //TODO I think this could be wrong....
                                        // break;
                                    }
                                    (Some(p), None, None) => {
                                        new_row.push(p.clone());
                                    }
                                    (Some(p1), Some(p2), Some(_)) => {
                                        let name = match p1 {
                                            Pattern::Variable {
                                                location,
                                                name,
                                                type_,
                                            } => name.clone(),
                                            _ => todo!(),
                                        };

                                        let tail_name = match p2.as_ref() {
                                            Pattern::Variable { name, .. } => name.clone(),
                                            _ => todo!(),
                                        };
                                        new_row.push(p1.clone());
                                        let list_name = match &matrix.hs[i] {
                                            TypedExpr::Var { name, .. } => name.clone(),
                                            _ => panic!(),
                                        };
                                        let _ = new_actions_and_env
                                            .1
                                            .insert(name, Binding::ListHead(list_name.clone()));
                                        new_row.push(p2.as_ref().clone());
                                        let _ = new_actions_and_env
                                            .1
                                            .insert(tail_name, Binding::ListTail(list_name));
                                    }
                                    _ => continue 'row, //Skip the row
                                                        // _ => panic!(),
                                }
                            }
                            _ => continue,
                        }
                    }
                    Pattern::Constructor {
                        location,
                        name,
                        arguments,
                        module,
                        constructor,
                        spread,
                        type_,
                    } => match tag {
                        Tag::Constructor(ref c_name) => {
                            if c_name == name {
                                //TODO check are the call args complete?
                                for argument in arguments {
                                    new_row.push(argument.value.clone());
                                }
                            } else {
                                continue 'row; //Skip the row
                            }
                        }
                        Tag::T => {
                            continue 'row; //skip the row
                        }
                        _ => {
                            todo!();
                        }
                    },
                    // Pattern::Tuple { location, elems } => todo!(),
                    // Pattern::BitArray { location, segments } => todo!(),
                    // Pattern::StringPrefix { location, left_location, left_side_assignment, right_location, left_side_string, right_side_assignment } => todo!(),
                    // Pattern::Invalid { location, type_ } => todo!(),
                    _ => continue, //No row, next iteration
                                   //TODO more for sure.
                }
            } else {
                new_row.push(matrix.patterns[row_idx][j].clone());
            }
        }
        if new_row.len() == new_matrix.hs.len() {
            // dbg!("len match");
            new_matrix.actions_and_env.push(new_actions_and_env.clone());
            //TODO add more bindings here? If they come from the pattern?
            new_matrix.patterns.push(new_row);
        } else {
            dbg!(new_row.len());
            dbg!(&matrix.hs.len());
            dbg!(new_matrix.hs.len());
            // panic!()
            dbg!(tag);
            dbg!(&matrix.patterns[row_idx][i]);
            dbg!(&matrix.hs[i]);
            dbg!(&new_matrix.hs);
            // dbg!(&new_row);
            // dbg!(&new_matrix.hs);
            dbg!("TODO?"); //I guess my continue was kinda stupid since no row with a continue in that spot is a half finished row...
                           // continue 'row;
            panic!();
        }
        // new_matrix
        // .actions_and_env
        // .push(matrix.actions_and_env[row_idx].clone()); //WHATTTTT about bindings and actions, huh!
    }

    // let eq = new_matrix.patterns[0].len() == new_matrix.hs.len();
    // if !eq {
    //     let pattern_len = new_matrix.patterns[0].len();
    //     let hs_len = new_matrix.hs.len();
    //     println!("pl {pattern_len}, hsl {hs_len} ");
    // }
    // println!("{eq}");
    // assert!(eq);

    if new_matrix.patterns.len() != new_matrix.actions_and_env.len() {
        panic!();
    }
    compile_tree(new_matrix, variant_count)
}

struct Score {
    index: usize,
    score: i32,
}

fn pick_column(
    matrix: &PatternMatrix,
    variant_count: HashMap<(EcoString, EcoString), Vec<RecordConstructor<Arc<Type>>>>,
) -> usize {
    assert!(matrix.patterns.first().is_some());
    //TODO qba might be better, but this one easier for testing
    // TODO too stupid
    let patterns = &matrix.patterns;
    let f_sorted = heuristic_f(patterns);
    if let (Some(Score { score: y, .. }), Some(Score { score: z, .. })) =
        (f_sorted.get(0), f_sorted.get(1))
    {
        if y == z {
            let top_scorers = f_sorted
                .iter()
                .filter_map(|s| if s.score == *y { Some(s.index) } else { None })
                .collect();
            let d_sorted = heuristic_d(patterns, top_scorers);
            if let (Some(Score { score: y, .. }), Some(Score { score: z, .. })) =
                (d_sorted.get(0), d_sorted.get(1))
            {
                if y == z {
                    let top_scorers = d_sorted
                        .iter()
                        .filter_map(|s| if s.score == *y { Some(s.index) } else { None })
                        .collect();
                    let b_sorted = heuristic_b(patterns, top_scorers, variant_count);
                    return b_sorted[0].index;
                } else {
                    return d_sorted[0].index;
                }
            }
        }
    }
    return f_sorted[0].index;
}

fn heuristic_f(patterns: &Patterns) -> Vec<Score> {
    patterns[0]
        .iter()
        .enumerate()
        .map(|(index, p)| Score {
            index,
            score: if let Pattern::Discard { .. } = p {
                0
            } else {
                1
            },
        })
        .collect()
}

fn heuristic_d(patterns: &Patterns, included: Vec<usize>) -> Vec<Score> {
    score(
        patterns,
        Box::new(|p| match p {
            Pattern::Discard { .. } => -1,
            _ => 0,
        }),
    )
    .into_iter()
    .filter(|s| included.contains(&s.index))
    .collect()
}
fn heuristic_b(
    patterns: &Patterns,
    included: Vec<usize>,
    variant_count: HashMap<(EcoString, EcoString), Vec<RecordConstructor<Arc<Type>>>>,
) -> Vec<Score> {
    score(
        patterns,
        Box::new(move |p| {
            match p {
                Pattern::List { .. } => -2,
                Pattern::Constructor { type_, .. } => match type_.borrow() {
                    Type::Named { name, module, .. } => match name.as_str() {
                        "Nil" => -1,
                        "Bool" | "Result" => -2,
                        _ => {
                            match variant_count.get(&(module.clone(), name.clone())) {
                                Some(x) => -(x.len() as i32),
                                None => -3,
                            }
                            // -((*variant_count
                            //     .get(&(module.clone(), name.clone()))
                            //     .unwrap_or(&3)) as i32)
                        }
                    },
                    _ => panic!(), //Unreachable..
                },
                _ => -1,
            }
        }),
    )
    .into_iter()
    .filter(|s| included.contains(&s.index))
    .collect()
}

fn score(patterns: &Patterns, f: Box<dyn Fn(&Pattern<Arc<Type>>) -> i32>) -> Vec<Score> {
    //Score columns, yes?
    let mut scores: Vec<i32> = (0..patterns[0].len()).into_iter().map(|_i| 0).collect();

    patterns.iter().for_each(|row| {
        row.iter().enumerate().for_each(|(i, p)| {
            scores[i] += f(p);
        })
    });

    let mut scores: Vec<Score> = scores
        .iter()
        .enumerate()
        .map(|(index, score)| Score {
            index,
            score: *score,
        })
        .collect();
    scores.sort_by_cached_key(|s| s.score);
    scores
}

#[derive(Debug)]
pub enum DecisionTree {
    Switch {
        discriminant: TypedExpr,
        cases: Vec<(Case, Box<DecisionTree>)>,
    },
    Success {
        branch: TypedExpr,
        bindings: Bindings,
    },
    Unreachable,
}

type Bindings = HashMap<EcoString, Binding>; //TypedExpr::Var

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Binding {
    Expr(TypedExpr),
    ListHead(EcoString),
    ListTail(EcoString),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Case {
    ConstructorEquality { constructor: EcoString }, //TypedExpr::BinOp
    ConstantEquality(EcoString),
    EmptyList,
    List,
    Default,
}

type Patterns = Vec<Vec<Pattern<Arc<Type>>>>;
#[derive(Debug, Clone)]
struct PatternMatrix {
    hs: Vec<TypedExpr>,
    patterns: Vec<Vec<Pattern<Arc<Type>>>>,
    actions_and_env: Vec<(TypedExpr, Bindings)>,
}

//TODO remove clone derive
#[derive(Eq, PartialEq, Debug, Clone)]
enum Tag {
    T, //catchall
    Constructor(EcoString),
    Constant(EcoString),
    List {
        head_element: Option<Pattern<Arc<Type>>>,
        tail: Option<Box<Pattern<Arc<Type>>>>,
    },
}

impl Hash for Tag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // core::mem::discriminant(self).hash(state);
        match self {
            Tag::T => core::mem::discriminant(self).hash(state),
            Tag::Constructor(c) => c.hash(state),
            Tag::Constant(_) => todo!(),
            Tag::List { head_element, tail } => core::mem::discriminant(self).hash(state), //TODO!
        }
    }
}

impl Ord for Case {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Case::Default, _) => Ordering::Less,
            (_, Case::Default) => Ordering::Greater,
            (_, _) => Ordering::Equal,
        }
    }
}

impl PartialOrd for Case {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
