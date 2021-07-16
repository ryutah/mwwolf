use async_graphql::{EmptySubscription, Schema};

#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate async_graphql;

mod project;

#[derive(MergedObject, Default)]
pub struct Query(project::ProjectQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(project::ProjectMutaion);

pub type KzSchema = Schema<Query, Mutation, EmptySubscription>;
