use crate::{
    constants::{
        GOVERNANCE_BOT_USER_CANISTER_ID,
        GOVERNANCE_BOT_USER_NAME,
        GOVERNANCE_CANISTER_ID,
        USER_INDEX_CANISTER_ID
    },
    types::{
        CreateCanisterArgs,
        CreateCanisterResponse,
        CurrentUserArgs,
        CurrentUserResponse,
        JoinGroupArgs,
        JoinGroupResponse,
        ListProposalInfo,
        ListProposalInfoResponse,
        MessageContent,
        ProposalInfo,
        RegisterUserArgs,
        RegisterUserResponse,
        SendMessageArgs,
        SendMessageResponse,
        TextContent
    }
};
use ic_cdk_macros::update;

#[update]
pub async fn register_governance_bot_user() -> String {
    let call_result: Result<(RegisterUserResponse,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text(USER_INDEX_CANISTER_ID).unwrap(),
        "register_user",
        (RegisterUserArgs {
            username: GOVERNANCE_BOT_USER_NAME.to_string()
        },)
    ).await;

    format!("{:#?}", call_result)
}

#[update]
pub async fn create_governance_bot_user_canister() -> String {
    let call_result: Result<(CreateCanisterResponse,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text(USER_INDEX_CANISTER_ID).unwrap(),
        "create_canister",
        (CreateCanisterArgs {},)
    ).await;

    format!("{:#?}", call_result)
}

#[update]
async fn get_governance_bot_user_canister_id() -> String {
    let call_result: Result<(CurrentUserResponse,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text(USER_INDEX_CANISTER_ID).unwrap(),
        "current_user",
        (CurrentUserArgs {},)
    ).await;

    match call_result {
        Ok(current_user_response_tuple) => {
            match current_user_response_tuple.0 {
                CurrentUserResponse::Created(current_user_response_created) => current_user_response_created.user_id.to_string(),
                _ => "Not Implemented".to_string()
            }
        },
        Err(error) => {
            error.1
        }
    }
}

#[update]
async fn governance_bot_join_group(group_canister_id: String) -> String {
    let call_result: Result<(JoinGroupResponse,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text(GOVERNANCE_BOT_USER_CANISTER_ID).unwrap(),
        "join_group_v2",
        (JoinGroupArgs {
            chat_id: ic_cdk::export::Principal::from_text(group_canister_id).unwrap(),
            as_super_admin: false
        },)
    ).await;

    format!("{:#?}", call_result)
}

#[update]
async fn send_message(
    message: String,
    group_canister_id: String
) -> String {
    let call_result: Result<(SendMessageResponse,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text(group_canister_id).unwrap(),
        "send_message",
        (SendMessageArgs {
            message_id: 0,
            content: MessageContent::Text(TextContent { text: message }),
            sender_name: GOVERNANCE_BOT_USER_NAME.to_string(),
            replies_to: None,
            mentioned: vec![]
        },)
    ).await;

    format!("{:#?}", call_result)
}

#[update]
async fn list_proposals() -> Result<Vec<ProposalInfo>, String> {
    let call_result: Result<(ListProposalInfoResponse,), _> = ic_cdk::api::call::call(
        ic_cdk::export::Principal::from_text(GOVERNANCE_CANISTER_ID).unwrap(),
        "list_proposals",
        (ListProposalInfo {
            include_reward_status: vec![],
            before_proposal: None,
            limit: 1,
            exclude_topic: vec![],
            include_status: vec![]
        },)
    ).await;

    match call_result {
        Ok(value) => Ok(value.0.proposal_info),
        Err(error) => Err(error.1)
    }
}