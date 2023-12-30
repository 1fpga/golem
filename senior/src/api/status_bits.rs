use crate::utils::get_core;
use mister_fpga::types::StatusBitMap;
use rocket::serde::json::Json;
use rocket::{get, Route};
use rocket_okapi::{openapi, openapi_get_routes};
use schemars::gen::SchemaGenerator;
use schemars::schema::Schema;
use schemars::{schema_for_value, JsonSchema};

#[derive(Debug, serde::Serialize)]
struct StatusBitsResponse {
    bits: StatusBitMap,
    mask: StatusBitMap,
}

impl JsonSchema for StatusBitsResponse {
    fn schema_name() -> String {
        "StatusBitsResponse".to_string()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        Schema::Object(
            schema_for_value!(StatusBitsResponse {
                bits: StatusBitMap::default(),
                mask: StatusBitMap::default(),
            })
            .schema,
        )
    }
}

#[openapi(tag = "Status Bits")]
#[get("/status_bits")]
async fn status_bits() -> Result<Json<StatusBitsResponse>, String> {
    let mut core = get_core().await?;
    let bits = *core.read_status_bits();
    let mask = core.config().status_bit_map_mask();

    Ok(Json(StatusBitsResponse { bits, mask }))
}

#[openapi(tag = "Status Bits")]
#[get("/status_bits/set/<bit>")]
async fn status_bits_set(bit: u8) -> Result<Json<StatusBitsResponse>, String> {
    let mut core = get_core().await?;
    let mut bits = core.status_bits().clone();
    bits.set(bit as usize, true);
    core.send_status_bits(bits);

    crate::api::status_bits()
}

#[openapi(tag = "Status Bits")]
#[get("/status_bits/pulse/<bit>")]
async fn status_bits_pulse(bit: u8) -> Result<Json<StatusBitsResponse>, String> {
    let mut core = get_core().await?;
    let mut bits = core.status_bits().clone();
    bits.set(bit as usize, true);
    core.send_status_bits(bits);
    bits.set(bit as usize, false);
    core.send_status_bits(bits);

    crate::api::status_bits()
}

pub(crate) fn routes() -> Vec<Route> {
    openapi_get_routes![status_bits, status_bits_set, status_bits_pulse]
}
