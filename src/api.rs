use log::info;
use caramel::ns::api::Client;
use serde::Deserialize;
use serenity::all::Timestamp;

#[derive(Deserialize)]
pub struct NationData {
    #[serde(rename = "FOUNDEDTIME")]
    pub foundedtime: i64,
    #[serde(rename = "REGION")]
    pub region: String,
}

pub fn parse_nation_data(xml: &str) -> Result<NationData, crate::bot::Error> {
    Ok(quick_xml::de::from_str::<NationData>(xml)?)
}

pub async fn query_nation_data(
    client: &Client, nation: &str
) -> Result<NationData, crate::bot::Error> {
    let response = client.make_request_with_retry(vec![
        ("nation", nation), ("q", "foundedtime+region")
    ]).await?;

    let data = parse_nation_data(&response)?;
    info!("Queried data for {}: foundedtime={}, region={}", nation, data.foundedtime, data.region);

    return Ok(data);
}

const MIN_COOLDOWN_AGE: i64 = 47174400; // 18 months
const MAX_DELAY: i64 = 14; // Youngest nations get 14 seconds per nation, 112 seconds for am 8-nation batch
const MIN_DELAY: i64 = 5; // Older nations get 5 seconds per nation, 40 seconds for an 8-nation batch

// Calculates the per-recipient telegram delay from a nation according to when it was founded.
// Maps time since founding between newly founded and 18 months old linearly to a per-recipient 
// delay between 14 and 5 seconds, with nations older than 18 months capped at 5 seconds.
pub fn calculate_telegram_delay(foundedtime: i64) -> i64 {
    let seconds = (Timestamp::now().timestamp() - foundedtime).clamp(0, MIN_COOLDOWN_AGE);

    let range = MAX_DELAY - MIN_DELAY;

    MAX_DELAY - ((seconds * range) / MIN_COOLDOWN_AGE)
}