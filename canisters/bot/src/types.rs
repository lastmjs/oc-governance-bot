use ic_cdk::export::{
    candid::CandidType,
    serde::Deserialize
};

#[derive(CandidType, Deserialize, Debug)]
pub enum CanisterCreationStatus {
    Pending,
    InProgress,
    Created
}

#[derive(CandidType, Deserialize, Debug)]
pub enum ConfirmationState {
    PhoneNumber(PhoneNumber),
    RegistrationFee(CurrentUserResponseUnconfirmedStateRegistrationFee)
}

#[derive(CandidType, Deserialize, Debug)]
pub struct ConfirmedPendingUsername {
    canister_creation_status: CanisterCreationStatus,
    confirmation_state: ConfirmationState
}

#[derive(CandidType, Deserialize)]
pub struct CreateCanisterArgs {}

#[derive(CandidType, Deserialize, Debug)]
pub enum CreateCanisterResponse {
    Success(ic_cdk::export::Principal),
    UserNotFound,
    UserUnconfirmed,
    UserAlreadyCreated,
    CreationInProgress,
    CyclesBalanceTooLow,
    InternalError(String)
}

#[derive(CandidType, Deserialize)]
pub struct CurrentUserArgs {}

#[derive(CandidType, Deserialize, Debug)]
pub enum CurrentUserResponse {
    UserNotFound,
    Unconfirmed(CurrentUserResponseUnconfirmed),
    ConfirmedPendingUsername(ConfirmedPendingUsername),
    Confirmed(CurrentUserResponseConfirmed),
    Created(CurrentUserResponseCreated)
}

#[derive(CandidType, Deserialize, Debug)]
pub struct CurrentUserResponseConfirmed {
    username: String
}

#[derive(CandidType, Deserialize, Debug)]
pub struct CurrentUserResponseCreated {
    pub user_id: ic_cdk::export::Principal
}

#[derive(CandidType, Deserialize, Debug)]
pub struct CurrentUserResponseUnconfirmed {
    state: CurrentUserResponseUnconfirmedState
}

#[derive(CandidType, Deserialize, Debug)]
pub enum CurrentUserResponseUnconfirmedState {
    PhoneNumber(CurrentUserResponseUnconfirmedStatePhoneNumber),
    RegistrationFee(CurrentUserResponseUnconfirmedStateRegistrationFee)
}

#[derive(CandidType, Deserialize, Debug)]
pub struct CurrentUserResponseUnconfirmedStatePhoneNumber {
    phone_number: PhoneNumber,
    valid_until: u64
}

#[derive(CandidType, Deserialize, Debug)]
pub enum CurrentUserResponseUnconfirmedStateRegistrationFee {
    ICP(ICPRegistrationFee),
    Cycles(CyclesRegistrationFee)
}

#[derive(CandidType, Deserialize, Debug)]
pub struct CyclesRegistrationFee {
    valid_until: u64
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GroupChatSummary {
    name: String
}

#[derive(CandidType, Deserialize)]
pub struct GroupReplyContext {
    event_index: u32
}

#[derive(CandidType, Deserialize, Debug)]
pub struct ICPRegistrationFee {
    valid_until: u64
}

#[derive(CandidType, Deserialize, Debug)]
pub struct JoinGroupArgs {
    pub chat_id: ic_cdk::export::Principal,
    pub as_super_admin: bool
}

#[derive(CandidType, Deserialize, Debug)]
pub enum JoinGroupResponse {
    Success(GroupChatSummary),
    AlreadyInGroup,
    Blocked,
    GroupNotFound,
    GroupNotPublic,
    ParticipantLimitReached(u32),
    InternalError(String),
    NotSuperAdmin
}

#[derive(CandidType, Deserialize)]
pub enum MessageContent {
    Text(TextContent)
}

#[derive(CandidType, Deserialize, Clone)]
pub struct NeuronId {
    pub id: u64
}

#[derive(CandidType, Deserialize, Debug)]
pub struct PhoneNumber {
    country_code: u16,
    number: String
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Proposal {
    pub url: String,
    pub title: Option<String>,
    pub summary: String
}

#[derive(CandidType, Deserialize, Clone)]
pub struct ProposalInfo {
    pub id: Option<NeuronId>,
    pub topic: i32,
    pub proposal: Option<Proposal>
}

#[derive(CandidType, Deserialize)]
pub struct RegisterUserArgs {
    pub username: String
}

#[derive(CandidType, Deserialize, Debug)]
pub enum RegisterUserResponse {
    Success,
    AlreadyRegistered,
    UserLimitReached,
    UsernameTaken,
    UsernameInvalid,
    UsernameTooShort(u16),
    UsernameTooLong(u16),
    NotSupported
}

#[derive(CandidType, Deserialize)]
pub struct SendMessageArgs {
    pub message_id: u128,
    pub content: MessageContent,
    pub sender_name: String,
    pub replies_to: Option<GroupReplyContext>,
    pub mentioned: Vec<User>
}

#[derive(CandidType, Deserialize, Debug)]
pub enum SendMessageResponse {
    Success(SendMessageResponseSuccess),
    MessageEmpty,
    TextTooLong(u32),
    CallerNotInGroup
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SendMessageResponseSuccess {
    message_index: u32,
    event_index: u32,
    timestamp: u64
}

#[derive(CandidType, Deserialize)]
pub struct TextContent {
    pub text: String
}

#[derive(CandidType, Deserialize)]
pub struct User {
    pub user_id: ic_cdk::export::Principal,
    pub username: String
}