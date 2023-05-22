use std::collections::HashMap;
use super::*;
use super::typecheck::Context;

pub type Id = String;
pub type ModId = String;

#[derive(Debug, Clone)]
pub struct Circuit {
    pub top: Id,
    // annotations,
    // info
    pub decls: Vec<Decl>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Mod(ModDef),
    ExtMod(ExtModDef),
    IntMod(IntModDef),
}

#[derive(Debug, Clone)]
pub struct ExtModDef;

#[derive(Debug, Clone)]
pub struct IntModDef;

#[derive(Debug, Clone)]
pub struct ModDef {
    pub name: Id,
    pub ports: Vec<Port>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub direction: Direction,
    pub name: Id,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Wire(Id, Type),
    Reg(Id, Type), // todo!()
    Inst(Id, ModId),
    Node(Id, Box<Expr>),
    // Mem
    Assign(RefPath, Box<Expr>),
    Connect(RefPath, Box<Expr>), // todo!() what even is this?
    Invalidate(RefPath),
    Attach(RefPath, RefPath),
    When(Box<Expr>, Vec<Statement>, Option<Vec<Statement>>),
    Skip,

    // stop
    // printf
    // define
    // force_release
    // RegSet
}

impl ModDef {
    pub fn refpaths(&self) -> Vec<RefPath> {
        let mut refpaths = vec![];

        for Port { name, .. }  in &self.ports {
            refpaths.push(name.as_str().into());
        }

        for statement in &self.statements {
            match statement {
                Statement::Wire(id, _typ) => refpaths.push(id.as_str().into()),
                Statement::Reg(id, _typ) => refpaths.push(id.as_str().into()),
                Statement::Inst(id, _mod_id) => refpaths.push(id.as_str().into()),
                Statement::Node(id, _e) => refpaths.push(id.as_str().into()),
                // Statement::Mem // todo!()
                // When() ... internal statements // todo!()
                _ => (),
            }
        }

        refpaths
    }

    pub fn port(&self, name: &str) -> Option<Port> {
        for port in &self.ports {
            if port.name == name {
                return Some(port.clone());
            }
        }
        None
    }

    pub fn components(&self) -> HashMap<String, Component> {
        let mut result = HashMap::new();
        for port in &self.ports {
            result.insert(port.name.to_string(), Component::Port(port.typ.clone(), port.direction));
        }

        for (statement, context) in &self.statements_with_contexts() {
            match statement {
                Statement::Wire(id, typ) => { result.insert(id.to_string(), Component::Wire(typ.clone())); },
                Statement::Reg(id, typ) => { result.insert(id.to_string(), Component::Reg(typ.clone())); },
                Statement::Inst(id, _mod_id) => { result.insert(id.to_string(), Component::Inst()); },
                Statement::Node(id, e) => {
                    let typ = context.infer_type(e).unwrap();
                    result.insert(id.to_string(), Component::Node(typ));
                },
                // Statement::Mem // todo!()
                // When() ... internal statements // todo!()
                _ => (),
            }
        }
        result
    }

    fn context_from_ports(&self) -> Context<Type> {
        let mut context = Context::empty();
        for port in &self.ports {
            context = context.extend(port.name.to_string(), port.typ.clone());
        }
        context
    }

    pub fn statements_with_contexts(&self) -> Vec<(Statement, Context<Type>)> {
        let _result: Vec<(Statement, Context<Type>)> = vec![];
        let _context = self.context_from_ports();

        todo!()
    }
}
