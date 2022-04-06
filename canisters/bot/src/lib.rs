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
    INITIAL_PROPOSAL_LIMIT,
    PROCESS_INTERVAL,
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
use ic_cdk::api::time;
use ic_cdk_macros::{
    heartbeat,
    query
};
use std::cell::RefCell;
use types::{
    ListProposalInfo,
    ListProposalInfoResponse,
    MessageContent,
    Proposal,
    ProposalInfo,
    SendMessageArgs,
    SendMessageResponse,
    TextContent
};

thread_local! {
    static LAST_PROPOSAL_ID_SENT_REF_CELL: RefCell<u64> = RefCell::new(52537);
    static LIST_PROPOSALS_ERROR_REF_CELL: RefCell<(u64, String)> = RefCell::new((0, String::from("")));
    static PREVIOUS_PROCESS_TIME_REF_CELL: RefCell<u64> = RefCell::new(0);
    static SEND_MESSAGE_RESPONSE_REF_CELL: RefCell<(u64, String)> = RefCell::new((0, String::from("")));
}

#[heartbeat]
fn heartbeat() {
    PREVIOUS_PROCESS_TIME_REF_CELL.with(|previous_process_time_ref_cell| {
        let mut previous_process_time_ref = previous_process_time_ref_cell.borrow_mut();
        if *previous_process_time_ref == 0 {
            *previous_process_time_ref = time();
        }

        if time() > *previous_process_time_ref + PROCESS_INTERVAL {
            process_proposals();
            *previous_process_time_ref = time();
        }
    });
}

#[query]
fn previous_process_time() -> u64 {
    PREVIOUS_PROCESS_TIME_REF_CELL.with(|previous_process_time_ref_cell| *previous_process_time_ref_cell.borrow())
}

#[query]
fn last_proposal_id_sent() -> u64 {
    LAST_PROPOSAL_ID_SENT_REF_CELL.with(|last_proposal_id_sent_ref_cell| *last_proposal_id_sent_ref_cell.borrow())
}

#[query]
fn list_proposals_error() -> (u64, String) {
    LIST_PROPOSALS_ERROR_REF_CELL.with(|list_proposals_error_ref_cell| list_proposals_error_ref_cell.borrow().clone())
}

#[query]
fn send_message_response() -> (u64, String) {
    SEND_MESSAGE_RESPONSE_REF_CELL.with(|send_message_response_ref_cell| send_message_response_ref_cell.borrow().clone())
}

fn process_proposals() {
    ic_cdk::spawn(async {    
        let proposals_infos_result = get_proposal_infos(INITIAL_PROPOSAL_LIMIT).await;
            
        match proposals_infos_result {
            Ok(proposal_infos) => {
                process_proposal_infos(proposal_infos).await;
            },
            Err(error) => {
                LIST_PROPOSALS_ERROR_REF_CELL.with(|list_proposals_error_ref_cell| {
                    let mut list_proposals_error_ref = list_proposals_error_ref_cell.borrow_mut();
    
                    *list_proposals_error_ref = (time(), error);
                })
            }
        }
    });
}

// TODO recursive async functions are complicated and might lead to some issues on the IC (call stack)
// TODO easier to go with the less functional iterative method for now
async fn get_proposal_infos(limit: u32) -> Result<Vec<ProposalInfo>, String> {
    let last_proposal_id_sent = LAST_PROPOSAL_ID_SENT_REF_CELL.with(|last_proposal_id_sent_ref_cell| *last_proposal_id_sent_ref_cell.borrow());

    let mut proposal_infos = list_proposals(limit).await?;
    let mut index = 1;

    while proposal_infos_contains_last_proposal(
        last_proposal_id_sent,
        &proposal_infos
    ) == false {
        proposal_infos = list_proposals(limit + index).await?;
        index += 1;
    }

    let mut proposal_infos_without_last_proposal = remove_proposal_from_proposal_infos(
        last_proposal_id_sent,
        proposal_infos
    );

    proposal_infos_without_last_proposal.reverse();

    Ok(proposal_infos_without_last_proposal)
}

fn proposal_infos_contains_last_proposal(
    last_proposal_id: u64,
    proposal_infos: &Vec<ProposalInfo>
) -> bool {
    proposal_infos.into_iter().find(|proposal_info| {
        if let Some(neuron_id) = &proposal_info.id {
            neuron_id.id == last_proposal_id
        }
        else {
            false
        }
    })
    .is_some()
}

fn remove_proposal_from_proposal_infos(
    proposal_id: u64,
    proposal_infos: Vec<ProposalInfo>
) -> Vec<ProposalInfo> {
    proposal_infos.into_iter().filter(|proposal_info| {
        if let Some(neuron_id) = &proposal_info.id {
            neuron_id.id != proposal_id
        }
        else {
            true
        }
    })
    .collect::<Vec<ProposalInfo>>()
}

async fn list_proposals(limit: u32) -> Result<Vec<ProposalInfo>, String> {
    let call_result: Result<(ListProposalInfoResponse,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text(GOVERNANCE_CANISTER_ID).unwrap(),
        "list_proposals",
        (ListProposalInfo {
            include_reward_status: vec![],
            before_proposal: None,
            limit,
            exclude_topic: vec![],
            include_status: vec![]
        },)
    ).await;

    match call_result {
        Ok(value) => Ok(value.0.proposal_info),
        Err(error) => Err(error.1)
    }
}

async fn process_proposal_infos(proposal_infos: Vec<ProposalInfo>) {
    for proposal_info in proposal_infos.iter() {
        if let Some(neuron_id) = &proposal_info.id {
            process_proposal_info(
                neuron_id.id as u128,
                proposal_info.clone()
            ).await;

            LAST_PROPOSAL_ID_SENT_REF_CELL.with(|last_proposal_id_sent_ref_cell| {
                let mut last_proposal_id_sent_ref = last_proposal_id_sent_ref_cell.borrow_mut();

                *last_proposal_id_sent_ref = neuron_id.id;
            });
        }
    }
}

async fn process_proposal_info(
    neuron_id: u128,
    proposal_info: ProposalInfo
) {
    let canister_id = get_group_canister_id(proposal_info.topic);

    let call_result = send_message_to_group(
        &canister_id,
        neuron_id,
        &format_proposal_message(
            neuron_id,
            proposal_info.proposal
        )
    ).await;

    SEND_MESSAGE_RESPONSE_REF_CELL.with(|send_message_response_ref_cell| {
        let mut send_message_response_ref = send_message_response_ref_cell.borrow_mut();

        *send_message_response_ref = (time(), format!("{:#?}", call_result));
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

        // TODO I would love to truncate the string immutably, but this is simple for now
        let mut proposal_message = format!(
            "{dashboard_url}{title}{summary}{url}",
            dashboard_url = dashboard_url,
            title = title,
            summary = summary,
            url = url
        );

        if proposal_message.len() > 1000 {
            proposal_message.truncate(1000);
        
            format!("{proposal_message}...", proposal_message = proposal_message)
        }
        else {
            proposal_message
        }
    }
    else {
        "".to_string()
    }
}