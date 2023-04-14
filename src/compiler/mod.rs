use std::{rc::Rc, fmt::{Debug, Formatter, self}, cell::RefCell};

use super::parser;

pub trait Visitor<T> {
    fn visit(&self, element: &parser::Ast) -> T;
}

pub struct Compiler;

#[derive(Clone)]
pub struct Type {
    pub name: String,
    pub size: u64,
}

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<Type {:?} size={} />", self.name, self.size)
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

pub fn void_type() -> Type {
    return Type {
        name: String::from("void"),
        size: 0,
    }
}

#[derive(Clone)]
pub struct Scope {
    variables: Vec<Variable>,
    functions: Vec<Function>,
    types: Vec<Type>,
    parent: Option<Box<Scope>>,
}

impl Debug for Scope {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Scope {{ types: {:?}, variables: {:?}, functions: {:?}, }}", self.types, self.variables, self.functions)
    }
}

impl Scope {
    fn new_global_scope() -> Self {
        return Scope {
            variables: Vec::new(),
            types: vec![int_type(), float_type(), array_type(), string_type()],
            functions: Vec::new(),
            parent: None,
        };
    }

    fn new() -> Self {
        return Scope {
            variables: Vec::new(),
            types: Vec::new(),
            functions: Vec::new(),
            parent: None,
        };
    }
}

#[derive(Clone)]
pub struct Variable {
    name: String,
    typeval: Type,
}

impl Debug for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<Var {:?} type={:?} />", self.name, self.typeval)
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Variable>,
    pub scope: Box<Scope>,
    pub return_type: Option<Type>,
}

fn add_variable_to_scope(var: Variable, scope: Scope) -> Result<Scope, String> {
    let mut scope = scope;
    let exisiting_var = match scope.variables.iter().filter(|&v| v.name == var.name).next() {
        Some(val) => val,
        None => {
            scope.variables.push(var);
            return Ok(scope);
        },
    };

    return match exisiting_var.typeval == var.typeval {
        true => Ok(scope),
        false => Err(format!("cannot change type of variable {:?} from {:?} to {:?}", &var.name, &exisiting_var.typeval, &var.typeval)),
    };
}

fn collect_symbols(statements: &Vec<parser::Ast>, scope: Scope) -> Result<Scope, String> {
    let mut scope = scope;

    for statement in statements {
        match statement {
            parser::Ast::Assignement { variable, expression } => {
                let result: Variable;
                let var = match &**variable {
                    parser::Ast::Variable(var) => var,
                    val => return Err(format!("expected variable, instead go {:?}", val)),
                };
                
                let expression_type = match calculate_expression_type(expression, &scope) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };

                if let Some(typename) = &var.typename {
                    if typename.name != expression_type.name {
                        return Err(format!("mismatching types for variable '{}', variable is of type '{}', and expression of type '{}'", var.name, typename.name, expression_type.name));
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
                scope = match add_variable_to_scope(result, scope) {
                    Ok(scope) => scope,
                    Err(e) => return Err(e),
                };
            },
            parser::Ast::FunctionDeclaration {..}
            => {
                scope.functions.push(match create_function(statement, Box::new(scope.clone())) {
                    Ok(f) => f,
                    Err(e) => return Err(e),
                });
            },
            _ => (),
        };
    }
    return Ok(scope);
}

fn get_variable_type(name: &str, scope: &Scope) -> Result<Type, String> {

    return match scope.variables.iter().filter(|&v| v.name == name).next() {
        Some(val) => Ok(val.typeval.clone()),
        None => Err(format!("compiler: undefined variable '{}'", name)),
    };
}


fn get_function_return_type(name: &str, scope: &Scope) -> Result<Type, String> {
    if let Some(func) = scope.functions.iter().filter(|&f| f.name == name).next() {
        return Ok(match func.return_type.clone() {
            Some(t) => t,
            None => void_type(),
        });
    } else if let Some(parent_scope) = scope.parent.clone() {
        return get_function_return_type(name, &parent_scope);
    } else {
        return Err(format!("compiler: undefined function '{}'", name));
    }
}

pub fn calculate_expression_type(expression: &parser::Ast, scope: &Scope) -> Result<Type, String> {

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
        parser::Ast::FunctionCall { name, .. } => get_function_return_type(name, &scope),
        _ => todo!(),
    };
}

fn collect_function_symbols(parameters: &Vec<Variable>, statements: &Vec<parser::Ast>, scope: Scope) -> Result<Scope, String> {
    let mut scope = scope;
    for param in parameters {
        scope.variables.push(param.clone());
    }
    return collect_symbols(statements, scope);
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

fn convert_params(parser_params: &Vec<parser::Variable>, scope: &Scope) -> Result<Vec<Variable>, String> {
    let mut result = Vec::<Variable>::new();
    for param in parser_params {
        let parser_type = param.typename.clone().unwrap();
        let typeval = match get_type(parser_type.name.clone(), &scope) {
            Ok(typeval) => typeval,
            Err(e) => return Err(e),
        };

        result.push(Variable { name: param.name.clone(), typeval });
    }
    return Ok(result);
}

fn create_function(ast: &parser::Ast, parent_scope: Box<Scope>) -> Result<Function, String> {
    let (
        name,
        parser_children,
        parser_params,
        return_type_opt,
    ) = match ast {
        parser::Ast::FunctionDeclaration { name, children, parameters, return_type }
        => (name, children, parameters, return_type),
        _ => return Err(format!("unexpected token type {:?}, expected function declaration", ast)),
    };

    let mut scope = Scope::new();
    scope.parent = Some(parent_scope.clone());

    let parameters = match convert_params(parser_params, parent_scope.as_ref()) {
        Ok(children) => children,
        Err(e) => return Err(e),
    };

    let scope = match collect_function_symbols(&parameters, &parser_children, scope) {
        Err(e) => return Err(e),
        Ok(f) => f,
    };

    let return_type = match return_type_opt {
        Some(val) => Some(match get_type(val.clone(), &scope) {
            Err(e) => return Err(e),
            Ok(typeval) => typeval,
        }),
        None => None,
    };
    return Ok(Function { name: name.clone(), parameters, scope: Box::new(scope), return_type });
}

impl Visitor<String> for Compiler {
    fn visit(&self, element: &parser::Ast) -> String {
        return String::new();
    }
}

pub fn test(ast: &parser::Ast) {
    let statements = match ast {
        parser::Ast::Global(children) => children,
        _ => return,
    };

    let scope = match collect_symbols(&statements, Scope::new_global_scope()) {
        Err(e) => panic!("{}", e),
        Ok(scope) => scope,
    };

    println!("{:?}", scope);
}
