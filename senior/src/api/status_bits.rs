use crate::guards::CoreGuard;
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

#[openapi(tag = "Status Bits", ignore = "core")]
#[get("/status_bits")]
async fn status_bits(core: CoreGuard) -> Result<Json<StatusBitsResponse>, String> {
    let (bits, mask) = {
        let mut c = core.lock().await;
        (*c.read_status_bits(), c.config().status_bit_map_mask())
    };

    Ok(Json(StatusBitsResponse { bits, mask }))
}

#[openapi(tag = "Status Bits", ignore = "core")]
#[get("/status_bits/set/<bit>")]
async fn status_bits_set(core: CoreGuard, bit: u8) -> Result<Json<StatusBitsResponse>, String> {
    let mut bits = core.lock().await.status_bits().clone();
    bits.set(bit as usize, true);
    core.lock().await.send_status_bits(bits);

    status_bits(core).await
}

#[openapi(tag = "Status Bits", ignore = "core")]
#[get("/status_bits/pulse/<bit>")]
async fn status_bits_pulse(core: CoreGuard, bit: u8) -> Result<Json<StatusBitsResponse>, String> {
    let mut bits = core.lock().await.status_bits().clone();
    bits.set(bit as usize, true);
    core.lock().await.send_status_bits(bits);
    bits.set(bit as usize, false);
    core.lock().await.send_status_bits(bits);

    status_bits(core).await
}

pub(crate) fn routes() -> Vec<Route> {
    openapi_get_routes![status_bits, status_bits_set, status_bits_pulse]
}
