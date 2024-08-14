use gclient::{EventProcessor, GearApi, Result};
use gear_core::ids::{MessageId, ProgramId};
use gstd::{ActorId, Decode, Encode};
use sails_rs::U256;

pub const USERS_STR: &[&str] = &["//John", "//Mike", "//Dan"];

pub trait ApiUtils {
    fn get_actor_id(&self) -> ActorId;
    fn get_specific_actor_id(&self, value: impl AsRef<str>) -> ActorId;
}

impl ApiUtils for GearApi {
    fn get_actor_id(&self) -> ActorId {
        ActorId::new(
            self.account_id()
                .encode()
                .try_into()
                .expect("Unexpected invalid account id length."),
        )
    }

    fn get_specific_actor_id(&self, value: impl AsRef<str>) -> ActorId {
        let api_temp = self
            .clone()
            .with(value)
            .expect("Unable to build `GearApi` instance with provided signer.");
        api_temp.get_actor_id()
    }
}

#[macro_export]
macro_rules! send_request {
    (api: $api: expr, program_id: $program_id: expr, service_name: $name: literal, action: $action: literal, payload: ($($val: expr),*)) => {
        {
            let request = [
                $name.encode(),
                $action.to_string().encode(),
                ( $( $val, )*).encode(),
            ]
            .concat();

            let gas_info = $api
                .calculate_handle_gas(None, $program_id, request.clone(), 0, true)
                .await?;


            let (message_id, _) = $api
                .send_message_bytes($program_id, request.clone(), gas_info.min_limit, 0)
                .await?;

            message_id
        }

    };
}

pub async fn init(api: &GearApi) -> (MessageId, ProgramId) {
    let init = ("TokenName".to_owned(), "TokenSymbol".to_owned(), 10_u8);
    let request = ["New".encode(), init.encode()].concat();
    let path = "../../target/wasm32-unknown-unknown/release/gear_erc20_wasm.opt.wasm";

    let gas_info = api
        .calculate_upload_gas(
            None,
            gclient::code_from_os(path).unwrap(),
            request.clone(),
            0,
            true,
        )
        .await
        .expect("Error calculate upload gas");

    let (message_id, program_id, _hash) = api
        .upload_program_bytes(
            gclient::code_from_os(path).unwrap(),
            gclient::now_micros().to_le_bytes(),
            request,
            gas_info.min_limit,
            0,
        )
        .await
        .expect("Error upload program bytes");

    (message_id, program_id)
}

fn decode<T: Decode>(payload: Vec<u8>) -> Result<T> {
    Ok(T::decode(&mut payload.as_slice())?)
}

pub async fn get_state_balances(
    api: &GearApi,
    program_id: ProgramId,
    listener: &mut gclient::EventListener,
    skip: u32,
    take: u32,
) -> Vec<(sails_rs::ActorId, U256)> {
    let request = [
        "Admin".encode(),
        "Balances".to_string().encode(),
        (skip, take).encode(),
    ]
    .concat();

    let gas_info = api
        .calculate_handle_gas(None, program_id, request.clone(), 0, true)
        .await
        .expect("Error calculate handle gas");

    let (message_id, _) = api
        .send_message_bytes(program_id, request.clone(), gas_info.min_limit, 0)
        .await
        .expect("Error send message bytes");

    let (_, raw_reply, _) = listener
        .reply_bytes_on(message_id)
        .await
        .expect("Error listen reply");

    let decoded_reply: (String, String, Vec<(sails_rs::ActorId, U256)>) = match raw_reply {
        Ok(raw_reply) => decode(raw_reply).expect("Erroe decode reply"),
        Err(_error) => gstd::panic!("Error in getting reply"),
    };
    decoded_reply.2
}

pub async fn get_state_is_paused(
    api: &GearApi,
    program_id: ProgramId,
    listener: &mut gclient::EventListener,
) -> bool {
    let request = ["Pausable".encode(), "IsPaused".to_string().encode()].concat();

    let gas_info = api
        .calculate_handle_gas(None, program_id, request.clone(), 0, true)
        .await
        .expect("Error calculate handle gas");

    let (message_id, _) = api
        .send_message_bytes(program_id, request.clone(), gas_info.min_limit, 0)
        .await
        .expect("Error send message bytes");

    let (_, raw_reply, _) = listener
        .reply_bytes_on(message_id)
        .await
        .expect("Error listen reply");

    let decoded_reply: (String, String, bool) = match raw_reply {
        Ok(raw_reply) => decode(raw_reply).expect("Erroe decode reply"),
        Err(_error) => gstd::panic!("Error in getting reply"),
    };
    decoded_reply.2
}

pub async fn get_state_has_role(
    api: &GearApi,
    program_id: ProgramId,
    listener: &mut gclient::EventListener,
    user: ActorId,
    role: String,
) -> bool {
    let request = [
        "Admin".encode(),
        "HasRole".to_string().encode(),
        (user, role).encode(),
    ]
    .concat();

    let gas_info = api
        .calculate_handle_gas(None, program_id, request.clone(), 0, true)
        .await
        .expect("Error calculate handle gas");

    let (message_id, _) = api
        .send_message_bytes(program_id, request.clone(), gas_info.min_limit, 0)
        .await
        .expect("Error send message bytes");

    let (_, raw_reply, _) = listener
        .reply_bytes_on(message_id)
        .await
        .expect("Error listen reply");

    let decoded_reply: (String, String, bool) = match raw_reply {
        Ok(raw_reply) => decode(raw_reply).expect("Erroe decode reply"),
        Err(_error) => gstd::panic!("Error in getting reply"),
    };
    decoded_reply.2
}

pub async fn get_new_client(api: &GearApi, name: &str) -> GearApi {
    let alice_balance = api
        .total_balance(api.account_id())
        .await
        .expect("Error total balance");
    let amount = alice_balance / 10;

    api.transfer_keep_alive(
        api.get_specific_actor_id(name)
            .encode()
            .as_slice()
            .try_into()
            .expect("Unexpected invalid `ProgramId`."),
        amount,
    )
    .await
    .expect("Error transfer");

    api.clone().with(name).expect("Unable to change signer.")
}
