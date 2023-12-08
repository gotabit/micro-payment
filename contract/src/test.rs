mod tests {
    use crate::contract::{execute, instantiate};
    use crate::msg::*;
    use cosmwasm_std::to_json_binary;
    use cosmwasm_std::{
        coins,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, CosmosMsg, Uint128, WasmMsg,
    };
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
            recipients: vec![("recipient_pubkey_hash".to_string(), 100, 10000)],
        };

        let msg = ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: "sender".to_string(),
            amount: Uint128::new(10000),
            msg: to_json_binary(&add_payment).unwrap(),
        });

        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes.len(), 1);

        let msg = ExecuteMsg::Cashing {
            recipient_pubkey_hash: "recipient_pubkey_hash".to_string(),
            cheques: vec![(
                PaymentCheque {
                    sender_pubkey_hash: "sender_pubkey_hash".to_string(),
                    sender_commitment: vec![],
                    recipient_pubkey_hash: "recipient_pubkey_hash".to_string(),
                    recipient_commitment: vec![],
                    value: None,
                    nonce: 1,
                },
                PaymentCheque {
                    sender_pubkey_hash: "sender_pubkey_hash".to_string(),
                    sender_commitment: vec![],
                    recipient_pubkey_hash: "recipient_pubkey_hash".to_string(),
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
}
