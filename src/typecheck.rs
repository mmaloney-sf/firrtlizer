use super::*;
/*

#[derive(Debug, Clone)]
pub enum TypeError {
    Error,
}

#[derive(Debug, Clone)]
pub struct Context<T>(Vec<(String, T)>);

impl<T> Context<T> {
    pub fn empty() -> Context<T> {
        Context(vec![])
    }

    pub fn extend(mut self, x: String, t: T) -> Context<T> {
        self.0.push((x, t));
        self
    }
}

impl Context<Type> {
    pub fn check_type(&self, expr: &Expr, typ: &Type) -> Result<(), TypeError> {
        match expr {
            Expr::Lit(v) => self.check_type_value(v, typ),
            Expr::Var(_component_ref) => todo!(),
            Expr::Mux(condition, e_true, e_false) => {
                self.check_type(condition, &Type::UInt(Some(1)))?;
                self.check_type(e_true, typ)?;
                self.check_type(e_false, typ)?;
                Ok(())
            },
            Expr::Op(_op, _es) => todo!(),
            // Expr::Read
        }
    }

    pub fn check_type_value(&self, value: &Value, typ: &Type) -> Result<(), TypeError> {
        match value {
            Value::UInt(_n, width) => {
                if let Type::UInt(Some(typ_width)) = typ {
                    if width <= typ_width {
                        Ok(())
                    } else {
                        Err(TypeError::Error)
                    }
                } else {
                    Err(TypeError::Error)
                }
            },
            Value::SInt(_n, width) => {
                if let Type::UInt(Some(typ_width)) = typ {
                    if width <= typ_width {
                        Ok(())
                    } else {
                        Err(TypeError::Error)
                    }
                } else {
                    Err(TypeError::Error)
                }
            },
            Value::Vec(vs) => {
                if let Type::Vec(n, vec_typ) = typ {
                    if vs.len() == *n {
                        for v in vs {
                            self.check_type_value(v, vec_typ)?;
                        }
                        Ok(())
                    } else {
                        Err(TypeError::Error)
                    }
                } else {
                    Err(TypeError::Error)
                }
            },
        }
    }

    pub fn infer_type(&self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Lit(_v) => todo!(),
            Expr::Var(_component_ref) => todo!(),
            Expr::Mux(_condition, _e_true, _e_false) => todo!(),
            Expr::Op(_op, _es) => todo!(),
            // Expr::Read
        }
    }
}
*/
