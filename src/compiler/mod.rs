use std::rc::Rc;

use super::parser;

pub trait Visitor<T> {
    fn visit(element: &parser::Ast) -> T;
}

pub struct Compiler;

#[derive(Clone)]
struct Type {
    name: &'static str,
    size: u64,
}

impl PartialEq<Type> for Type {
    fn eq(&self, other: &Type) -> bool {
        return self.name == other.name && self.size == other.size;
    }
}

const INT: Type = Type {
    name: "int",
    size: 4,
};

const BOOL: Type = Type {
    name: "bool",
    size: 1,
};

const FLOAT: Type = Type {
    name: "float",
    size: 8,
};

const ARRAY: Type = Type {
    name: "",
    size: 8,
};

#[derive(Clone)]
struct Scope {
    variables: Vec<Variable>,
    types: Vec<Type>,
    parent: Option<Rc<Scope>>,
}

impl Scope {
    fn new_global_scope() -> Rc<Self> {
        return Rc::new(Scope {
            variables: Vec::new(),
            types: vec![INT, FLOAT, ARRAY],
            parent: None,
        });
    }

    fn build_child(parent: Rc<Self>) -> Rc<Self> {
        return Rc::new(Scope {
            variables: Vec::new(),
            types: Vec::new(),
            parent: Some(parent),
        });
    }
}

#[derive(Clone)]
struct Variable {
    name: String,
    typeval: Type,
}

struct Function {
    parameters: Vec<Variable>,
    variables: Vec<Variable>,
    statements: Vec<parser::Ast>,
}

impl Function {
    fn new() -> Self {
        return Function { parameters: Vec::new(), variables: Vec::new(), statements: Vec::new() };
    }
}

fn collect_variables(statements: &Vec<parser::Ast>, scope: &mut Scope) -> Result<(), String> {

    let mut variables = Vec::<Variable>::new();
    for statement in statements {
        match statement {
            parser::Ast::Assignement { variable, expression } => {
                let result: Variable;
                let var = match &**variable {
                    parser::Ast::Variable(var) => var,
                    val => return Err(format!("expected variable, instead go {:?}", val)),
                };
                
                let expression_type = match calculate_expression_type(expression, scope) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                if let Some(typename) = &var.typename {
                    if typename.name != expression_type.name {
                        return Err(format!("mismatching types for variable '{}', variable is of type '{}', and expression if of type '{}'", var.name, typename.name, expression_type.name));
                    }
                    result = Variable {
                        name: var.name.clone(),
                        typeval: expression_type,
                    };
                } else {
                    result = Variable {
                        name: var.name.clone(),
                        typeval: expression_type,
                    };
                }

                // TODO: check for invalid duplicates.
                variables.push(result);
            },
            _ => (),
        };
    }

    scope.variables = variables;
    return Ok(());
}

fn calculate_expression_type(expression: &parser::Ast, scope: &Scope) -> Result<Type, String> {

    let mut current_type: Option<Type> = None;

    return match expression {
        parser::Ast::Int(_) => Ok(INT),
        parser::Ast::Float(_) => Ok(FLOAT),
        parser::Ast::ArrayValue(..)
            | parser::Ast::Str(..) => Ok(ARRAY),
        parser::Ast::EqualTo {..}
        | parser::Ast::NotEqualTo {..}
        | parser::Ast::GreaterThan {..}
        | parser::Ast::GreaterOrEqual {..}
        | parser::Ast::LowerThan {..}
        | parser::Ast::LowerOrEqual {..}
            => Ok(BOOL),
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
                if type1 == FLOAT && (type2 == FLOAT || type2 == INT) {
                    Ok(type1)
                } else if type2 == FLOAT && (type1 == INT || type1 == FLOAT) {
                    Ok(type2)
                } else {
                    Err(format!("mismatching types '{}' and '{}'", type1.name, type2.name))
                }
            } else {
                Ok(type1)
            }
        },
        _ => todo!(),
    };
}

fn collect_function_symbols(parameters: &Vec<parser::Variable>, statements: &Vec<parser::Ast>) -> Function {
    return Function::new();
}

impl Visitor<String> for Compiler {
    fn visit(element: &parser::Ast) -> String {
        
        return String::new();
    }
}
