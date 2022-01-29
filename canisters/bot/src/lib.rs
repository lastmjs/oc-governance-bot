// TODO Figure out how to return Result from update/query calls so that we can get rid of unwrap and use the ? operator
// TODO Follow this forum post for the above: https://forum.dfinity.org/t/return-rust-result-from-update-or-query-call/10577

// mod bot_utilities; // uncomment this to enable the bot utilities
mod constants;
mod types;

// uncomment this to enable the bot utilities
// pub use bot_utilities::{
//     create_governance_bot_user_canister,
//     register_governance_bot_user
// };
use constants::{
    GOVERNANCE_BOT_USER_NAME,
    GOVERNANCE_CANISTER_ID,
    TOPIC_0_GROUP_CANISTER_ID,
    TOPIC_1_GROUP_CANISTER_ID,
    TOPIC_2_GROUP_CANISTER_ID,
    TOPIC_3_GROUP_CANISTER_ID,
    TOPIC_4_GROUP_CANISTER_ID,
    TOPIC_5_GROUP_CANISTER_ID,
    TOPIC_6_GROUP_CANISTER_ID,
    TOPIC_7_GROUP_CANISTER_ID,
    TOPIC_8_GROUP_CANISTER_ID,
    TOPIC_9_GROUP_CANISTER_ID,
    TOPIC_10_GROUP_CANISTER_ID
};
use ic_cdk_macros::{
    heartbeat,
    query
};
use std::{
    cell::RefCell,
    collections::HashMap
};
use types::{
    MessageContent,
    Proposal,
    ProposalInfo,
    SendMessageArgs,
    SendMessageResponse,
    TextContent
};

thread_local! {
    static PROPOSAL_IDS_MAP_REF: RefCell<HashMap<u64, ()>> = RefCell::new(HashMap::new()); // TODO eventually we will want to cull this, we do not to keep them around forever
    static PENDING_PROPOSALS_ERROR_REF: RefCell<String> = RefCell::new(String::from(""));
    static SEND_MESSAGE_RESPONSE_REF: RefCell<String> = RefCell::new(String::from(""));
}

#[heartbeat]
fn heartbeat() {
    process_proposals();
}

#[query]
fn proposal_ids() -> Vec<u64> {
    PROPOSAL_IDS_MAP_REF.with(|proposal_ids_map_ref| proposal_ids_map_ref.borrow().clone().into_keys().collect::<Vec<u64>>())
}

#[query]
fn pending_proposals_error() -> String {
    PENDING_PROPOSALS_ERROR_REF.with(|pending_proposals_error_ref| pending_proposals_error_ref.borrow().clone())
}

#[query]
fn send_message_response() -> String {
    SEND_MESSAGE_RESPONSE_REF.with(|send_message_response_ref| send_message_response_ref.borrow().clone())
}

fn process_proposals() {
    ic_cdk::spawn(async {
        let proposals_infos_result = get_proposal_infos().await;
        
        match proposals_infos_result {
            Ok(proposal_infos) => {
                process_proposal_infos(proposal_infos);
            },
            Err(error) => {
                PENDING_PROPOSALS_ERROR_REF.with(|pending_proposals_error_ref| {
                    let mut pending_proposals_error = pending_proposals_error_ref.borrow_mut();

                    *pending_proposals_error = error;
                })
            }
        }
    });
}

async fn get_proposal_infos() -> Result<Vec<ProposalInfo>, String> {
    let call_result: Result<(Vec<ProposalInfo>,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text(GOVERNANCE_CANISTER_ID).unwrap(),
        "get_pending_proposals",
        ()
    ).await;

    match call_result {
        Ok(value) => Ok(value.0),
        Err(error) => Err(error.1)
    }
}

fn process_proposal_infos(proposal_infos: Vec<ProposalInfo>) {
    PROPOSAL_IDS_MAP_REF.with(|proposal_ids_map_ref| {
        let mut proposal_ids_map = proposal_ids_map_ref.borrow_mut();

        proposal_infos.into_iter().for_each(|proposal_info| {
            if let Some(neuron_id) = &proposal_info.id {
                let proposal_already_processed = proposal_ids_map.contains_key(&neuron_id.id);

                if proposal_already_processed == true {
                    return;
                }

                // TODO we may want to implement retry logic in case the send message call fails
                // TODO this probably only matters for proposals that are open for very short windows of time (like conversion rate proposals)
                process_proposal_info(
                    neuron_id.id as u128,
                    proposal_info.clone()
                );

                proposal_ids_map.insert(
                    neuron_id.id,
                    ()
                );
            }
        });
    });
}

fn process_proposal_info(
    neuron_id: u128,
    proposal_info: ProposalInfo
) {
    ic_cdk::spawn(async move {
        let canister_id = get_group_canister_id(proposal_info.topic);

        let call_result = send_message_to_group(
            &canister_id,
            neuron_id,
            &format_proposal_message(
                neuron_id,
                proposal_info.proposal
            )
        ).await;

        SEND_MESSAGE_RESPONSE_REF.with(|send_message_response_ref| {
            let mut send_message_response = send_message_response_ref.borrow_mut();

            *send_message_response = format!("{:#?}", call_result);
        })
    });
}

fn get_group_canister_id(topic: i32) -> String {
    match topic {
        0 => TOPIC_0_GROUP_CANISTER_ID.to_string(),
        1 => TOPIC_1_GROUP_CANISTER_ID.to_string(),
        2 => TOPIC_2_GROUP_CANISTER_ID.to_string(),
        3 => TOPIC_3_GROUP_CANISTER_ID.to_string(),
        4 => TOPIC_4_GROUP_CANISTER_ID.to_string(),
        5 => TOPIC_5_GROUP_CANISTER_ID.to_string(),
        6 => TOPIC_6_GROUP_CANISTER_ID.to_string(),
        7 => TOPIC_7_GROUP_CANISTER_ID.to_string(),
        8 => TOPIC_8_GROUP_CANISTER_ID.to_string(),
        9 => TOPIC_9_GROUP_CANISTER_ID.to_string(),
        10 => TOPIC_10_GROUP_CANISTER_ID.to_string(),
        _ => "not found".to_string() // TODO this should probably return an error, we shouldn't even attempt a call at this point
    }
}

async fn send_message_to_group(
    group_canister_id: &str,
    message_id: u128,
    content: &str
) -> Result<(SendMessageResponse,), (ic_cdk::api::call::RejectionCode, String)> {
    ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text(group_canister_id).unwrap(),
        "send_message",
        (SendMessageArgs {
            message_id,
            content: MessageContent::Text(TextContent { text: content.to_string() }),
            sender_name: GOVERNANCE_BOT_USER_NAME.to_string(),
            replies_to: None,
            mentioned: vec![]
        },)
    ).await
}

fn format_proposal_message(
    id: u128,
    proposal_option: Option<Proposal>
) -> String {
    if let Some(proposal) = proposal_option {
        let dashboard_url = format!("https://dashboard.internetcomputer.org/proposal/{}", id);
        
        let title = match &proposal.title {
            Some(title) => format!("\n\n{}", title.to_string()),
            None => "".to_string()
        };
    
        let summary = match &proposal.summary[..] {
            "" => "".to_string(),
            _ => format!("\n\n{}", proposal.summary)
        };
    
        let url = match &proposal.url[..] {
            "" => "".to_string(),
            _ => format!("\n\n{}", proposal.url)
        };
    
        format!(
            "{dashboard_url}{title}{summary}{url}",
            dashboard_url = dashboard_url,
            title = title,
            summary = summary,
            url = url
        )
    }
    else {
        "".to_string()
    }
}