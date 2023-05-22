use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub parser);

pub mod lexer;
pub mod typecheck;
pub mod ast;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    UInt(Option<usize>),
    SInt(Option<usize>),
    Clock,
    Reset,
    AsyncReset,
//    Analog(Option<usize>), // todo!()
    Const(Box<Type>),
    Probe(Box<Type>),
    RwProbe(Box<Type>),
    Vec(usize, Box<Type>),
    Bundle(Vec<BundleField>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Flippedness {
    Aligned,
    Flipped,
}

pub type Fieldname = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleField(pub Flippedness, pub Fieldname, Box<Type>);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Input,
    Output,
}

#[derive(Debug, Clone)]
pub enum Component {
    Node(Type),
    Wire(Type),
    Reg(Type), // todo!() need reset value and clock
    Port(Type, Direction),
    Inst(), // todo!() need a mod def handle of some sort
    //InstPort(), // todo!()
}

#[derive(Debug, Clone)]
pub enum Value {
    UInt(usize, usize),
    SInt(usize, usize),
    Vec(Vec<Value>),
    //Ref(Path) // todo!()
    // Clock?
    // AsyncReset?
}

pub type ComponentRef = String;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Op {}

#[derive(Debug, Clone)]
pub enum RefPath {
    Id(String),
    Dot(Box<RefPath>, String),
    Index(Box<RefPath>, isize),
    DynIndex(Box<RefPath>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Value),
    Var(RefPath),
    Mux(Box<Expr>, Box<Expr>, Box<Expr>),
    Op(Op, Vec<Expr>),
    // Read() // todo!()
}

impl Type {
    pub fn is_passive(&self) -> bool {
        match self {
            Type::UInt(_width) => true,
            Type::SInt(_width) => true,
            Type::Clock => true,
            Type::Reset => true,
            Type::AsyncReset => true,
            Type::Const(_typ) => true, // todo!() should this be typ.is_passive()?
            Type::Probe(_typ) => true,
            Type::RwProbe(_typ) => true,
            Type::Vec(_usize, _typ) => true,
            Type::Bundle(fields) => {
                for BundleField(flip, _field_name, typ) in fields {
                    if flip == &Flippedness::Flipped {
                        return false;
                    }
                    if !typ.is_passive() {
                        return false;
                    }
                }
                true
            },
        }
    }

    pub fn passify(&self) -> Type {
        match self {
            Type::UInt(_width) => self.clone(),
            Type::SInt(_width) => self.clone(),
            Type::Clock => self.clone(),
            Type::Reset => self.clone(),
            Type::AsyncReset => self.clone(),
            Type::Const(_typ) => self.clone(), // todo!()
            Type::Probe(_typ) => self.clone(),
            Type::RwProbe(_typ) => self.clone(),
            Type::Vec(usize, typ) => Type::Vec(*usize, Box::new(typ.passify())),
            Type::Bundle(fields) => {
                let new_fields = fields.iter().cloned().map(|field| field.align()).collect();
                Type::Bundle(new_fields)
            }
        }
    }

    // t1.connects_from(t2) means c1 <= c2 is legal
    // if c1 has type t1 and c2 has type t2
    pub fn connects_from(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Reset, Type::AsyncReset) => true,
            (Type::Reset, Type::UInt(Some(1))) => true,
            (t1, t2) => t1 == t2,
        }
    }
}

impl BundleField {
    pub fn align(&self) -> BundleField {
        let BundleField(_flip, field_name, typ) = self.clone();
        BundleField(Flippedness::Aligned, field_name, typ)
    }
}

impl Expr {
    pub fn min_width(&self) -> usize {
        match self {
            Expr::Lit(v) => {
                match v {
                    Value::UInt(_n, width) => *width,
                    Value::SInt(_n, width) => *width,
                    Value::Vec(_vs) => todo!(),
                }
            },
            Expr::Var(_component_ref) => todo!(),
            Expr::Mux(_condition, e_true, e_false) => e_true.min_width().max(e_false.min_width()),
            Expr::Op(op, es) => op.min_width(es),
            // Expr::Read
        }
    }
}

impl Op {
    pub fn min_width(&self, _es: &[Expr]) -> usize {
        todo!()
    }
}

impl Component {
    pub fn prnicipal_type(&self) -> Type {
        match self {
            Component::Node(typ) => typ.clone(),
            Component::Wire(typ) => typ.clone(),
            Component::Reg(typ) => typ.clone(),
            Component::Port(typ, _dir) => typ.clone(),
            Component::Inst() => todo!(),
            //Component::InstPort(), // todo!()
        }
    }

    pub fn is_source(&self) -> bool {
        match self {
            Component::Node(_typ) => true,
            Component::Wire(_typ) => true,
            Component::Reg(_typ) => true,
            Component::Port(_typ, _dir) => true,
            Component::Inst() => todo!(),
            //Component::InstPort(), // todo!()
        }
    }

    pub fn is_sink(&self) -> bool {
        match self {
            Component::Node(_typ) => false,
            Component::Wire(_typ) => true,
            Component::Reg(_typ) => true,
            Component::Port(_typ, dir) => dir == &Direction::Output,
            Component::Inst() => todo!(),
            //Component::InstPort(), // todo!()
        }
    }

    pub fn is_duplex(&self) -> bool {
        self.is_source() && self.is_sink()
    }
}
impl From<&str> for RefPath {
    fn from(s: &str) -> RefPath {
        RefPath::Id(s.to_string())
    }
}
