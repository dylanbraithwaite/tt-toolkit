use cons_list::ConsList;

use crate::DeBruijnIndexed;

pub trait Context<Entry> {
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a Entry> where Entry: 'a;
    fn append(&self, variable: Entry) -> Self;
    fn empty() -> Self;

    fn get(&self, var: usize) -> Option<Entry>
    where Entry: Clone,
    {
        self.iter().nth(var).cloned()
    }

    fn get_shifted(&self, var: usize) -> Option<Entry> 
    where Entry: DeBruijnIndexed
    {
        self.iter().nth(var).map(|expr| expr.increment_indices_by(var + 1))
    }
}

pub trait PartialContext<Entry>: Context<Option<Entry>> {}

impl<Entry, Ctx> PartialContext<Entry> for Ctx where Ctx: Context<Option<Entry>> {}

pub struct ListContext<T>(ConsList<T>);

impl<Entry> Context<Entry> for ListContext<Entry> {
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a Entry> where Entry: 'a {
        self.0.iter()
    }

    fn append(&self, variable: Entry) -> Self {
        ListContext(self.0.append(variable))
    }

    fn empty() -> Self {
        ListContext(ConsList::new())
    }
}
