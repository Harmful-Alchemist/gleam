use core::hash::Hash;

use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::{HashMap, HashSet},
    option::Option,
    sync::Arc,
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
    if matrix.patterns[0].is_empty()
        || matrix.patterns[0].iter().all(|p| match p {
            Pattern::Discard { .. } => true,
            _ => false,
        })
    {
        let (branch, bindings) = matrix.actions_and_env[0].clone();
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
    let mut tags = HashSet::new();
    let type_: &Type = type_.borrow();
    get_tags(type_, &mut matrix, i, &mut tags, &variant_count);

    //3 Compile the decision sub-trees corresponding to each branch
    let mut cases = Vec::new();
    for tag in &tags {
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
            } => Case::ConstructorEquality {
                constructor: "Empty list".into(),
            },
            Tag::List {
                head_element: _,
                tail: _,
            } => Case::ConstructorEquality {
                constructor: "Cons list".into(),
            },
            x => {
                println!("{x:?}");
                todo!()
            }
        };
        cases.push((case, branch));
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
    tags: &mut HashSet<Tag>,
    variant_count: &HashMap<(EcoString, EcoString), Vec<RecordConstructor<Arc<Type>>>>,
) {
    match type_ {
        Type::Named { module, name, .. } => {
            let len = matrix.patterns.len();
            for row_idx in 0..len {
                match &matrix.patterns[row_idx][i] {
                    Pattern::Constructor { name, .. } => {
                        let _ = tags.insert(Tag::Constructor(name.clone()));
                    }
                    Pattern::Variable { name, .. } => {
                        let val = matrix.hs[i].clone();
                        let _ = matrix.actions_and_env[row_idx].1.insert(name.clone(), val);
                        let _ = tags.insert(Tag::T);
                    }
                    Pattern::List {
                        location,
                        elements,
                        tail,
                        type_,
                    } => {
                        // if elements.len() != 1 {
                        //     println!("\n{elements:?}\n{tail:?}\n");
                        //     todo!();
                        // }
                        if elements.len() == 1 {
                            let _ = tags.insert(Tag::List {
                                head_element: Some(elements[0].clone()),
                                tail: tail.clone(),
                            });
                        } else if elements.len() == 0 && tail.is_none() {
                            //Empty list.
                            let _ = tags.insert(Tag::List {
                                head_element: None,
                                tail: tail.clone(),
                            });
                        } else {
                            todo!()
                        }
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
                    let _ = tags.insert(Tag::T);
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
                    let _ = tags.insert(Tag::T);
                } //Hmm unbound type vars...
            }
        }
        Type::Tuple { .. } => {
            let _ = tags.insert(Tag::T);
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
    for (j, h) in matrix.hs.iter().enumerate() {
        if i == j {
            // println!("h:\n {h:?}\nt: {tag:?}\n");
            match h {
                TypedExpr::Var { constructor, .. } => {
                    match tag {
                        Tag::Constructor(c_name) => {
                            //No field map? Nope variant is local variable.
                            let c_t: &Type = constructor.type_.borrow();
                            // println!("constructor: {constructor:?}\nconstructor type {c_t:?}\n");
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

                                                let constructor = constructors
                                                    .iter()
                                                    .find(|c| c.name.as_str() == c_name.as_str())
                                                    .unwrap();
                                                for (index, arg) in
                                                    constructor.arguments.iter().enumerate()
                                                {
                                                    let label = match &arg.label {
                                                        Some((l, _)) => l.clone(),
                                                        None => EcoString::new(), //Will this work?
                                                    };
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
                            // if let Some(map) = constructor.field_map() {
                            //     println!("fieldmap: {map:?}");
                            //     for (field_name, idx) in map.fields.iter() {
                            //         new_matrix.hs.push(TypedExpr::RecordAccess {
                            //             location: h.location(),
                            //             typ: h.type_(), //TODO does not make sense but type checking was done so why care? If we have to declare typed locals maybe based on this.
                            //             label: field_name.clone(),
                            //             index: *idx as u64,
                            //             record: Box::new(h.clone()),
                            //         });
                            //     }
                            // }
                        }
                        Tag::List { head_element, tail } => {
                            // println!("{tag:?}");
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
                                    // let thing = &matrix.hs[i];
                                    // // println!("thhing: {thing:?}");
                                    // match &matrix.hs[i] {
                                    //     TypedExpr::List { elements, tail, .. } => {
                                    //         println!("even used???");
                                    //         new_matrix.hs.push(elements[0].clone());
                                    //         new_matrix
                                    //             .hs
                                    //             .push(tail.clone().unwrap().as_ref().clone());
                                    //         //Stupid
                                    //     }
                                    //     TypedExpr::Var { name, .. } => {
                                    //         //Oooooooh shoot this is the whole list duh get the head and tail! Get them from the tag? whut....Looks like!
                                    //         // new_matrix.hs.push(thing.clone());
                                    //         //TODO use name???? Add to env? Prolly yeah
                                    //         // println!("add to env? Or we have var name already so eh....");
                                    //         if let Pattern::List { elements, .. } = p1 {
                                    //             if let Some(Pattern::Variable {
                                    //                 location,
                                    //                 name,
                                    //                 type_,
                                    //             }) = elements.first()
                                    //             {
                                    //                 new_matrix.hs.push(TypedExpr::Var { location: location.clone(), constructor: ValueConstructor {
                                    //             publicity: Publicity::Internal,
                                    //             deprecation: Deprecation::NotDeprecated,
                                    //             variant: ValueConstructorVariant::LocalVariable{location: location.clone()},
                                    //             type_: type_.clone(),
                                    //         }, name: name.clone() })
                                    //             } else {
                                    //                 panic!();
                                    //             }
                                    //         } else {
                                    //             dbg!(p1);
                                    //             dbg!(p2);
                                    //             // Oh shoot yeah now we recurr but ehm well now.........
                                    //             //Maybe we need to start waaaaaaaaaaay simpler!
                                    //             //go on here next time! Long match
                                    //             // but only on last pattern or smth Just constructor then can generate the code.
                                    //             panic!();
                                    //         }

                                    //         if let Pattern::Variable {
                                    //             location,
                                    //             name,
                                    //             type_,
                                    //         } = p2.as_ref()
                                    //         {
                                    //             new_matrix.hs.push(TypedExpr::Var {
                                    //                 location: location.clone(),
                                    //                 constructor: ValueConstructor {
                                    //                     publicity: Publicity::Internal,
                                    //                     deprecation: Deprecation::NotDeprecated,
                                    //                     variant:
                                    //                         ValueConstructorVariant::LocalVariable {
                                    //                             location: location.clone(),
                                    //                         },
                                    //                     type_: type_.clone(),
                                    //                 },
                                    //                 name: name.clone(),
                                    //             });
                                    //         } else {
                                    //             panic!();
                                    //         }
                                    //     }
                                    //     _ => panic!(), //well now
                                    // }
                                }
                                _ => panic!(),
                            }
                        }
                        Tag::T => {
                            () //Do nothing? TODO could be wrong! Bu I think right
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
                _ => (), //TODO could be too little matching going on here?
            }
        } else {
            new_matrix.hs.push(h.clone())
        }
    }

    //f output for expand, could be one loop with above but eh.
    for (row_idx, row) in matrix.patterns.iter().enumerate() {
        let mut new_row = Vec::new();
        for j in 0..row.len() {
            if i == j {
                //TODO env!?
                let p = &matrix.patterns[row_idx][j];
                // println!("p:\n {p:?}\n");
                match &matrix.patterns[row_idx][j] {
                    // Pattern::Int { location, value } => todo!(),
                    // Pattern::Float { location, value } => todo!(),
                    // Pattern::String { location, value } => todo!(),
                    // Pattern::Variable { location, name, type_ } => todo!(),
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
                                _ => {
                                    todo!();
                                    continue;
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
                            _ => todo!(),
                        }
                    }
                    Pattern::List { .. } => {
                        match tag {
                            Tag::List { head_element, tail } => {
                                match (head_element, tail) {
                                    (None, None) => {
                                        println!("wrong?");
                                        continue; //TODO I think this could be wrong....
                                    }
                                    (Some(p), None) => {
                                        new_row.push(p.clone());
                                    }
                                    (Some(p1), Some(p2)) => {
                                        new_row.push(p1.clone());
                                        new_row.push(p2.as_ref().clone());
                                    }
                                    _ => panic!(),
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
                                for argument in arguments {
                                    new_row.push(argument.value.clone());
                                }
                            } else {
                                continue;
                            }
                        }
                        Tag::T => {
                            continue;
                        }
                        _ => {
                            todo!();
                            continue;
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
            new_matrix.patterns.push(new_row);
        } else {
            // println!("TODO?"); //I guess my continue was kinda stupid since no row with a continue in that spot is a half finished row...
        }
        new_matrix
            .actions_and_env
            .push(matrix.actions_and_env[row_idx].clone());
    }

    // let eq = new_matrix.patterns[0].len() == new_matrix.hs.len();
    // if !eq {
    //     let pattern_len = new_matrix.patterns[0].len();
    //     let hs_len = new_matrix.hs.len();
    //     println!("pl {pattern_len}, hsl {hs_len} ");
    // }
    // println!("{eq}");
    // assert!(eq);

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

type Bindings = HashMap<EcoString, TypedExpr>; //TypedExpr::Var

#[derive(Debug)]
pub enum Case {
    ConstructorEquality { constructor: EcoString }, //TypedExpr::BinOp
    ConstantEquality(EcoString),
    Default,
}

type Patterns = Vec<Vec<Pattern<Arc<Type>>>>;
#[derive(Debug, Clone)]
struct PatternMatrix {
    hs: Vec<TypedExpr>,
    patterns: Vec<Vec<Pattern<Arc<Type>>>>,
    actions_and_env: Vec<(TypedExpr, Bindings)>,
}

#[derive(Eq, PartialEq, Debug)]
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
