use std::env;

use juniper::{graphql_object, EmptySubscription, GraphQLInputObject};
use warp::{hyper::Response, Filter};

struct Context {}
// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

// Cannot use GraphQLInputObject on these enums
// #[derive(GraphQLInputObject)]
pub enum DispenseCommand {
    DispenseWeight(i32),
    SwitchToManualMode,
}

// Each Option must have a value so manual mode has an ignored input
#[derive(GraphQLInputObject)]
struct DispenseCommandUnionWithIdInput {
    id: i32,
    dispense_weight: Option<i32>,
    switch_to_manual_mode: Option<i32>,
}
impl From<DispenseCommandUnionWithIdInput> for DispenseCommand {
    fn from(input: DispenseCommandUnionWithIdInput) -> Self {
        match (input.dispense_weight, input.switch_to_manual_mode) {
            (Some(weight), None) => DispenseCommand::DispenseWeight(weight),
            (None, Some(_)) => DispenseCommand::SwitchToManualMode,
            _ => panic!("Bad input"),
        }
    }
}

// Each Option must have a value, so we can pull ID into each child command
#[derive(GraphQLInputObject)]
struct DispenseWeightInput {
    id: i32,
    dispense_weight: i32,
}
#[derive(GraphQLInputObject)]
struct SwitchToManualModeInput {
    id: i32,
}

#[derive(GraphQLInputObject)]
struct DispenseCommandUnionNestedIdInput {
    dispense_weight: Option<DispenseWeightInput>,
    switch_to_manual_mode: Option<SwitchToManualModeInput>,
}
impl From<DispenseCommandUnionNestedIdInput> for DispenseCommand {
    fn from(input: DispenseCommandUnionNestedIdInput) -> Self {
        match (input.dispense_weight, input.switch_to_manual_mode) {
            (Some(weight_input), None) => {
                DispenseCommand::DispenseWeight(weight_input.dispense_weight)
            }
            (None, Some(_)) => DispenseCommand::SwitchToManualMode,
            _ => panic!("Bad input"),
        }
    }
}

// Break into own inputs
#[derive(GraphQLInputObject)]
struct DispenseWeightCommandInput {
    id: i32,
    weight: i32,
}
impl From<DispenseWeightCommandInput> for DispenseCommand {
    fn from(input: DispenseWeightCommandInput) -> Self {
        DispenseCommand::DispenseWeight(input.weight)
    }
}

#[derive(GraphQLInputObject)]
struct DispenseManualModeCommandInput {
    id: i32,
}
impl From<DispenseManualModeCommandInput> for DispenseCommand {
    fn from(_: DispenseManualModeCommandInput) -> Self {
        DispenseCommand::SwitchToManualMode
    }
}

struct Mutation;

#[graphql_object(context = Context)]
impl Mutation {
    // command with id at top level
    async fn dispense_command_union_with_id_input(
        _input: DispenseCommandUnionWithIdInput,
    ) -> String {
        // send command
        String::from("Success")
    }

    // command with id in each body
    async fn dispense_command_union_nested_id_input(
        _input: DispenseCommandUnionNestedIdInput,
    ) -> String {
        // send command
        String::from("Success")
    }

    // Break apart commands
    async fn dispense_weight_command(_input: DispenseWeightCommandInput) -> String {
        // send command
        String::from("Success")
    }

    async fn dispense_manual_mode_command(_input: DispenseManualModeCommandInput) -> String {
        // send command
        String::from("Success")
    }
}

struct Query;

#[graphql_object(context = Context)]
impl Query {
    async fn test() -> String {
        String::from("Success")
    }
}

type Schema = juniper::RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "warp_server");
    env_logger::init();

    let log = warp::log("warp_server");

    let homepage = warp::path::end().map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body(format!(
                "<html><h1>juniper_warp</h1><div>visit <a href=\"/graphiql\">/graphiql</a></html>"
            ))
    });

    log::info!("Listening on 127.0.0.1:8080");

    let state = warp::any().map(move || Context{});
    let graphql_filter = juniper_warp::make_graphql_filter(schema(), state.boxed());

    warp::serve(
        warp::get()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql", None))
            .or(homepage)
            .or(warp::path("graphql").and(graphql_filter))
            .with(log),
    )
    .run(([127, 0, 0, 1], 8080))
    .await
}
