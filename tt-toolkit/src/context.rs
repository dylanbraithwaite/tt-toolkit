use cons_list::ConsList;

use crate::DeBruijnIndexed;

pub trait Context {
    type Entry: DeBruijnIndexed;

    fn iter(&self) -> impl Iterator<Item = Self::Entry>;
    fn append(&self, variable: Self::Entry) -> Self;
    fn empty() -> Self;

    fn get(&self, var: usize) -> Option<Self::Entry> {
        self.iter()
            .nth(var)
            .map(|v| v.increment_indices_by(var + 1))
    }
}

pub struct ListContext<T>(ConsList<T>);

impl<Entry: DeBruijnIndexed + Clone> Context for ListContext<Entry> {
    type Entry = Entry;

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
