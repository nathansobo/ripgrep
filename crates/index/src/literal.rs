use std::mem;

use bstr::{BString, ByteVec};
use regex_syntax::hir::{self, Hir, HirKind};

#[derive(Clone, Debug)]
pub enum Literals {
    Exact(LiteralSet),
    Inexact(Inexact),
}

#[derive(Clone, Debug)]
struct Inexact {
    prefix: LiteralSet,
    inner: LiteralSet,
    suffix: LiteralSet,
    empty: bool,
}

impl Literals {
    fn empty() -> Literals {
        Literals::inexact(true)
    }

    fn inexact(empty: bool) -> Literals {
        Literals::Inexact(Inexact {
            prefix: LiteralSet::new(),
            inner: LiteralSet::new(),
            suffix: LiteralSet::new(),
            empty,
        })
    }

    fn make_inexact(&mut self) -> &mut Inexact {
        let exact = match *self {
            Literals::Inexact(ref mut inex) => return inex,
            Literals::Exact(ref mut exact) => {
                mem::replace(exact, LiteralSet::new())
            }
        };
        *self = Literals::Inexact(Inexact {
            prefix: exact.clone(),
            inner: exact.clone(),
            suffix: exact,
            empty: false,
        });
        match *self {
            Literals::Inexact(ref mut inex) => inex,
            _ => unreachable!(),
        }
    }

    fn union(&mut self, o: Literals) {
        match o {
            Literals::Exact(set2) => match *self {
                Literals::Exact(ref mut set1) => {
                    set1.union(set2);
                }
                Literals::Inexact(ref mut inex1) => {
                    inex1.prefix.union(set2.clone());
                    inex1.inner.union(set2.clone());
                    inex1.suffix.union(set2.clone());
                }
            },
            Literals::Inexact(inex2) => {
                let inex1 = self.make_inexact();
                inex1.prefix.union(inex2.prefix);
                inex1.inner.union(inex2.inner);
                inex1.suffix.union(inex2.suffix);
            }
        }
    }
}

#[derive(Clone, Debug)]
struct LiteralSet {
    lits: Vec<BString>,
}

impl LiteralSet {
    fn new() -> LiteralSet {
        LiteralSet { lits: vec![] }
    }

    fn single(lit: BString) -> LiteralSet {
        LiteralSet { lits: vec![lit] }
    }

    fn union(&mut self, o: LiteralSet) {
        self.lits.extend(o.lits)
    }
}

#[derive(Clone, Debug)]
pub struct LiteralsBuilder {
    limit_len: usize,
    limit_class: usize,
}

impl LiteralsBuilder {
    pub fn new() -> LiteralsBuilder {
        LiteralsBuilder { limit_len: 250, limit_class: 10 }
    }

    pub fn limit_len(&mut self, len: usize) -> &mut LiteralsBuilder {
        self.limit_len = len;
        self
    }

    pub fn limit_class(&mut self, len: usize) -> &mut LiteralsBuilder {
        self.limit_class = len;
        self
    }

    // pub fn build(&self, exp: &Hir) -> Literals {
    // let mut lits = Literals::none();
    // self.build_into(exp, &mut lits);
    // lits
    // }
    //
    // pub fn build_into(&self, exp: &Hir, set: &mut Literals) {
    // match exp.kind() {
    // HirKind::Empty => *set = Literals::empty(),
    // HirKind::Literal(hir::Literal::Unicode(ch)) => todo!(),
    // HirKind::Literal(hir::Literal::Byte(b)) => todo!(),
    // _ => todo!(),
    // }
    // }

    pub fn build(&self, exp: &Hir) -> Literals {
        match *exp.kind() {
            HirKind::Empty => Literals::empty(),
            HirKind::Literal(hir::Literal::Unicode(ch)) => {
                let mut lit = BString::from(vec![]);
                lit.push_char(ch);
                Literals::Exact(LiteralSet::single(lit))
            }
            HirKind::Literal(hir::Literal::Byte(b)) => {
                let mut lit = BString::from(vec![]);
                lit.push_byte(b);
                Literals::Exact(LiteralSet::single(lit))
            }
            HirKind::Alternation(ref exps) => {
                if exps.is_empty() {
                    Literals::empty()
                } else {
                    let mut set = self.build(&exps[0]);
                    for e in exps.iter().skip(1) {
                        set.union(self.build(e));
                    }
                    set
                }
            }
            _ => todo!(),
        }
    }
}

impl Default for LiteralsBuilder {
    fn default() -> LiteralsBuilder {
        LiteralsBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex_syntax::Parser;

    fn parse(pattern: &str) -> Hir {
        Parser::new().parse(pattern).unwrap()
    }

    #[test]
    fn scratch() {
        let re = parse("a|b|c");

        let mut b = LiteralsBuilder::new();
        let lits = b.build(&re);
        println!("{:?}", lits);
    }
}
