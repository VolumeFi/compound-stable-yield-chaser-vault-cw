#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint256};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, PalomaMsg, QueryMsg};
use crate::state::{State, STATE};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:compound-stable-yield-chaser-vault-cw";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        retry_delay: msg.retry_delay,
        job_id: msg.job_id.clone(),
        owner: info.sender.clone(),
        release_nonce: 0u128,
    };
    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("job_id", msg.job_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<PalomaMsg>, ContractError> {
    match msg {
        ExecuteMsg::ChangeAsset {
            new_c_asset,
            swap_info,
        } => execute::change_asset(deps, env, info, new_c_asset, swap_info),
        ExecuteMsg::SetBobby { bobby } => execute::set_bobby(deps, info, bobby),
        ExecuteMsg::ReleaseBobby {
            recipient,
            amount,
            nonce,
        } => execute::release_bobby(deps, info, recipient, amount, nonce),
        ExecuteMsg::ReRelease { nonce } => execute::rerelease_bobby(deps, info, nonce),
        ExecuteMsg::SetPaloma {} => execute::set_paloma(deps, info),
        ExecuteMsg::UpdateCompass { new_compass } => {
            execute::update_compass(deps, info, new_compass)
        }
        ExecuteMsg::UpdateRefundWallet { new_refund_wallet } => {
            execute::update_refund_wallet(deps, info, new_refund_wallet)
        }
        ExecuteMsg::UpdateEntranceFee { new_entrance_fee } => {
            execute::update_entrance_fee(deps, info, new_entrance_fee)
        }
        ExecuteMsg::UpdateServiceFeeCollector {
            new_service_fee_collector,
        } => execute::update_service_fee_collector(deps, info, new_service_fee_collector),
        ExecuteMsg::UpdateServiceFee { new_service_fee } => {
            execute::update_service_fee(deps, info, new_service_fee)
        }
        ExecuteMsg::UpdateJobId { new_job_id } => execute::update_job_id(deps, info, new_job_id),
    }
}

pub mod execute {
    use super::*;
    use crate::{
        msg::{ExecuteJob, SwapInfo},
        state::{NONCE, WITHDRAW_TIMESTAMP},
        ContractError::{AllPending, Unauthorized},
    };
    use cosmwasm_std::CosmosMsg;
    use ethabi::{Address, Contract, Function, Param, ParamType, StateMutability, Token, Uint};
    use std::collections::BTreeMap;
    use std::str::FromStr;

    pub fn change_asset(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        new_asset: String,
        swap_info: SwapInfo,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "change_asset".to_string(),
                vec![Function {
                    name: "change_asset".to_string(),
                    inputs: vec![
                        Param {
                            name: "_new_c_asset".to_string(),
                            kind: ParamType::Address,
                            internal_type: None,
                        },
                        Param {
                            name: "swap_info".to_string(),
                            kind: ParamType::Tuple(vec![
                                ParamType::FixedArray(Box::new(ParamType::Address), 11),
                                ParamType::FixedArray(
                                    Box::new(ParamType::FixedArray(
                                        Box::new(ParamType::Uint(256)),
                                        5,
                                    )),
                                    5,
                                ),
                                ParamType::Uint(256),
                                ParamType::Uint(256),
                                ParamType::FixedArray(Box::new(ParamType::Address), 5),
                            ]),
                            internal_type: None,
                        },
                    ],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        let retry_delay: u64 = state.retry_delay;
        let mut tokens: Vec<Token> = Vec::new();
        if let Some(timestamp) = WITHDRAW_TIMESTAMP.may_load(
            deps.storage,
            ("change_asset".to_string(), new_asset.clone()),
        )? {
            if timestamp.plus_seconds(retry_delay).lt(&env.block.time) {
                tokens = get_change_asset_tokens(new_asset.clone(), swap_info);
                WITHDRAW_TIMESTAMP.save(
                    deps.storage,
                    ("change_asset".to_string(), new_asset),
                    &env.block.time,
                )?;
            }
        } else {
            tokens = get_change_asset_tokens(new_asset.clone(), swap_info);
            WITHDRAW_TIMESTAMP.save(
                deps.storage,
                ("change_asset".to_string(), new_asset),
                &env.block.time,
            )?;
        }
        if tokens.is_empty() {
            Err(AllPending {})
        } else {
            Ok(Response::new()
                .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                    execute_job: ExecuteJob {
                        job_id: state.job_id,
                        payload: Binary::new(
                            contract
                                .function("change_asset")
                                .unwrap()
                                .encode_input(tokens.as_slice())
                                .unwrap(),
                        ),
                    },
                }))
                .add_attribute("action", "change_asset"))
        }
    }

    fn get_change_asset_tokens(new_asset: String, swap_info: SwapInfo) -> Vec<Token> {
        let token_new_asset: Token = Token::Address(Address::from_str(new_asset.as_str()).unwrap());
        let mut token_swap_info: Vec<Token> = Vec::new();
        token_swap_info.push(Token::FixedArray(
            swap_info
                .route
                .iter()
                .map(|x| Token::Address(Address::from_str(x.as_str()).unwrap()))
                .collect(),
        ));
        token_swap_info.push(Token::FixedArray(
            swap_info
                .swap_params
                .iter()
                .map(|x| {
                    Token::FixedArray(
                        x.iter()
                            .map(|y| Token::Uint(Uint::from_big_endian(&y.to_be_bytes())))
                            .collect(),
                    )
                })
                .collect(),
        ));
        token_swap_info.push(Token::Uint(Uint::from_big_endian(
            &swap_info.amount.to_be_bytes(),
        )));
        token_swap_info.push(Token::Uint(Uint::from_big_endian(
            &swap_info.expected.to_be_bytes(),
        )));
        token_swap_info.push(Token::FixedArray(
            swap_info
                .pools
                .iter()
                .map(|x| Token::Address(Address::from_str(x.as_str()).unwrap()))
                .collect(),
        ));
        vec![token_new_asset, Token::Tuple(token_swap_info)]
    }

    pub fn set_paloma(
        deps: DepsMut,
        info: MessageInfo,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "set_paloma".to_string(),
                vec![Function {
                    name: "set_paloma".to_string(),
                    inputs: vec![],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: state.job_id,
                    payload: Binary::new(
                        contract
                            .function("set_paloma")
                            .unwrap()
                            .encode_input(&[])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "set_paloma"))
    }

    pub fn set_bobby(
        deps: DepsMut,
        info: MessageInfo,
        bobby: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        let bobby_address: Address = Address::from_str(bobby.as_str()).unwrap();
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "set_bobby".to_string(),
                vec![Function {
                    name: "set_bobby".to_string(),
                    inputs: vec![Param {
                        name: "_bobby".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };

        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: state.job_id,
                    payload: Binary::new(
                        contract
                            .function("set_bobby")
                            .unwrap()
                            .encode_input(&[Token::Address(bobby_address)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "set_bobby"))
    }

    pub fn release_bobby(
        deps: DepsMut,
        info: MessageInfo,
        recipient: String,
        amount: Uint256,
        nonce: Uint256,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        let recipient_address: Address = Address::from_str(recipient.as_str()).unwrap();
        NONCE.save(
            deps.storage,
            &nonce.to_be_bytes(),
            &(recipient.clone(), amount),
        )?;
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "release_bobby".to_string(),
                vec![Function {
                    name: "release_bobby".to_string(),
                    inputs: vec![
                        Param {
                            name: "recipient".to_string(),
                            kind: ParamType::Address,
                            internal_type: None,
                        },
                        Param {
                            name: "amount".to_string(),
                            kind: ParamType::Uint(256),
                            internal_type: None,
                        },
                        Param {
                            name: "nonce".to_string(),
                            kind: ParamType::Uint(256),
                            internal_type: None,
                        },
                    ],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };

        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: state.job_id,
                    payload: Binary::new(
                        contract
                            .function("release_bobby")
                            .unwrap()
                            .encode_input(&[
                                Token::Address(recipient_address),
                                Token::Uint(Uint::from_big_endian(&amount.to_be_bytes())),
                                Token::Uint(Uint::from_big_endian(&nonce.to_be_bytes())),
                            ])
                            .unwrap(),
                    ),
                },
            }))
            .add_attributes(vec![
                ("action", "release_bobby"),
                ("recipient", &recipient),
                ("amount", &amount.to_string()),
                ("nonce", &nonce.to_string()),
            ]))
    }

    pub fn rerelease_bobby(
        deps: DepsMut,
        info: MessageInfo,
        nonce: Uint256,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        let (recipient, amount) = NONCE.load(deps.storage, &nonce.to_be_bytes())?;
        let recipient_address: Address = Address::from_str(recipient.as_str()).unwrap();
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "release_bobby".to_string(),
                vec![Function {
                    name: "release_bobby".to_string(),
                    inputs: vec![
                        Param {
                            name: "recipient".to_string(),
                            kind: ParamType::Address,
                            internal_type: None,
                        },
                        Param {
                            name: "amount".to_string(),
                            kind: ParamType::Uint(256),
                            internal_type: None,
                        },
                        Param {
                            name: "nonce".to_string(),
                            kind: ParamType::Uint(256),
                            internal_type: None,
                        },
                    ],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };

        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: state.job_id,
                    payload: Binary::new(
                        contract
                            .function("release_bobby")
                            .unwrap()
                            .encode_input(&[
                                Token::Address(recipient_address),
                                Token::Uint(Uint::from_big_endian(&amount.to_be_bytes())),
                                Token::Uint(Uint::from_big_endian(&nonce.to_be_bytes())),
                            ])
                            .unwrap(),
                    ),
                },
            }))
            .add_attributes(vec![
                ("action", "rerelease_bobby"),
                ("recipient", &recipient),
                ("amount", &amount.to_string()),
                ("nonce", &nonce.to_string()),
            ]))
    }

    pub fn update_compass(
        deps: DepsMut,
        info: MessageInfo,
        new_compass: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        let new_compass_address: Address = Address::from_str(new_compass.as_str()).unwrap();
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_compass".to_string(),
                vec![Function {
                    name: "update_compass".to_string(),
                    inputs: vec![Param {
                        name: "new_compass".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };

        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: state.job_id,
                    payload: Binary::new(
                        contract
                            .function("update_compass")
                            .unwrap()
                            .encode_input(&[Token::Address(new_compass_address)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_compass"))
    }

    pub fn update_refund_wallet(
        deps: DepsMut,
        info: MessageInfo,
        new_compass: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        let update_refund_wallet_address: Address =
            Address::from_str(new_compass.as_str()).unwrap();
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_refund_wallet".to_string(),
                vec![Function {
                    name: "update_refund_wallet".to_string(),
                    inputs: vec![Param {
                        name: "new_compass".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };

        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: state.job_id,
                    payload: Binary::new(
                        contract
                            .function("update_refund_wallet")
                            .unwrap()
                            .encode_input(&[Token::Address(update_refund_wallet_address)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_refund_wallet"))
    }

    pub fn update_entrance_fee(
        deps: DepsMut,
        info: MessageInfo,
        new_entrance_fee: Uint256,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_entrance_fee".to_string(),
                vec![Function {
                    name: "update_entrance_fee".to_string(),
                    inputs: vec![Param {
                        name: "new_entrance_fee".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };

        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: state.job_id,
                    payload: Binary::new(
                        contract
                            .function("update_entrance_fee")
                            .unwrap()
                            .encode_input(&[Token::Uint(Uint::from_big_endian(
                                &new_entrance_fee.to_be_bytes(),
                            ))])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_entrance_fee"))
    }

    pub fn update_service_fee_collector(
        deps: DepsMut,
        info: MessageInfo,
        new_service_fee_collector: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        let new_service_fee_collector_address: Address =
            Address::from_str(new_service_fee_collector.as_str()).unwrap();
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_service_fee_collector".to_string(),
                vec![Function {
                    name: "update_service_fee_collector".to_string(),
                    inputs: vec![Param {
                        name: "new_service_fee_collector".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };

        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: state.job_id,
                    payload: Binary::new(
                        contract
                            .function("update_service_fee_collector")
                            .unwrap()
                            .encode_input(&[Token::Address(new_service_fee_collector_address)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_service_fee_collector"))
    }

    pub fn update_service_fee(
        deps: DepsMut,
        info: MessageInfo,
        new_service_fee: Uint256,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_service_fee".to_string(),
                vec![Function {
                    name: "update_service_fee".to_string(),
                    inputs: vec![Param {
                        name: "new_service_fee".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };

        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: state.job_id,
                    payload: Binary::new(
                        contract
                            .function("update_service_fee")
                            .unwrap()
                            .encode_input(&[Token::Uint(Uint::from_big_endian(
                                &new_service_fee.to_be_bytes(),
                            ))])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_service_fee"))
    }

    pub fn update_job_id(
        deps: DepsMut,
        info: MessageInfo,
        new_job_id: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        STATE.update(deps.storage, |mut state| -> Result<State, ContractError> {
            state.job_id = new_job_id.clone();
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "update_job_id"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
