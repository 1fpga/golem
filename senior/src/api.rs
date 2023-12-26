use rocket::{get, Route};
use rocket_okapi::{openapi, openapi_get_routes};

mod status_bits;

pub fn api_routes() -> Vec<Route> {
    status_bits::routes()
}
