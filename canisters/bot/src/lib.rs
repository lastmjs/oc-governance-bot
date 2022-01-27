use std::cell::RefCell;
use ic_cdk_macros::{
    query,
    heartbeat
};
use ic_cdk::export::candid::CandidType;
use std::collections::BTreeMap;
use ic_cdk::export::serde::Deserialize;

// TODO I don't even need to store the proposal really, just the ids...maybe just do that in a set
thread_local! {
    static COUNTER_REF: RefCell<i32> = RefCell::new(0);
    static PROPOSAL_INFO_MAP_REF: RefCell<BTreeMap<u64, ProposalInfo>> = RefCell::new(BTreeMap::new());
    static ERROR_MESSAGE_REF: RefCell<String> = RefCell::new(String::from(""));
}

const NUM_ITERS: i32 = 10;

#[derive(CandidType, Deserialize, Clone)]
struct ProposalInfo {
    id: Option<NeuronId>,
    topic: i32,
    proposal: Option<Proposal>
}

#[derive(CandidType, Deserialize, Clone)]
struct Proposal {
    url: String,
    title: Option<String>,
    summary: String
}

#[derive(CandidType, Deserialize, Clone)]
struct NeuronId {
    id: u64
}

#[query]
fn counter() -> i32 {
    COUNTER_REF.with(|counter_ref| *counter_ref.borrow())
}

#[query]
fn proposal_ids() -> Vec<u64> {
    PROPOSAL_INFO_MAP_REF.with(|proposal_info_map_ref| {
        proposal_info_map_ref.borrow().clone().into_keys().collect::<Vec<u64>>()
    })
}

#[query]
fn proposal_infos() -> Vec<ProposalInfo> {
    PROPOSAL_INFO_MAP_REF.with(|proposal_info_map_ref| {
        proposal_info_map_ref.borrow().clone().into_values().collect::<Vec<ProposalInfo>>()
    })
}

#[query]
fn error_message() -> String {
    ERROR_MESSAGE_REF.with(|error_message_ref| error_message_ref.borrow().clone())
}

#[heartbeat]
fn canister_heartbeat() {
    COUNTER_REF.with(|counter_ref| {
        let mut counter = counter_ref.borrow_mut();

        *counter = *counter + 1;
        
        let interval_elapsed = *counter % NUM_ITERS == 0;

        if interval_elapsed == true {
            process_proposals();
        }
    });
}

fn process_proposals() {
    ic_cdk::spawn(async {
        ic_cdk::println!("do governance things");

        let call_result: Result<(Vec<ProposalInfo>,), _> = ic_cdk::api::call::call(
            ic_cdk::export::Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap(),
            "get_pending_proposals",
            ()
        ).await;
        
        match call_result {
            Ok(value) => {
                let proposal_infos = value.0;

                PROPOSAL_INFO_MAP_REF.with(|proposal_info_map_ref| {
                    let mut proposal_info_map = proposal_info_map_ref.borrow_mut();

                    proposal_infos.into_iter().for_each(|proposal_info| {
                        if let Some(neuron_id) = &proposal_info.id {
                            proposal_info_map.insert(
                                neuron_id.id,
                                proposal_info.clone()
                            );
                        }
                    });
                });
            },
            Err(error) => {
                ERROR_MESSAGE_REF.with(|error_message_ref| {
                    let mut error_message = error_message_ref.borrow_mut();

                    *error_message = error.1;
                })
            }
        }
    });
}