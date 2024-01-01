use rocket::Route;

mod status_bits;

pub fn api_routes() -> Vec<Route> {
    status_bits::routes()
}
