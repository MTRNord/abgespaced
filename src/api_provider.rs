use juniper::{EmptyMutation, Value};
use rocket::response::content;
use rocket::State;

#[derive(juniper::GraphQLObject)]
struct TrainMovement {
    movement_type: String,
    train_number: String,
    platform: String,
    time: String,
    stops: Vec<String>,
}

#[derive(juniper::GraphQLObject)]
struct Timetables {
    next_arrivals: Vec<TrainMovement>,
    next_depatures: Vec<TrainMovement>,
}

#[derive(juniper::GraphQLObject)]
struct Location {
    latitude: f64,
    longitude: f64,
}

#[derive(juniper::GraphQLObject)]
#[graphql(description = "A Station in the Bahn Universe")]
struct Station {
    name: String,
    primary_eva_id: Option<i32>,
    has_local_public_transport: bool,
    has_taxi_rank: bool,
    has_stepless_access: bool,
    has_wifi: bool,
    location: Option<Location>,
    timetables: Timetables,
}

#[derive(juniper::GraphQLObject)]
#[graphql(description = "A Space in the Space API")]
struct Space {
    #[graphql(description = "The human readable name of a *Space")]
    name: String,
    #[graphql(description = "All Bahn Stations in a radius of 2000m")]
    stations: Vec<Station>,
}

struct Query;

enum NotFoundErr {
    NotFound,
}

impl juniper::IntoFieldError for NotFoundErr {
    fn into_field_error(self) -> juniper::FieldError {
        match self {
            NotFoundErr::NotFound => {
                juniper::FieldError::new("The searched space does not exist", Value::null())
            }
        }
    }
}

#[juniper::object]
impl Query {
    fn apiVersion() -> &str {
        "0.1.0"
    }

    fn space(name: String) -> Result<Space, NotFoundErr> {
        let searched_space_coordinates = crate::spaceapi::find_space(&name);
        match searched_space_coordinates {
            Ok(searched_space_coordinates_unpacked) => {
                let stations = crate::bahnapi::nearby(
                    searched_space_coordinates_unpacked["lat"],
                    searched_space_coordinates_unpacked["lon"],
                    Some(2000),
                    Some(20),
                );
                let mut stations_usable = Vec::new();

                if let Ok(x) = stations {
                    let data = x.data.unwrap();
                    let nearby_stations = data.nearby.stations;
                    for station in nearby_stations.into_iter() {
                        let primary_eva_id: Option<i32>;
                        if station.primary_eva_id.is_some() {
                            let primary_eva_id_raw: i32 = station.primary_eva_id.unwrap() as i32;
                            primary_eva_id = Some(primary_eva_id_raw);
                        } else {
                            primary_eva_id = None;
                        }
                        let has_stepless_access: bool;
                        if station.has_stepless_access == "yes" {
                            has_stepless_access = true
                        } else {
                            has_stepless_access = false
                        }
                        let location: Option<Location>;
                        if station.location.is_some() {
                            let location_bahn = station.location.unwrap();
                            location = Some(Location {
                                latitude: location_bahn.latitude,
                                longitude: location_bahn.longitude,
                            });
                        } else {
                            location = None;
                        }
                        let station_usable = Station {
                            name: station.name,
                            primary_eva_id,
                            has_local_public_transport: station.has_local_public_transport,
                            has_taxi_rank: station.has_taxi_rank,
                            has_stepless_access,
                            has_wifi: station.has_wi_fi,
                            location,
                            timetables: Timetables {
                                next_arrivals: vec![],
                                next_depatures: vec![],
                            },
                        };
                        stations_usable.push(station_usable);
                    }
                }
                let space = Space {
                    name,
                    stations: stations_usable,
                };
                return Ok(space);
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
        Err(NotFoundErr::NotFound)
    }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
type Schema = juniper::RootNode<'static, Query, EmptyMutation<()>>;

#[rocket::get("/")]
fn graphiql() -> content::Html<String> {
    juniper_rocket::graphiql_source("/graphql")
}

#[rocket::get("/graphql?<request>")]
fn get_graphql_handler(
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, &())
}

#[rocket::post("/graphql", data = "<request>")]
fn post_graphql_handler(
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, &())
}

pub fn start_api() {
    rocket::ignite()
        .manage(Schema::new(Query, EmptyMutation::<()>::new()))
        .mount(
            "/",
            rocket::routes![graphiql, get_graphql_handler, post_graphql_handler],
        )
        .launch();
}
