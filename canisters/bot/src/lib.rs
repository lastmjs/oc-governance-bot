use std::cell::RefCell;
use ic_cdk_macros::{
    query,
    update,
    heartbeat
};
use ic_cdk::export::candid::CandidType;
use std::collections::BTreeMap;
use ic_cdk::export::serde::Deserialize;

// TODO consider creating one group per proposal type

// TODO I don't even need to store the proposal really, just the ids...maybe just do that in a set
thread_local! {
    static COUNTER_REF: RefCell<i32> = RefCell::new(0);
    static PROPOSAL_INFO_MAP_REF: RefCell<BTreeMap<u64, ProposalInfo>> = RefCell::new(BTreeMap::new());
    static ERROR_MESSAGE_REF: RefCell<String> = RefCell::new(String::from(""));
    static SEND_MESSAGE_RESPONSE_REF: RefCell<String> = RefCell::new(String::from(""));
}

const NUM_ITERS: i32 = 1; // TODO I don't want to miss any proposals

#[derive(CandidType, Deserialize)]
struct RegisterUserArgs {
    username: String
}

#[derive(CandidType, Deserialize)]
struct CreateCanisterArgs {

}

#[derive(CandidType, Deserialize)]
struct CurrentUserArgs {

}

#[derive(CandidType, Deserialize, Debug)]
enum RegisterUserResponse {
    Success,
    AlreadyRegistered,
    UserLimitReached,
    UsernameTaken,
    UsernameInvalid,
    UsernameTooShort(u16),
    UsernameTooLong(u16),
    NotSupported
}

#[derive(CandidType, Deserialize, Debug)]
enum CreateCanisterResponse {
    Success(ic_cdk::export::Principal),
    UserNotFound,
    UserUnconfirmed,
    UserAlreadyCreated,
    CreationInProgress,
    CyclesBalanceTooLow,
    InternalError(String)
}

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

#[derive(CandidType, Deserialize, Clone)]
struct SendMessageArgs {
    message_id: u128,
    content: MessageContent,
    sender_name: String,
    replies_to: Option<GroupReplyContext>,
    mentioned: Vec<User>
}

#[derive(CandidType, Deserialize, Clone)]
struct User {
    user_id: ic_cdk::export::Principal,
    username: String
}

#[derive(CandidType, Deserialize, Clone)]
struct GroupReplyContext {
    event_index: u32
}

#[derive(CandidType, Deserialize, Clone)]
enum MessageContent {
    Text(TextContent)
}

#[derive(CandidType, Deserialize, Clone)]
struct TextContent {
    text: String
}

#[derive(CandidType, Deserialize, Clone)]
enum SendMessageResponse {
    Success(SendMessageResponseSuccess),
    MessageEmpty,
    TextTooLong(u32),
    CallerNotInGroup
}

#[derive(CandidType, Deserialize, Clone)]
struct SendMessageResponseSuccess {
    message_index: u32,
    event_index: u32,
    timestamp: u64
}

#[derive(CandidType, Deserialize, Debug)]
enum CurrentUserResponse {
    UserNotFound,
    Unconfirmed(CurrentUserResponseUnconfirmed),
    ConfirmedPendingUsername(ConfirmedPendingUsername),
    Confirmed(CurrentUserResponseConfirmed),
    Created(CurrentUserResponseCreated)
}

#[derive(CandidType, Deserialize, Debug)]
struct ConfirmedPendingUsername {
    canister_creation_status: CanisterCreationStatus,
    confirmation_state: ConfirmationState
}

#[derive(CandidType, Deserialize, Debug)]
enum CanisterCreationStatus {
    Pending,
    InProgress,
    Created
}

#[derive(CandidType, Deserialize, Debug)]
enum ConfirmationState {
    PhoneNumber(PhoneNumber),
    RegistrationFee(CurrentUserResponseUnconfirmedStateRegistrationFee)
}

#[derive(CandidType, Deserialize, Debug)]
struct CurrentUserResponseUnconfirmed {
    state: CurrentUserResponseUnconfirmedState
}

#[derive(CandidType, Deserialize, Debug)]
enum CurrentUserResponseUnconfirmedState {
    PhoneNumber(CurrentUserResponseUnconfirmedStatePhoneNumber),
    RegistrationFee(CurrentUserResponseUnconfirmedStateRegistrationFee)
}

#[derive(CandidType, Deserialize, Debug)]
enum CurrentUserResponseUnconfirmedStateRegistrationFee {
    ICP(ICPRegistrationFee),
    Cycles(CyclesRegistrationFee)
}

#[derive(CandidType, Deserialize, Debug)]
struct CyclesRegistrationFee {
    valid_until: u64
}

#[derive(CandidType, Deserialize, Debug)]
struct ICPRegistrationFee {
    valid_until: u64
}

#[derive(CandidType, Deserialize, Debug)]
struct CurrentUserResponseUnconfirmedStatePhoneNumber {
    phone_number: PhoneNumber,
    valid_until: u64
}

#[derive(CandidType, Deserialize, Debug)]
struct PhoneNumber {
    country_code: u16,
    number: String
}

#[derive(CandidType, Deserialize, Debug)]
struct CurrentUserResponseConfirmed {
    username: String
}

#[derive(CandidType, Deserialize, Debug)]
struct CurrentUserResponseCreated {
    user_id: ic_cdk::export::Principal
}

#[derive(CandidType, Deserialize, Debug)]
struct JoinGroupArgs {
    chat_id: ic_cdk::export::Principal,
    as_super_admin: bool
}

#[derive(CandidType, Deserialize, Debug)]
enum JoinGroupResponse {
    Success(GroupChatSummary),
    AlreadyInGroup,
    Blocked,
    GroupNotFound,
    GroupNotPublic,
    ParticipantLimitReached(u32),
    InternalError(String),
    NotSuperAdmin
}

#[derive(CandidType, Deserialize, Debug)]
struct GroupChatSummary {
    name: String
}

#[update]
async fn join_group(canister_id: String) -> String {
    let call_result: Result<(JoinGroupResponse,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text("edwiy-ryaaa-aaaaf-abu7q-cai").unwrap(),
        "join_group_v2",
        (JoinGroupArgs {
            chat_id: ic_cdk::export::Principal::from_text(canister_id).unwrap(),
            as_super_admin: false
        },)
    ).await;

    format!("{:#?}", call_result)
}

#[update]
async fn get_bot_canister_id() -> String {
    let call_result: Result<(CurrentUserResponse,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text("4bkt6-4aaaa-aaaaf-aaaiq-cai").unwrap(),
        "current_user",
        (CurrentUserArgs {},)
    ).await;

    match call_result {
        Ok(current_user_response_tuple) => {
            match current_user_response_tuple.0 {
                CurrentUserResponse::Created(current_user_response_created) => current_user_response_created.user_id.to_string(),
                CurrentUserResponse::Confirmed(current_user_response_confirmed) => current_user_response_confirmed.username,
                _ => "nothing".to_string()
            }
            // format!("{:#?}", current_user_response_tuple.0)
        },
        Err(error) => {
            error.1
        }
    }
}

#[update]
async fn send_message(message: String) -> String {
    let call_result: Result<(SendMessageResponse,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text("7x3dz-5qaaa-aaaaf-abrrq-cai").unwrap(),
        "send_message",
        (SendMessageArgs {
            message_id: 0,
            content: MessageContent::Text(TextContent { text: message }),
            sender_name: "GovernanceBot".to_string(),
            replies_to: None,
            mentioned: vec![]
        },)
    ).await;

    match call_result {
        Ok(send_message_response) => {
            match send_message_response.0 {
                SendMessageResponse::Success(_) => "Success".to_string(),
                SendMessageResponse::MessageEmpty => "MessageEmpty".to_string(),
                SendMessageResponse::TextTooLong(_) => "TextTooLong".to_string(),
                SendMessageResponse::CallerNotInGroup => "CallerNotInGroup".to_string()
            }
        },
        Err(error) => {
            error.1
        }
    }
}

#[update]
async fn register_user() -> String {
    let call_result: Result<(RegisterUserResponse,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text("4bkt6-4aaaa-aaaaf-aaaiq-cai").unwrap(),
        "register_user",
        (RegisterUserArgs {
            username: "GovernanceBot".to_string()
        },)
    ).await;

    match call_result {
        Ok(register_user_response_tuple) => {
            match register_user_response_tuple.0 {
                RegisterUserResponse::Success => "Success".to_string(),
                RegisterUserResponse::AlreadyRegistered => "AlreadyRegistered".to_string(),
                RegisterUserResponse::UserLimitReached => "UserLimitReached".to_string(),
                RegisterUserResponse::UsernameTaken => "UsernameTaken".to_string(),
                RegisterUserResponse::UsernameInvalid => "UsernameInvalid".to_string(),
                RegisterUserResponse::UsernameTooShort(_) => "UsernameTooShort".to_string(),
                RegisterUserResponse::UsernameTooLong(_) => "UsernameTooLong".to_string(),
                RegisterUserResponse::NotSupported => "NotSupported".to_string()
            }
        },
        Err(error) => {
            error.1
        }
    }
}

#[update]
async fn create_canister() -> String {
    let call_result: Result<(CreateCanisterResponse,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text("4bkt6-4aaaa-aaaaf-aaaiq-cai").unwrap(),
        "create_canister",
        (CreateCanisterArgs {},)
    ).await;

    match call_result {
        Ok(create_canister_response_tuple) => {
            match create_canister_response_tuple.0 {
                CreateCanisterResponse::Success(_) => "Success".to_string(),
                CreateCanisterResponse::UserNotFound => "UserNotFound".to_string(),
                CreateCanisterResponse::UserUnconfirmed => "UserUnconfirmed".to_string(),
                CreateCanisterResponse::UserAlreadyCreated => "UserAlreadyCreated".to_string(),
                CreateCanisterResponse::CreationInProgress => "CreationInProgress".to_string(),
                CreateCanisterResponse::CyclesBalanceTooLow => "CyclesBalanceTooLow".to_string(),
                CreateCanisterResponse::InternalError(_) => "UsernameTooLong".to_string()
            }
        },
        Err(error) => {
            error.1
        }
    }
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

#[query]
fn send_message_response() -> String {
    SEND_MESSAGE_RESPONSE_REF.with(|send_message_response_ref| send_message_response_ref.borrow().clone())
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
                        if let Some(neuron_id) = proposal_info.clone().id {
                            let neuron_id_clone = neuron_id.clone();

                            let proposal_info_clone = proposal_info.clone();

                            ic_cdk::spawn(async move {
                                let canister_id = get_group_canister_id(proposal_info.topic);

                                let call_result = send_message_to_group(
                                    &canister_id,
                                    neuron_id_clone.id as u128,
                                    &format_proposal_message(
                                        neuron_id_clone.id as u128,
                                        proposal_info.proposal
                                    )
                                ).await;

                                // TODO I do not have any retry logic...I would want to include that to be super robust
                                match call_result {
                                    Ok(send_message_response) => {
                                        SEND_MESSAGE_RESPONSE_REF.with(|send_message_response_ref| {
                                            let mut previous_send_message_response = send_message_response_ref.borrow_mut();
                        
                                            *previous_send_message_response = match send_message_response.0 {
                                                SendMessageResponse::Success(_) => "Success".to_string(),
                                                SendMessageResponse::MessageEmpty => "MessageEmpty".to_string(),
                                                SendMessageResponse::TextTooLong(_) => "TextTooLong".to_string(),
                                                SendMessageResponse::CallerNotInGroup => "CallerNotInGroup".to_string()
                                            };
                                        })
                                    },
                                    Err(error) => {
                                        SEND_MESSAGE_RESPONSE_REF.with(|send_message_response_ref| {
                                            let mut send_message_response = send_message_response_ref.borrow_mut();
                        
                                            *send_message_response = error.1;
                                        })
                                    }
                                }
                            });

                            proposal_info_map.insert(
                                neuron_id.clone().id,
                                proposal_info_clone
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

async fn send_message_to_group(
    canister_id: &str,
    message_id: u128,
    content: &str
) -> Result<(SendMessageResponse,), (ic_cdk::api::call::RejectionCode, String)> {
    ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text(canister_id).unwrap(),
        "send_message",
        (SendMessageArgs {
            message_id,
            content: MessageContent::Text(TextContent { text: content.to_string() }),
            sender_name: "GovernanceBot".to_string(),
            replies_to: None,
            mentioned: vec![]
        },)
    ).await
}

fn get_group_canister_id(topic: i32) -> String {
    match topic {
        0 => "7l7zi-kqaaa-aaaaf-abrtq-cai".to_string(),
        1 => "6gr5g-fyaaa-aaaaf-abrua-cai".to_string(),
        2 => "6bq3s-iaaaa-aaaaf-abruq-cai".to_string(),
        3 => "kcmzb-zaaaa-aaaaf-abscq-cai".to_string(),
        4 => "klps5-piaaa-aaaaf-absda-cai".to_string(),
        5 => "kmouj-cqaaa-aaaaf-absdq-cai".to_string(),
        6 => "nkjgq-xiaaa-aaaaf-absta-cai".to_string(),
        7 => "evgax-iiaaa-aaaaf-abtba-cai".to_string(),
        8 => "glmx5-dyaaa-aaaaf-abutq-cai".to_string(),
        9 => "hgctt-mqaaa-aaaaf-abuua-cai".to_string(),
        10 => "hbdvh-biaaa-aaaaf-abuuq-cai".to_string(),
        _ => "not found".to_string()
    }
}