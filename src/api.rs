use crate::node_interface::current_block_height;
use crate::oracle_config::{get_core_api_port, get_node_url, PoolParameters};
use crate::oracle_state::{OraclePool, PoolBoxState};
use crate::print_action_results;
use anyhow::anyhow;
use crossbeam::Receiver;
use json;
use sincere;
use std::env;
use std::str::from_utf8;

/// Starts the POST API server which can be made publicly available without security risk
pub fn start_post_api() {
    let mut app = sincere::App::new();
    let args: Vec<String> = env::args().collect();

    // Accept a datapoint to be posted within a "Commit Datapoint" action tx
    app.post("/submitDatapoint", move |context| {
        let op = OraclePool::new();
        let res_post_json = from_utf8(context.request.body()).map(|t| json::parse(t));

        // Check if oracle core is in `read only` mode
        if args.len() > 1 && &args[1] == "--readonly" {
            let error_json = object! {error: "Oracle Core is in `read only` mode."}.to_string();

            context
                .response
                .header(("Access-Control-Allow-Origin", "*"))
                .from_json(error_json).unwrap();
        }

        // If the post request body is valid json
        if let Ok(Ok(post_json)) = res_post_json {
            // If the datapoint provided is a valid Integer
            if let Ok(datapoint) = post_json["datapoint"].to_string().parse() {
                // Check if in Live Epoch stage
                if let PoolBoxState::LiveEpoch = op.check_oracle_pool_stage() {
                    if let Ok(epoch_state) = op.get_live_epoch_state() {
                        let old_datapoint = epoch_state.latest_pool_datapoint;
                        println!("Oracle old datapoint - (api.rs file ...) : {}.", old_datapoint);
                        /*// 2% difference checks
                        let difference = datapoint as f64/old_datapoint as f64;
                        let mut action_result = Err(anyhow!("No datapoint has been submit."));


                        // If the new datapoint is 100% higher, post the new datapoint
                        if difference > 2.00 {
                            action_result = op.action_commit_datapoint(datapoint);
                        }
                        // If the new datapoint is 100% lower, post the new datapoint
                        else if difference < 0.50 {
                            action_result = op.action_commit_datapoint(datapoint);
                        }

                        // If the new datapoint is 0.49% to 20% lower, post 0.49% lower than old
                        else if difference < 0.9951 {
                            let new_datapoint = (old_datapoint as f64 * 0.9951) as u64;
                            action_result = op.action_commit_datapoint(new_datapoint);
                        }
                        // If the new datapoint is 0.49% to 20% higher, post 0.49% higher than old
                        else if difference > 1.0049 {
                            let new_datapoint = (old_datapoint as f64 * 1.0049) as u64;
                            action_result = op.action_commit_datapoint(new_datapoint);
                        }
                        // Else if the difference is within 4% either way, post the new datapoint
                        else {
                            action_result = op.action_commit_datapoint(datapoint);
                        }
*/
                        action_result = op.action_commit_datapoint(datapoint);
                        let action_name = "Submit Datapoint";
                        print_action_results(&action_result, action_name);
                        // If transaction succeeded being posted
                        if let Ok(res) = action_result{
                            let tx_id: String = res.chars().filter(|&c| c != '\"').collect();
                            let resp_json = object! {tx_id: tx_id}.to_string();

                            context
                                .response
                                .header(("Access-Control-Allow-Origin", "*")).from_json(resp_json).unwrap();
                        }
                        // If transaction failed being posted
                        else {
                            let error_json = object! {error: "Failed to post 'Commit Datapoint' action transaction."}.to_string();

                            context
                                .response
                                .header(("Access-Control-Allow-Origin", "*")).from_json(error_json).unwrap();
                        }
                    }
                    // Else if in Epoch Prep stage
                    else {
                        let error_json = object! {error: "Unable to submit Datapoint. The Oracle Pool is currently in the Epoch Preparation Stage."}.to_string();

                        context
                           .response
                           .header(("Access-Control-Allow-Origin", "*")).from_json(error_json).unwrap();
                    }
                }
                // If the datapoint provided is not a valid i32 Integer
                else {
                    let error_json = object! {error: "Invalid Datapoint Provided. Please ensure that your request includes a valid Integer i32 'datapoint' field."}.to_string();

                    context
                        .response
                        .header(("Access-Control-Allow-Origin", "*")).from_json(error_json).unwrap();
                }
            }

        }
        // If the post request body is not valid json
        else {
            let error_json = object! {error: "Invalid JSON Request Body."}.to_string();

            context
                .response
                .header(("Access-Control-Allow-Origin", "*")).from_json(error_json).unwrap();
        }
    });

    // Start the POST API server with the port designated in the config + 1.
    let port = ((get_core_api_port()
        .parse::<u16>()
        .expect("Failed to parse oracle core port from config to u16."))
        + 1)
    .to_string();
    let address = "0.0.0.0:".to_string() + &port;
    app.run(&address, 1).ok();
}

/// Starts the GET API server which can be made publicly available without security risk
pub fn start_get_api(repost_receiver: Receiver<bool>) {
    let mut app = sincere::App::new();

    // Basic welcome endpoint
    app.get("/", move |context| {
        let response_text = format!(
            "This is an Oracle Core. Please use one of the endpoints to interact with it.\n"
        );
        context
            .response
            .header(("Access-Control-Allow-Origin", "*"))
            .from_text(response_text)
            .unwrap();
    });

    // Basic oracle information
    app.get("/oracleInfo", move |context| {
        let op = OraclePool::new();
        let response_json = object! {
            oracle_address: op.local_oracle_address,
        };

        context
            .response
            .header(("Access-Control-Allow-Origin", "*"))
            .from_json(response_json.dump())
            .unwrap();
    });

    // Basic information about the oracle pool
    app.get("/poolInfo", move |context| {
        let op = OraclePool::new();
        let parameters = PoolParameters::new();

        let num_of_oracles = match op.datapoint_stage.number_of_boxes() {
            Ok(n) => n,
            Err(_) => 10,
        };

        let response_json = object! {
            number_of_oracles: num_of_oracles,
            live_epoch_address: op.live_epoch_stage.contract_address,
            epoch_prep_address: op.epoch_preparation_stage.contract_address,
            pool_deposits_address: op.pool_deposit_stage.contract_address,
            datapoint_address: op.datapoint_stage.contract_address,
            oracle_payout_price: parameters.oracle_payout_price,
            live_epoch_length: parameters.live_epoch_length,
            epoch_prep_length: parameters.epoch_preparation_length,
            deviation_range: parameters.deviation_range,
            consensus_num: parameters.consensus_num,
            minimum_pool_box_value: parameters.minimum_pool_box_value,
            oracle_pool_nft_id: op.oracle_pool_nft,
            oracle_pool_participant_token_id: op.oracle_pool_participant_token,

        };

        context
            .response
            .header(("Access-Control-Allow-Origin", "*"))
            .from_json(response_json.dump())
            .unwrap();
    });

    // Basic information about node the oracle core is using
    app.get("/nodeInfo", move |context| {
        let response_json = object! {
            node_url: get_node_url(),
        };

        context
            .response
            .header(("Access-Control-Allow-Origin", "*"))
            .from_json(response_json.dump())
            .unwrap();
    });

    // Status of the oracle
    app.get("/oracleStatus", move |context| {
        let op = OraclePool::new();

        // Check whether waiting for datapoint to be submit to oracle core
        let waiting_for_submit = match op.get_live_epoch_state() {
            Ok(l) => !l.commit_datapoint_in_epoch,
            Err(_) => false,
        };
        // Get latest datapoint the local oracle produced/submit
        let self_datapoint = match op.get_datapoint_state() {
            Ok(d) => d.datapoint,
            Err(_) => 0,
        };
        // Get latest datapoint submit epoch
        let datapoint_epoch = match op.get_datapoint_state() {
            Ok(d) => d.origin_epoch_id,
            Err(_) => "Null".to_string(),
        };
        // Get latest datapoint submit epoch
        let datapoint_creation = match op.get_datapoint_state() {
            Ok(d) => d.creation_height,
            Err(_) => 0,
        };

        let response_json = object! {
            waiting_for_datapoint_submit: waiting_for_submit,
            latest_datapoint: self_datapoint,
            latest_datapoint_epoch: datapoint_epoch,
            latest_datapoint_creation_height: datapoint_creation,
        };

        context
            .response
            .header(("Access-Control-Allow-Origin", "*"))
            .from_json(response_json.dump())
            .unwrap();
    });

    // Status of the oracle pool
    app.get("/poolStatus", move |context| {
        let op = OraclePool::new();
        let parameters = PoolParameters::new();

        // Current stage of the oracle pool box
        let current_stage = match op.check_oracle_pool_stage() {
            PoolBoxState::LiveEpoch => "Live Epoch",
            PoolBoxState::Preparation => "Epoch Preparation",
        };

        let mut funded_percentage = 0;
        let mut latest_datapoint = 0;
        let mut current_epoch_id = "".to_string();
        let mut epoch_ends = 0;
        if let Ok(l) = op.get_live_epoch_state() {
            // The percentage that the pool is funded
            funded_percentage = (l.funds / parameters.minimum_pool_box_value) * 100;
            latest_datapoint = l.latest_pool_datapoint;
            current_epoch_id = l.epoch_id;
            epoch_ends = l.epoch_ends;
        } else if let Ok(ep) = op.get_preparation_state() {
            // The percentage that the pool is funded
            funded_percentage = (ep.funds / parameters.minimum_pool_box_value) * 100;
            latest_datapoint = ep.latest_pool_datapoint;
            current_epoch_id = "Preparing Epoch Currently".to_string();
            epoch_ends = ep.next_epoch_ends;
        }

        let response_json = object! {
            funded_percentage: funded_percentage,
            current_pool_stage: current_stage,
            latest_datapoint: latest_datapoint,
            current_epoch_id : current_epoch_id,
            epoch_ends: epoch_ends,
        };

        context
            .response
            .header(("Access-Control-Allow-Origin", "*"))
            .from_json(response_json.dump())
            .unwrap();
    });

    // Block height of the Ergo blockchain
    app.get("/blockHeight", move |context| {
        let current_height =
            current_block_height().expect("Please ensure that the Ergo node is running.");
        let response_text = format!("{}", current_height);
        context
            .response
            .header(("Access-Control-Allow-Origin", "*"))
            .from_text(response_text)
            .unwrap();
    });

    // Whether the Core requires the Connector to repost a new Datapoint
    app.get("/requireDatapointRepost", move |context| {
        let mut response_text = format!("false");
        if let Ok(b) = repost_receiver.try_recv() {
            response_text = b.to_string();
        }
        context
            .response
            .header(("Access-Control-Allow-Origin", "*"))
            .from_text(response_text)
            .unwrap();
    });

    // Start the API server with the port designated in the config.
    app.run(&("0.0.0.0:".to_string() + &get_core_api_port()), 1)
        .ok();
}
