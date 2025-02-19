use ttt::{ListContext, SynthAttribute};
use ttt_derive::attr_dsl;

#[derive(Clone)]
enum Ty {
    Prod(Box<Ty>, Box<Ty>),
    Func(Box<Ty>, Box<Ty>),
    Unit,
}

#[derive(Clone)]
enum Expr {
    Unit,
    Lam(Box<Ty>, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Pair(Box<Expr>, Box<Expr>),
}

impl SynthAttribute<Ty> for Expr {
    type Error = ();
    type Entry = Ty;

    type Ctx = ListContext<Ty>;

    fn synth(&self, ctx: &Self::Ctx) -> Result<Ty, Self::Error> {
        match self {
            Expr::Unit => Ok(Ty::Unit),
            Expr::Pair(_expr, _expr1) => todo!(),
            Expr::Lam(ty, expr) => attr_dsl! {
                context_type = ListContext<Ty>;
                context = ctx;

                let tgt = bind (**ty).clone() {
                    synth(Ty, *expr.clone())
                };

                Ok(Ty::Func(ty.clone().into(), tgt.into()))
            },
            Expr::App(_expr, _expr1) => todo!(),
        }
    }
}
