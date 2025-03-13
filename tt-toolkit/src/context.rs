use cons_list::ConsList;

use crate::DeBruijnIndexed;

pub trait Context<Entry> {
    fn iter(&self) -> impl Iterator<Item = Entry>;
    fn append(&self, variable: Entry) -> Self;
    fn empty() -> Self;

    fn get(&self, var: usize) -> Option<Entry>
    where Entry: Clone,
    {
        self.iter().nth(var)
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

impl<Entry> Context<Entry> for ListContext<Entry> where Entry: Clone {
    fn iter(&self) -> impl Iterator<Item = Entry> {
        self.0.iter().cloned()
    }

    fn append(&self, variable: Entry) -> Self {
        ListContext(self.0.append(variable))
    }

    fn empty() -> Self {
        ListContext(ConsList::new())
    }
}
