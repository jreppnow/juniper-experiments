use std::error::Error;

use juniper::{
    execute_sync, futures::stream::Empty, graphql_object, EmptyMutation, EmptySubscription,
    FieldResult, GraphQLEnum, GraphQLObject, RootNode, Variables, ID,
};

#[derive(GraphQLObject)]
struct Skill {
    id: ID,
    description: String,
}

#[derive(GraphQLObject)]
struct Employee {
    id: ID,
    first_name: Option<String>,
    last_name: String,
    skills: Vec<Skill>,
}

#[derive(GraphQLEnum)]
enum State {
    Planning,
    Ongoing,
    Finished,
}

#[derive(GraphQLObject)]
struct Project {
    id: ID,
    short: Option<String>,
    description: Option<String>,
    state: State,
    skills_required: Vec<Skill>,
    responsible: Employee,
    assigned: Vec<Employee>,
}

struct InternalEmployee {
    first_name: Option<String>,
    last_name: String,
    skills: Vec<usize>,
}

struct InternalProject {
    short: Option<String>,
    description: Option<String>,
    state: State,
    skills_required: Vec<usize>,
    responsible: Employee,
    assigned: Vec<usize>,
}

struct Context {
    skills: Vec<String>,
    employees: Vec<InternalEmployee>,
    projects: Vec<InternalProject>,
}

impl juniper::Context for Context {}

struct Queries;

#[graphql_object(Context = Context)]
impl Queries {
    fn skills(context: &Context) -> Vec<Skill> {
        context
            .skills
            .clone()
            .into_iter()
            .enumerate()
            .map(|(index, description)| Skill {
                id: index.to_string().into(),
                description,
            })
            .collect()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let context = Context {
        skills: vec!["Dancing".to_owned(), "Singing".to_owned()],
        employees: vec![],
        projects: vec![],
    };

    let schema: &'static _ = Box::leak(Box::new(RootNode::new(
        Queries,
        EmptySubscription::default(),
        EmptyMutation::default(),
    )));
    println!(
        "{:?}",
        execute_sync(
            "query { skills { id, description } }",
            None,
            &schema,
            &Variables::new(),
            &context,
        )?,
    );
    Ok(())
}
