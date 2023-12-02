mod methods;
pub use methods::*;

pub type Id = String;
pub type ModId = String;

macro_rules! unused {
    ($name:ident) => {
        #[doc=stringify!(GRAMMAR $name)]
        #[allow(non_camel_case_types, dead_code)]
        struct $name;
    }
}

/// GRAMMAR: circuit
#[derive(Debug, Clone)]
pub struct Circuit {
    // version
    pub top: Id,
    // annotations,
    // info
    pub decls: Vec<Decl>,
}

/// GRAMMAR: decl
#[derive(Debug, Clone)]
pub enum Decl {
    Mod(ModDef),
//    ExtMod(ExtModDef), // TODO
//    IntMod(IntModDef), // TODO
//    Layer, // TODO
//    TypeAlias // TODO
}

/// GRAMMAR decl_module
#[derive(Debug, Clone)]
pub struct ModDef {
    pub is_public: bool,
    // info
    pub name: Id,
    pub ports: Vec<Port>,
    pub statements: Vec<Statement>,
}

// TODO
/// GRAMMAR decl_extmodule
// #[derive(Debug, Clone)]
// pub struct ExtModDef;

// TODO
/// GRAMMAR decl_intmodule
// #[derive(Debug, Clone)]
// pub struct IntModDef;

// TODO
/// GRAMMAR decl_layer
// #[derive(Debug, Clone)]
// pub struct Layer;

// TODO
/// GRAMMAR decl_type_alias
// #[derive(Debug, Clone)]
// pub struct TypeAlias;

/// GRAMMAR port
#[derive(Debug, Clone)]
pub struct Port {
    pub direction: Direction,
    pub name: Id,
    pub typ: Type, // or type property
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Input,
    Output,
}

unused!(type_param);
unused!(type_property);

/// GRAMMAR statement
#[derive(Debug, Clone)]
pub enum Statement {
    CircuitComponent(CircuitComponent),
    Connectlike(Connectlike),
//    Conditional(Conditional), // TODO
//    Command(Command), // TODO
//    LayerBlock(LayerBlock), // TODO
    Skip,
}

/// GRAMMAR circuit_component
#[derive(Debug, Clone)]
pub enum CircuitComponent {
    Node(Id, Box<Expr>),
    Wire(Id, Type),
    Reg(Id, Type), // todo!()
    Inst(Id, ModId),
    // Mem // TODO
}

unused!(circuit_component_node);
unused!(circuit_component_wire);
unused!(circuit_component_inst);
unused!(circuit_conditional_reg); // TODO: sic
unused!(circuit_component_mem);
unused!(read_under_write);

/// GRAMAMR connectlike
#[derive(Debug, Clone)]
pub enum Connectlike {
    Connect(Reference, Box<Expr>),
    Invalidate(Reference),
//    Attach(RefPath, RefPath), // TODO
//    Define(), // TODO
//    PropAssign(), // TODO
}

/// GRAMAMR conditional
#[derive(Debug, Clone)]
pub enum Conditional {
//    When(Box<Expr>, Vec<Statement>, Option<Vec<Statement>>), // TODO
//    Match // TODO
}

unused!(conditional_when);
unused!(conditional_match);
unused!(conditional_match_branch);

/// GRAMMAR command
#[derive(Debug, Clone)]
pub enum Command {
    // stop // TODO
    // force // TODO
    // force_initial // TODO
    // release // TODO
    // release_initial // TODO
    // printf // TODO
}

unused!(layerblock);
unused!(skip);

/// GRAMAMR expr
#[derive(Debug, Clone)]
pub enum Reference {
    Id(String),
    Dot(Box<Reference>, String),
    Index(Box<Reference>, usize),
    IndexDynamic(Box<Reference>, Box<Expr>),
}


unused!(reference_static);
unused!(reference_dynamic);

/// GRAMAMR expr
#[derive(Debug, Clone)]
pub enum Expr {
    Var(Reference),
    Lit(Lit),
    // enum
    Mux(Box<Expr>, Box<Expr>, Box<Expr>),
    // Read() // todo!()
    // PrimOp(Op, Vec<Expr>),
}

unused!(expr_reference);

/// GRAMMAR expr_lit
#[derive(Debug, Clone)]
pub enum Lit {
    UInt(Option<Width>, isize), // TODO
    SInt(Option<Width>, isize),
}

unused!(expr_enum);
unused!(expr_mux);
unused!(expr_read);
unused!(expr_probe);
unused!(property_literal_expr);
unused!(property_expr);
unused!(expr_primop);


/// GRAMMAR type
#[derive(Debug, Clone)]
pub enum Type {
    UInt(Option<u64>),
    SInt(Option<u64>), // TODO u64 or i64? Arbprec integer?
    Clock,
    Reset,
    AsyncReset,
//    Analog(Option<usize>), // todo!()
//    Const(Box<Type>),
//    Probe(Box<Type>),
//    RwProbe(Box<Type>),
    Vec(usize, Box<Type>),
    Bundle(Vec<BundleField>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Flippedness {
    Aligned,
    Flipped,
}

pub type Fieldname = String;


unused!(type_hardware);
unused!(type_ground);
unused!(type_ground_nowidth);
unused!(type_ground_width);

/// GRAMMAR width
pub type Width = usize;

unused!(type_bundle);

/// GRAMMAR type_bundle_field
#[derive(Debug, Clone)]
pub struct BundleField(pub Flippedness, pub Fieldname, pub Box<Type>);

unused!(type_vec);
unused!(type_enum);
unused!(type_enum_alt);
unused!(type_probe);
unused!(primop_2expr);
unused!(primop_1expr);
unused!(primop_1expr1int);
unused!(primop_1expr2int);
unused!(annotations);
