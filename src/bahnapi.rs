use graphql_client::*;
use std::fs;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "bahn_definitions/schema.graphql",
    query_path = "bahn_definitions/search.graphql",
    response_derives = "Debug"
)]
pub struct Search;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "bahn_definitions/schema.graphql",
    query_path = "bahn_definitions/nearby.graphql",
    response_derives = "Debug"
)]
pub struct Nearby;

pub fn search(search_term: String) -> Result<(), failure::Error> {
    let q = Search::build_query(search::Variables {
        search_term: Some(search_term),
    });

    let client = reqwest::Client::new();

    let mut res = client
        .post("https://api.deutschebahn.com/free1bahnql/v1/graphql")
        .bearer_auth("6fc6827d4628e9568210eed4a22fa8b5")
        .json(&q)
        .send()?;
    let response_body: Response<search::ResponseData> = res.json()?;
    //println!("{:?}", response_body);
    fs::write("./search.json", format!("{:?}", response_body)).expect("Unable to write file");
    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    }
    Ok(())
}

pub fn nearby(
    lat: f64,
    long: f64,
    radius: Option<i64>,
    count: Option<i64>,
) -> Result<Response<nearby::ResponseData>, failure::Error> {
    let q = Nearby::build_query(nearby::Variables {
        latitude: lat,
        longitude: long,
        radius,
        count,
    });

    let client = reqwest::Client::new();

    let mut res = client
        .post("https://api.deutschebahn.com/free1bahnql/v1/graphql")
        .bearer_auth("6fc6827d4628e9568210eed4a22fa8b5")
        .json(&q)
        .send()?;
    let response_body: Response<nearby::ResponseData> = res.json()?;
    //println!("{:?}", response_body);
    fs::write("./nearby.json", format!("{:?}", response_body)).expect("Unable to write file");
    if let Some(errors) = &response_body.errors {
        println!("there are errors:");

        for error in errors {
            println!("{:?}", error);
        }
    }
    Ok(response_body)
}
