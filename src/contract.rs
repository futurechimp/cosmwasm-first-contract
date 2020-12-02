use cosmwasm_std::{
    to_binary, Api, BankMsg, Binary, Context, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdError, StdResult, Storage,
};

use crate::state::{config, config_read, State};
use crate::{
    msg::{HandleMsg, InitMsg, QueryMsg},
    state::ConfigResponse,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    if msg.expires <= env.block.height {
        return Err(StdError::generic_err("Cannot create expired option"));
    }

    let state = State {
        creator: env.message.sender.clone(),
        owner: env.message.sender.clone(),
        collateral: env.message.sent_funds,
        counter_offer: msg.counter_offer,
        expires: msg.expires,
    };
    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Transfer { recipient } => handle_transfer(deps, env, recipient),
        HandleMsg::Execute {} => handle_execute(deps, env),
        HandleMsg::Burn {} => handle_burn(deps, env),
    }
}

pub fn handle_burn<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // ensure is expired
    let state = config(&mut deps.storage).load()?;
    if env.block.height < state.expires {
        return Err(StdError::generic_err("option not yet expired"));
    }

    // ensure sending proper counter_offer
    if !env.message.sent_funds.is_empty() {
        return Err(StdError::generic_err("don't send funds with burn"));
    }

    // release collateral to creator
    let mut res = Context::new();
    res.add_message(BankMsg::Send {
        from_address: env.contract.address,
        to_address: state.creator,
        amount: state.collateral,
    });

    // delete the option
    config(&mut deps.storage).remove();

    res.add_log("action", "burn");
    Ok(res.into())
}

pub fn handle_execute<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // ensure msg sender is the owner
    let state = config(&mut deps.storage).load()?;
    if env.message.sender != state.owner {
        return Err(StdError::unauthorized());
    }

    // ensure not expired
    if env.block.height >= state.expires {
        return Err(StdError::generic_err("option expired"));
    }

    // env.message.send_funds is the coins sent along the tx.
    // ensure sending proper counter_offer
    if env.message.sent_funds != state.counter_offer {
        return Err(StdError::generic_err(format!(
            "must send exact counter offer: {:?}",
            state.counter_offer
        )));
    }

    // release counter_offer to creator
    let mut res = Context::new();

    /*
     * CosmWasm smart contract cannot execute native Cosmos SDK messages
     * such as bank send msg. Instead we return native messages in the execution context
     * to be processed by Cosmos SDK transaction runtime.
     */
    res.add_message(BankMsg::Send {
        from_address: env.contract.address.clone(),
        to_address: state.creator,
        amount: state.counter_offer,
    });

    // release collateral to sender
    res.add_message(BankMsg::Send {
        from_address: env.contract.address,
        to_address: state.owner,
        amount: state.collateral,
    });

    // delete the option
    config(&mut deps.storage).remove();

    /*
     * logs will go to the cosmos sdk event log.
     */
    res.add_log("action", "execute");
    Ok(res.into())
    /*
     * res.into converts Context to HandleResponse.
     */
}

pub fn handle_transfer<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: HumanAddr,
) -> StdResult<HandleResponse> {
    // ensure msg sender is the owner
    let mut state = config(&mut deps.storage).load()?;

    if env.message.sender != state.owner {
        return Err(StdError::unauthorized());
    }

    // set new owner on state
    state.owner = recipient.clone();
    config(&mut deps.storage).save(&state)?;

    let mut res = Context::new();
    res.add_log("action", "transfer");
    res.add_log("owner", recipient);

    Ok(res.into())
}

// pub fn try_increment<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
//     _env: Env,
// ) -> StdResult<HandleResponse> {
//     config(&mut deps.storage).update(|mut state| {
//         state.count += 1;
//         Ok(state)
//     })?;

//     Ok(HandleResponse::default())
// }

// pub fn try_reset<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
//     env: Env,
//     count: i32,
// ) -> StdResult<HandleResponse> {
//     let api = &deps.api;
//     config(&mut deps.storage).update(|mut state| {
//         if api.canonical_address(&env.message.sender)? != state.owner {
//             return Err(StdError::unauthorized());
//         }
//         state.count = count;
//         Ok(state)
//     })?;
//     Ok(HandleResponse::default())
// }

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(state)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies, mock_env};
//     use cosmwasm_std::{coins, from_binary, StdError};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies(20, &[]);

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = init(&mut deps, env, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies(20, &coins(2, "token"));

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(2, "token"));
//         let _res = init(&mut deps, env, msg).unwrap();

//         // beneficiary can release it
//         let env = mock_env("anyone", &coins(2, "token"));
//         let msg = HandleMsg::Increment {};
//         let _res = handle(&mut deps, env, msg).unwrap();

//         // should increase counter by 1
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies(20, &coins(2, "token"));

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(2, "token"));
//         let _res = init(&mut deps, env, msg).unwrap();

//         // beneficiary can release it
//         let unauth_env = mock_env("anyone", &coins(2, "token"));
//         let msg = HandleMsg::Reset { count: 5 };
//         let res = handle(&mut deps, unauth_env, msg);
//         match res {
//             Err(StdError::Unauthorized { .. }) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_env = mock_env("creator", &coins(2, "token"));
//         let msg = HandleMsg::Reset { count: 5 };
//         let _res = handle(&mut deps, auth_env, msg).unwrap();

//         // should now be 5
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }
