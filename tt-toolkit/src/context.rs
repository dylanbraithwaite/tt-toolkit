use cons_list::ConsList;

use crate::DeBruijnIndexed;

pub trait Context<Entry: DeBruijnIndexed> {
    fn iter(&self) -> impl Iterator<Item = Entry>;
    fn append(&self, variable: Entry) -> Self;
    fn empty() -> Self;

    fn get(&self, var: usize) -> Option<Entry> {
        self.iter()
            .nth(var)
            .map(|v| v.increment_indices_by(var + 1))
    }
}

pub trait PartialContext<Entry: DeBruijnIndexed> : Context<Option<Entry>> { }

impl<Entry: DeBruijnIndexed, Ctx> PartialContext<Entry> for Ctx where Ctx: Context<Option<Entry>> {}


pub struct ListContext<T>(ConsList<T>);

impl<Entry: DeBruijnIndexed + Clone> Context<Entry> for ListContext<Entry> {
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
