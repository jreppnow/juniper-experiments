use std::error::Error;

use juniper::{
    execute_sync, graphql_object, EmptyMutation, EmptySubscription, GraphQLEnum, GraphQLObject,
    RootNode, Variables, ID,
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

#[derive(GraphQLEnum, Clone, Copy)]
enum State {
    Planning,
    Ongoing,
    Finished,
}

#[derive(GraphQLObject)]
struct Project {
    id: ID,
    short: String,
    description: Option<String>,
    state: State,
    skills_required: Vec<Skill>,
    responsible: Employee,
    assigned: Vec<Employee>,
}

#[derive(Clone)]
struct InternalEmployee {
    first_name: Option<String>,
    last_name: String,
    skills: Vec<usize>,
}

struct InternalProject {
    short: String,
    description: Option<String>,
    state: State,
    skills_required: Vec<usize>,
    responsible: usize,
    assigned: Vec<usize>,
}

struct Context {
    skills: Vec<String>,
    employees: Vec<InternalEmployee>,
    projects: Vec<InternalProject>,
}

impl Context {
    fn resolve_skill(&self, id: usize) -> Option<Skill> {
        Some(Skill {
            id: id.to_string().into(),
            description: self.skills.get(id)?.clone(),
        })
    }

    fn resolve_employee(&self, id: usize) -> Option<Employee> {
        let InternalEmployee {
            first_name,
            last_name,
            skills,
        } = self.employees.get(id)?.clone();

        Some(Employee {
            id: id.to_string().into(),
            first_name,
            last_name,
            skills: skills
                .iter()
                .map(|skill_id| {
                    self.resolve_skill(*skill_id)
                        .expect("We take care that we keep all IDs valid!")
                })
                .collect(),
        })
    }
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

    fn employees(context: &Context) -> Vec<Employee> {
        context
            .employees
            .iter()
            .enumerate()
            .map(|(index, employee)| Employee {
                id: index.to_string().into(),
                first_name: employee.first_name.clone(),
                last_name: employee.last_name.clone(),
                skills: employee
                    .skills
                    .iter()
                    .map(|skill_id| {
                        let skill = &context.skills[*skill_id];
                        Skill {
                            id: skill_id.to_string().into(),
                            description: skill.clone(),
                        }
                    })
                    .collect(),
            })
            .collect()
    }

    fn projects(context: &Context) -> Vec<Project> {
        context
            .projects
            .iter()
            .enumerate()
            .map(|(index, project)| Project {
                id: index.to_string().into(),
                short: project.short.clone(),
                description: project.description.clone(),
                state: project.state,
                skills_required: project
                    .skills_required
                    .iter()
                    .map(|skill_id| {
                        let skill = &context.skills[*skill_id];
                        Skill {
                            id: skill_id.to_string().into(),
                            description: skill.clone(),
                        }
                    })
                    .collect(),
                responsible: context
                    .resolve_employee(project.responsible)
                    .expect("We take care to only give out valid indices!"),
                assigned: project
                    .assigned
                    .iter()
                    .map(|employee_id| {
                        context
                            .resolve_employee(*employee_id)
                            .expect("We take to only give out valid indices!")
                    })
                    .collect(),
            })
            .collect()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let context = Context {
        skills: vec!["Dancing".to_owned(), "Singing".to_owned()],
        employees: vec![InternalEmployee {
            first_name: Some("Michael".to_owned()),
            last_name: "Jackson".to_owned(),
            skills: vec![0, 1],
        }],
        projects: vec![InternalProject {
            short: "One of the greatest pop songs of all time!".to_owned(),
            description: None,
            state: State::Finished,
            skills_required: vec![0, 1],
            responsible: 0,
            assigned: vec![],
        }],
    };

    let schema: &'static _ = Box::leak(Box::new(RootNode::new(
        Queries,
        EmptySubscription::default(),
        EmptyMutation::default(),
    )));
    println!(
        "{:?}",
        execute_sync(
            "query { projects { id, description, responsible { lastName, skills { description } }, assigned { lastName }, short } }",
            None,
            schema,
            &Variables::new(),
            &context,
        )?,
    );
    Ok(())
}
