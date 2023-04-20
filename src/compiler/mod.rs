use std::{fmt::{Debug, Formatter, self, Display}, collections::HashMap, hash::Hash};

use super::parser;

#[derive(Clone, Hash, Eq)]
pub struct Type {
    pub name: String,
    pub size: u64,
}

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<Type {:?} size={} />", self.name, self.size)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq<Type> for Type {
    fn eq(&self, other: &Type) -> bool {
        return self.name == other.name && self.size == other.size;
    }
}

pub fn int_type() -> Type {
    return Type {
        name: String::from("int"),
        size: 4,
    };
}

pub fn bool_type() -> Type {
    return Type {
        name: String::from("bool"),
        size: 1,
    };
}

pub fn float_type() -> Type {
    return Type {
        name: String::from("float"),
        size: 8,
    };
}

pub fn array_type() -> Type {
    return Type {
        name: String::from("array"),
        size: 8,
    };
}

pub fn string_type() -> Type {
    return Type {
        name: String::from("str"),
        size: 8,
    };
}

#[derive(Clone)]
struct Variable {
    name: String,
    typeval: Type,
}

#[derive(Clone, Eq)]
struct FunctionDeclaration {
    name: String,
    parameters: Vec<Type>,
    return_type: Option<Type>,
    implemented: bool,
}

struct Function {
    name: String,
    statements: Vec<parser::Ast>,
}

impl ToString for FunctionDeclaration {
    fn to_string(&self) -> String {
        let mut res = String::new();
        res.push_str(self.name.as_str());
        res.push('(');
        if self.parameters.len() >= 1 {
            let mut iter = self.parameters.iter();
            res.push_str(iter.next().unwrap().name.as_str());
            for val in iter {
                res.push_str(format!(",{}", val.name).as_str());
            }
        }
        res.push(')');
        res.push_str(format!(": {}", match &self.return_type {
            Some(val) => val.name.clone(),
            None => String::from("void"),
        }).as_str());
        return res;
    }
}

impl Hash for FunctionDeclaration {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.parameters.hash(state);
        self.name.hash(state);
    }   
}

impl PartialEq<FunctionDeclaration> for FunctionDeclaration {
    fn eq(&self, other: &FunctionDeclaration) -> bool {
        return self.name == other.name && self.parameters == other.parameters;
    }
}

#[derive(Clone)]
struct Scope {
    functions: Vec<FunctionDeclaration>,
    variables: Vec<Variable>,
    types: Vec<Type>,
    parent: Option<Box<Scope>>,
    functions_symbol_table: HashMap<FunctionDeclaration, String>,
}

impl Scope {

    fn new_global_scope() -> Self {
        return Scope {
            functions: Vec::new(),
            variables: Vec::new(),
            types: vec![int_type(), float_type(), string_type(), bool_type(), array_type()],
            functions_symbol_table: HashMap::new(),
            parent: None,
        };
    }

    fn new(parent: Option<Box<Scope>>) -> Self {
        return Scope {
            functions: Vec::new(),
            variables: Vec::new(),
            types: Vec::new(),
            parent,
            functions_symbol_table: HashMap::<FunctionDeclaration, String>::new(),
        };
    }
}

fn get_variable_type(name: &str, scope: &Scope) -> Result<Type, String> {

    return match scope.variables.iter().filter(|&v| v.name == name).next() {
        Some(val) => Ok(val.typeval.clone()),
        None => Err(format!("compiler: undefined variable '{}'", name)),
    };
}

fn function_exists(name: &str, param_types: &Vec<Type>, scope: &Scope) -> Option<FunctionDeclaration> {

    for dec in &scope.functions {
        if dec.name != name {
            continue;
        }
        if param_types == &dec.parameters {
            return Some(dec.clone());
        }
    }

    if let Some(parent_scope) = &scope.parent {
        return function_exists(name, param_types, &parent_scope);
    }

    return None;
}

fn get_function_return_type(name: &str, param_types: &Vec<Type>, scope: &Scope) -> Result<Option<Type>, String> {
    return match function_exists(name, param_types, scope) {
        None => Err(format!("no function with the following signature: {}({:?})", name, param_types)),
        Some(dec) => Ok(dec.return_type),
    };
}

fn calculate_expression_type(expression: &parser::Ast, scope: &Scope) -> Result<Type, String> {

    return match expression {
        parser::Ast::Int(..) => Ok(int_type()),
        parser::Ast::Float(..) => Ok(float_type()),
        parser::Ast::Bool(..) => Ok(bool_type()),
        parser::Ast::ArrayValue(..) => Ok(array_type()),
        parser::Ast::Str(..) => Ok(string_type()),
        parser::Ast::EqualTo {..}
        | parser::Ast::NotEqualTo {..}
        | parser::Ast::GreaterThan {..}
        | parser::Ast::GreaterOrEqual {..}
        | parser::Ast::LowerThan {..}
        | parser::Ast::LowerOrEqual {..}
            => Ok(bool_type()),
        parser::Ast::Substraction { left, right }
        | parser::Ast::Addition { left, right }
        | parser::Ast::Division { left, right }
        | parser::Ast::Multiplication { left, right }
        | parser::Ast::Modulo { left, right }
        => {
            let type1 = match calculate_expression_type(right, scope) {
                Err(e) => return Err(e),
                Ok(val) => val,
            };
            let type2 = match calculate_expression_type(left, scope) {
                Err(e) => return Err(e),
                Ok(val) => val,
            };

            if type1 != type2 {
                if type1 == float_type() && (type2 == float_type() || type2 == int_type()) {
                    Ok(type1)
                } else if type2 == float_type() && (type1 == int_type() || type1 == float_type()) {
                    Ok(type2)
                } else {
                    Err(format!("mismatching types '{}' and '{}'", type1.name, type2.name))
                }
            } else {
                Ok(type1)
            }
        },
        parser::Ast::Variable(var) => get_variable_type(&var.name, &scope),
        parser::Ast::FunctionCall { name, children } => {
            let mut types = Vec::<Type>::new();
            for child in children {
                types.push(match calculate_expression_type(child, &scope) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                });
            }
            match get_function_return_type(name, &types, scope) {
                Err(e) => return Err(e),
                Ok(val) => match val {
                    None => return Err(format!("function with void return type cannot be used as an expression.")),
                    Some(val) => Ok(val),
                },
            }
        },
        _ => todo!(),
    };
}

fn get_type(typename: String, scope: &Scope) -> Result<Type, String> {
    if let Some(typeval) = scope.types.iter().filter(|&t| t.name == typename).next() {
        return Ok(typeval.clone());
    } else if let Some(parent_scope) = scope.parent.clone() {
        return get_type(typename, parent_scope.as_ref());
    } else {
        return Err(format!("undefined type {:?}", typename));
    }
}

fn convert_type(old_type: &Option<String>, scope: &Scope) -> Result<Option<Type>, String> {
    if let Some(val) = old_type {
        return match get_type(val.clone(), scope) {
            Err(e) => return Err(e),
            Ok(val) => Ok(Some(val)),
        };
    } else {
        return Ok(None);
    }
}

fn convert_params(parser_params: &Vec<parser::Variable>, scope: &Scope) -> Result<Vec<Type>, String> {
    let mut result = Vec::<Type>::new();
    for param in parser_params {
        let parser_type = param.typename.clone().unwrap();
        let typeval = match get_type(parser_type.name.clone(), &scope) {
            Ok(typeval) => typeval,
            Err(e) => return Err(e),
        };

        result.push(typeval);
    }
    return Ok(result);
}

fn build_function_name(scope_name: String, declaration: &FunctionDeclaration) -> String {
    return format!("{}_{}", scope_name, declaration.to_string());
}

fn get_function_effective_name(declaration: &FunctionDeclaration, scope: &Scope) -> Result<String, String> {
    if let Some(val) = scope.functions_symbol_table.get(declaration) {
        return Ok(val.clone());
    } else if let Some(parent_scope) = &scope.parent {
        return get_function_effective_name(declaration, parent_scope);
    } else {
        return Err(format!("undefined symbol {}", declaration.to_string()));
    }
}

fn flatten_tree(children: &Vec<parser::Ast>, scope: Scope, scope_name: String, func_impl: &mut Vec<parser::Ast>, extern_symbols: &mut Vec<FunctionDeclaration>) -> Result<Vec<Function>, String> {
    let mut children_functions = Vec::<Function>::new();
    let mut scope = scope;
    for child in children {
        match child {
            parser::Ast::FunctionDeclaration { name, children, parameters, return_type }
            => {
                let parameters = match convert_params(parameters, &scope) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                let return_type = match convert_type(return_type, &scope) {
                    Err(e) => return Err(e),
                    Ok(val) => val,
                };

                let dec = FunctionDeclaration {
                    name: name.clone(),
                    parameters,
                    return_type,
                    implemented: true,
                };

                match scope.functions_symbol_table.get_key_value(&dec) {
                    Some((key, ..)) if key.implemented => return Err(format!("redeclaration of function {}", dec.to_string())),
                    Some((key, ..)) if key.return_type != dec.return_type
                        => return Err(
                            format!(
                                "invalid return type for function {}, expected {}, found {}", dec.to_string(),
                                match &key.return_type {
                                    None => String::from("void"),
                                    Some(val) => val.name.clone(),
                                },
                                match &dec.return_type {
                                    None => String::from("void"),
                                    Some(val) => val.name.clone(),
                                },
                            )
                        ),
                    _ => (),
                };

                scope.functions.push(dec.clone());

                let function_name = build_function_name(scope_name.clone(), &dec);
                scope.functions_symbol_table.remove(&dec);
                scope.functions_symbol_table.insert(dec.clone(), function_name.clone());

                let sub_scope = Scope::new(Some(Box::new(scope.clone())));
                let mut statements = Vec::<parser::Ast>::new();
                let sub_functions = match flatten_tree(
                    children,
                    sub_scope, 
                    format!("{}_{}", scope_name.clone(), name.clone()),
                    &mut statements,
                    extern_symbols,
                ) {
                    Err(e) => return Err(e),
                    Ok(val) => val,
                };
                for f in sub_functions {
                    children_functions.push(f);
                }
                children_functions.push(Function {
                    name: function_name.clone(),
                    statements,
                });
            },
            parser::Ast::FunctionHeader { name, parameters, return_type }
            if match scope.parent {None => true, _ => false,} => {
                let parameters = match convert_params(parameters, &scope) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                let return_type = match convert_type(return_type, &scope) {
                    Err(e) => return Err(e),
                    Ok(val) => val,
                };

                let dec = FunctionDeclaration {
                    name: name.clone(),
                    parameters,
                    return_type,
                    implemented: false,
                };

                match scope.functions_symbol_table.get(&dec) {
                    Some(..) => return Err(format!("redeclaration of function {}", dec.to_string())),
                    None => (),
                };

                scope.functions.push(dec.clone());
                let function_name = build_function_name(scope_name.clone(), &dec);
                scope.functions_symbol_table.insert(dec.clone(), function_name.clone());
            },
            parser::Ast::FunctionCall { name, children } => {
                let mut types = Vec::<Type>::new();
                for child in children {
                    types.push(match calculate_expression_type(child, &scope) {
                        Err(e) => return Err(e),
                        Ok(val) => val,
                    });
                }

                let dec = match function_exists(name.as_str(), &types, &scope) {
                    None => return Err(format!("undefined function {}", name)),
                    Some(val) => val,
                };

                let effective_name = match get_function_effective_name(&dec, &scope) {
                    Err(e) => return Err(e),
                    Ok(val) => val,
                };

                func_impl.push(parser::Ast::FunctionCall { 
                    name: effective_name.clone(),
                    children: children.clone(), 
                });
            },
            parser::Ast::FunctionHeader {..} => return Err(format!("cannot create nested function declarations")),
            child => func_impl.push(child.clone()),
        }

    }

    for dec in scope.functions_symbol_table.keys().filter(|f| !f.implemented) {
        extern_symbols.push(dec.clone());
    }

    return Ok(children_functions);
}

struct CompilerContext {
    functions: Vec<Function>,
    main_function: Vec<Function>,
    extern_symbols: Vec<String>,
}

pub fn test(ast: &parser::Ast) {
    let children = match ast {
        parser::Ast::Global(children) => children,
        _ => return,
    };

    let mut main_func = Function {
        name: String::from("main"),
        statements: Vec::new(),
    };

    let mut extern_symbols = Vec::<FunctionDeclaration>::new();

    let functions = match flatten_tree(&children, Scope::new_global_scope(), String::new(), &mut main_func.statements, &mut extern_symbols) {
        Err(e) => panic!("{}", e),
        Ok(f) => f,
    };

    for dec in extern_symbols {
        println!("extern: {}", dec.to_string());
    }

    for f in functions {
        println!("{}", f.name);
        for child in f.statements {
            println!("{:?}", child);
        }
    }

    println!("main function");
    for child in main_func.statements {
        println!("{:?}", child);
    }
}
