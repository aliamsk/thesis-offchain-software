/// This Connector obtains the nanoErg per 1 USD rate and submits it
/// to an oracle core. It reads the `oracle-config.yaml` to find the port
/// of the oracle core (via Connector-Lib) and submits it to the POST API
/// server on the core.
/// Note: The value that is posted on-chain is the number
/// of nanoErgs per 1 USD, not the rate per nanoErg.
use anyhow::{anyhow, Result};
use frontend_connector_lib::FrontendConnector;

static BTC_latestHash_URL: &str = "https://blockchain.info/q/latesthash";

/// Acquires the price of Ergs in USD from CoinGecko, convert it
/// into nanoErgs per 1 USD, and return it.
fn get_datapoint() -> Result<String> {
    let resp = reqwest::blocking::Client::new().get(BTC_latestHash_URL).send()?;
    //let price_json = json::parse(&resp.text()?)?;
    let blockHash = &resp.text()?;
    println!("Here is the datapoint after fetching from external url: {}", price);
    if let Some(p) = blockHash.as_str() {
        println!("Datapoint on connector datapoint provider function: {}", price);
        return Ok(price);
    } else {
        Err(anyhow!("Failed to get datapoint in connectors collect data function...."))
    }
}

fn main() {
    // Create the FrontendConnector
    let connector = FrontendConnector::new_basic_connector(
        "test-string",
        get_datapoint,
        generate_current_price,
    );

    // Start the FrontendConnector
    connector.run();
}
