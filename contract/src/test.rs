mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::*;
    use crate::state::{Config, Recipient};
    use cosmwasm_std::{
        coins,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, CosmosMsg, Uint128, WasmMsg,
    };
    use cosmwasm_std::{from_json, to_json_binary, Api};
    use cw20::Cw20ExecuteMsg;
    const TEST_DENOM: &str = "ugtb";

    #[test]
    fn test_init() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            denom: crate::state::Denom::Cw20(Addr::unchecked("0x01")),
            admin: Some("0x02".to_string()),
            auto_release_time: 100,
            max_recipient: 1024,
        };

        let info = mock_info("admin", &coins(0, TEST_DENOM.to_string()));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.attributes.len(), 1);
    }

    #[test]
    fn test_add_payment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            denom: crate::state::Denom::Cw20(Addr::unchecked("0x01")),
            admin: Some("0x02".to_string()),
            auto_release_time: 100,
            max_recipient: 1024,
        };

        let info = mock_info("admin", &coins(0, TEST_DENOM.to_string()));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let add_payment = ExecuteMsg::AddPaymentChan {
            sender_pubkey_hash: "sender_pubkey_hash".to_string(),
            recipients: vec![("recipient_pubkey_hash".to_string(), 100, 10000)],
        };

        let msg = ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: "sender".to_string(),
            amount: Uint128::new(10000),
            msg: to_json_binary(&add_payment).unwrap(),
        });

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 1);
    }

    #[test]
    fn test_cashing() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            denom: crate::state::Denom::Cw20(Addr::unchecked("0x01")),
            admin: Some("0x02".to_string()),
            auto_release_time: 100,
            max_recipient: 1024,
        };

        let info = mock_info("admin", &coins(0, TEST_DENOM.to_string()));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        // face_value = 100; total_amount = 10000;
        let add_payment = ExecuteMsg::AddPaymentChan {
            sender_pubkey_hash: "sender_pubkey_hash".to_string(),
            recipients: vec![("recipient_pubkey_hash1".to_string(), 100, 10000)],
        };

        let msg = ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: "sender".to_string(),
            amount: Uint128::new(10000),
            msg: to_json_binary(&add_payment).unwrap(),
        });

        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes.len(), 1);

        let add_payment = ExecuteMsg::AddPaymentChan {
            sender_pubkey_hash: "sender_pubkey_hash".to_string(),
            recipients: vec![("recipient_pubkey_hash2".to_string(), 200, 20000)],
        };

        let msg = ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: "sender".to_string(),
            amount: Uint128::new(20000),
            msg: to_json_binary(&add_payment).unwrap(),
        });

        let _ = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::PaymentChan {
                sender_pubkey_hash: "sender_pubkey_hash".to_string(),
                recipient_pubkey_hash: None,
                page: None,
                size: None,
            },
        );

        let payment_chan_resp: Option<Vec<Recipient>> = from_json(res.unwrap()).unwrap();

        assert_eq!(payment_chan_resp.unwrap().len(), 2);

        let msg = ExecuteMsg::Cashing {
            recipient_pubkey_hash: "recipient_pubkey_hash1".to_string(),
            cheques: vec![(
                PaymentCheque {
                    sender_pubkey_hash: "sender_pubkey_hash".to_string(),
                    sender_commitment: vec![],
                    recipient_pubkey_hash: "recipient_pubkey_hash1".to_string(),
                    recipient_commitment: vec![],
                    value: None,
                    nonce: 1,
                },
                PaymentCheque {
                    sender_pubkey_hash: "sender_pubkey_hash".to_string(),
                    sender_commitment: vec![],
                    recipient_pubkey_hash: "recipient_pubkey_hash1".to_string(),
                    recipient_commitment: vec![],
                    value: None,
                    nonce: 3,
                },
            )],
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 1);
        // 2 cheque amount = 2 * 100 = 200
        let refund_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "0x01".to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "admin".to_string(),
                amount: Uint128::new(300),
            })
            .unwrap(),
            funds: vec![],
        });
        assert_eq!(res.messages[0].msg, refund_msg);
    }

    #[test]
    fn test_update_config() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            denom: crate::state::Denom::Cw20(Addr::unchecked("0x01")),
            admin: Some("0x02".to_string()),
            auto_release_time: 100,
            max_recipient: 1024,
        };

        let info = mock_info("admin", &coins(0, TEST_DENOM.to_string()));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let mut config_res: Config =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(
            config_res.denom,
            crate::state::Denom::Cw20(Addr::unchecked("0x01"))
        );
        assert_eq!(
            config_res.owner,
            deps.api.addr_canonicalize("admin").unwrap()
        );
        assert_eq!(config_res.auto_release_time, 100);
        assert_eq!(config_res.max_recipient, 1024);

        let update_config_msg = ExecuteMsg::UpdateConfig {
            owner: Some("new_owner".to_string()),
            auto_release_time: Some(2000000),
            max_recipient: Some(10),
        };

        let res = execute(deps.as_mut(), mock_env(), info, update_config_msg);
        config_res =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(res.unwrap().attributes.len(), 1);
        assert_eq!(
            config_res.denom,
            crate::state::Denom::Cw20(Addr::unchecked("0x01"))
        );
        assert_eq!(config_res.auto_release_time, 2000000);
        assert_eq!(config_res.max_recipient, 10);
        assert_eq!(
            config_res.owner,
            deps.api.addr_canonicalize("new_owner").unwrap()
        );
        let not_admin_info = mock_info("admin", &coins(0, TEST_DENOM.to_string()));

        let update_config_msg = ExecuteMsg::UpdateConfig {
            owner: Some("new_owner".to_string()),
            auto_release_time: Some(2000000),
            max_recipient: Some(10),
        };

        let res = execute(deps.as_mut(), mock_env(), not_admin_info, update_config_msg);

        assert!(res.is_err());
    }

    #[test]
    fn test_close_payment_chan() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            denom: crate::state::Denom::Cw20(Addr::unchecked("0x01")),
            admin: Some("0x02".to_string()),
            auto_release_time: 100,
            max_recipient: 1024,
        };

        let info = mock_info("admin", &coins(0, TEST_DENOM.to_string()));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        // face_value = 100; total_amount = 10000;
        let add_payment = ExecuteMsg::AddPaymentChan {
            sender_pubkey_hash: "sender_pubkey_hash".to_string(),
            recipients: vec![("recipient_pubkey_hash1".to_string(), 100, 10000)],
        };

        let msg = ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: "sender".to_string(),
            amount: Uint128::new(10000),
            msg: to_json_binary(&add_payment).unwrap(),
        });

        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes.len(), 1);

        let close_msg = ExecuteMsg::ClosePaymentChan {
            sender_pubkey_hash: "sender_pubkey_hash".to_string(),
            sender_commitment: vec![],
            recipients: vec![("recipient_pubkey_hash1".to_string(), vec![])],
        };

        let res = execute(deps.as_mut(), mock_env(), info.clone(), close_msg).unwrap();
        assert_eq!(res.messages.len(), 1);
    }
}
