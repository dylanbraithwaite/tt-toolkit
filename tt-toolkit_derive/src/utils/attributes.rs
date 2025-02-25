use syn::{parse::Parse, Attribute};


pub trait HasAttributes {
    fn find_all_attributes(&self, name: &str) -> Vec<Attribute>;

    fn find_attribute(&self, name: &str) -> Option<Attribute> {
        self.find_all_attributes(name).into_iter().next()
    }

    fn has_attribute(&self, name: &str) -> bool {
        self.find_attribute(name).is_some()
    }

    fn attribute_position(&self, name: &str) -> Option<usize>;

    fn parse_attribute<T>(&self, name: &str) -> Option<T>
    where
        T: Parse,
    {
        self.find_attribute(name)
            .map(|attr| attr.parse_args().unwrap())
    }
}

impl HasAttributes for [Attribute] {
    fn find_all_attributes(&self, name: &str) -> Vec<Attribute> {
        self.iter()
            .filter(|attr| attr.path().is_ident(name))
            .cloned()
            .collect()
    }

    fn attribute_position(&self, name: &str) -> Option<usize> {
        self.iter().position(|attr| attr.path().is_ident(name))
    }
}

impl HasAttributes for syn::Field {
    fn find_all_attributes(&self, name: &str) -> Vec<Attribute> {
        self.attrs.find_all_attributes(name)
    }

    fn attribute_position(&self, name: &str) -> Option<usize> {
        self.attrs.attribute_position(name)
    }
}

impl HasAttributes for synstructure::VariantInfo<'_> {
    fn find_all_attributes(&self, name: &str) -> Vec<Attribute> {
        self.ast().attrs.find_all_attributes(name)
    }

    fn attribute_position(&self, name: &str) -> Option<usize> {
        self.ast().attrs.attribute_position(name)
    }
}

impl HasAttributes for synstructure::BindingInfo<'_> {
    fn find_all_attributes(&self, name: &str) -> Vec<Attribute> {
        self.ast().find_all_attributes(name)
    }

    fn attribute_position(&self, name: &str) -> Option<usize> {
        self.ast().attribute_position(name)
    }
}

impl HasAttributes for synstructure::Structure<'_> {
    fn find_all_attributes(&self, name: &str) -> Vec<Attribute> {
        self.ast().attrs.find_all_attributes(name)
    }

    fn attribute_position(&self, name: &str) -> Option<usize> {
        self.ast().attrs.attribute_position(name)
    }
}