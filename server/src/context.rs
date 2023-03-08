use crate::data::resolver::DataResolver;

pub struct Context {
    data_resolver: DataResolver,
}

impl Context {
    pub fn new(data_resolver: DataResolver) -> Self {
        Self {
            data_resolver,
        }
    }

    pub fn data_resolver(&self) -> &DataResolver {
        &self.data_resolver
    }
}