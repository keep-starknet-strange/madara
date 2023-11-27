use std::sync::Arc;

use ethers::types::U256;
use sp_blockchain::HeaderBackend;
use starknet_api::state::StateDiff;

use crate::parser::*;

macro_rules! u256_array {
    ($($val:expr),*) => {{
        [$(U256::from_dec_str($val).unwrap()),*]
    }};
}

#[test]
fn test_decode_pre_011_diff_v2() {
    // the test data is from starknet docs: https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/on-chain-data/#data_availability_pre_v0_11_0
    let data = u256_array![
        "2",
        "2472939307328371039455977650994226407024607754063562993856224077254594995194",
        "1336043477925910602175429627555369551262229712266217887481529642650907574765",
        "5",
        "2019172390095051323869047481075102003731246132997057518965927979101413600827",
        "18446744073709551617",
        "5",
        "102",
        "2111158214429736260101797453815341265658516118421387314850625535905115418634",
        "2",
        "619473939880410191267127038055308002651079521370507951329266275707625062498",
        "1471584055184889701471507129567376607666785522455476394130774434754411633091",
        "619473939880410191267127038055308002651079521370507951329266275707625062499",
        "541081937647750334353499719661793404023294520617957763260656728924567461866",
        "2472939307328371039455977650994226407024607754063562993856224077254594995194",
        "1",
        "955723665991825982403667749532843665052270105995360175183368988948217233556",
        "2439272289032330041885427773916021390926903450917097317807468082958581062272",
        "3429319713503054399243751728532349500489096444181867640228809233993992987070",
        "1",
        "5",
        "1110",
        "3476138891838001128614704553731964710634238587541803499001822322602421164873",
        "6",
        "59664015286291125586727181187045849528930298741728639958614076589374875456",
        "600",
        "221246409693049874911156614478125967098431447433028390043893900771521609973",
        "400",
        "558404273560404778508455254030458021013656352466216690688595011803280448030",
        "100",
        "558404273560404778508455254030458021013656352466216690688595011803280448031",
        "200",
        "558404273560404778508455254030458021013656352466216690688595011803280448032",
        "300",
        "1351148242645005540004162531550805076995747746087542030095186557536641755046",
        "500"
    ];

    let state_diff_json = r#"
    {
        "storage_diffs": {
            "0x476cfa27c83ea2498c4fb61972c2b80d2b1cd500986a881ec3c4e5b4f726e3b": {
                "0x5": "0x66"
            },
            "0x4aadf8a572285b2a66d0403e9fac13801df71579aed10cada9a6df1aff0300a": {
                "0x15e9c1d7addfeb3b259d996f8bfd15cb87dc68acc054b26f66aae6ff432c062": "0x340e31649968a352dd3911b2a9f91f5340df2deb32eb3221063fa9a2d7419c3",
                "0x15e9c1d7addfeb3b259d996f8bfd15cb87dc68acc054b26f66aae6ff432c063": "0x1323dd482f0f1fff87e2685adf90b3714eed0e696b9e99785b3cf8ac4015fea"
            },
            "0x577a250e3e38c3898ee30851ca5860b57db0de23c81a5c3fc250a9eb56123fa": {
                "0x21ceba100a6bb320bb902ca4f7c383cf1e24c71675fd1c3312ad07288648094": "0x5649445c6dc6ed99e5aea4f76045a8d15314ec84d760e2e6e90c27168ff5a80"
            },
            "0x794ed19bd70161831b513cf00986a7527faa50b57b825da0539b624a02f61be": {
                "0x5": "0x456"
            },
            "0x7af6cc5951e3dfb3093c6705a63bc494f9d9083b461c79d98302794745a9b49": {
                "0x21c4c5532291099abe40e8d2cb2871c85926c298064c2578ee9d2a882aa740": "0x258",
                "0x7d38956fbf1979f05aaa5e40c985257ce7e8876793335c3c2dea20088a40f5": "0x190",
                "0x13c0bada91d5722019ed80ac13f96247da123b41a08906feae7e19f33b21a1e": "0x64",
                "0x13c0bada91d5722019ed80ac13f96247da123b41a08906feae7e19f33b21a1f": "0xc8",
                "0x13c0bada91d5722019ed80ac13f96247da123b41a08906feae7e19f33b21a20": "0x12c",
                "0x2fcb909b899963f2ad25580a2f9375f9a517769e8770c7cee71f6bcc09027a6": "0x1f4"
            }
        },
        "nonces": {
            "0x476cfa27c83ea2498c4fb61972c2b80d2b1cd500986a881ec3c4e5b4f726e3b": "0x1",
            "0x4aadf8a572285b2a66d0403e9fac13801df71579aed10cada9a6df1aff0300a": "0x0",
            "0x577a250e3e38c3898ee30851ca5860b57db0de23c81a5c3fc250a9eb56123fa": "0x0",
            "0x794ed19bd70161831b513cf00986a7527faa50b57b825da0539b624a02f61be": "0x0",
            "0x7af6cc5951e3dfb3093c6705a63bc494f9d9083b461c79d98302794745a9b49": "0x0"
        },
        "deployed_contracts": {
            "0x577a250e3e38c3898ee30851ca5860b57db0de23c81a5c3fc250a9eb56123fa": "0x2f42c7edbed0fabd97d566fa27674df17bd0f36b888c55fb6d69f5db98201ed"
        },
        "deprecated_declared_classes": {},
		"declared_classes": {},
		"replaced_classes": {}
    }
    "#;

    let state_diff_from_json = serde_json::from_str::<StateDiff>(state_diff_json).unwrap();
    let state_diff_from_parser = decode_pre_011_diff(&mut data.to_vec(), false).unwrap();

    assert_eq!(state_diff_from_parser, state_diff_from_json);
}

#[test]
fn test_decode_011_diff_v2() {
    // the test data is from starknet docs: https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/on-chain-data/#data_availability_post_v0_11_0
    let data = u256_array![
        "1",
        "2019172390095051323869047481075102003731246132997057518965927979101413600827",
        "18446744073709551617",
        "100",
        "200",
        "1",
        "1351148242645005540004162531550805076995747746087542030095186557536641755046",
        "558404273560404778508455254030458021013656352466216690688595011803280448032"
    ];

    let state_diff_json = r#"
    {
        "storage_diffs": {
			"0x476cfa27c83ea2498c4fb61972c2b80d2b1cd500986a881ec3c4e5b4f726e3b": {
                "0x64": "0xc8"
			}
		},
		"nonces": {
			"0x476cfa27c83ea2498c4fb61972c2b80d2b1cd500986a881ec3c4e5b4f726e3b": "0x1"
		},
		"declared_classes": {
			"0x2fcb909b899963f2ad25580a2f9375f9a517769e8770c7cee71f6bcc09027a6": [
                "0x13c0bada91d5722019ed80ac13f96247da123b41a08906feae7e19f33b21a20",
                {"sierra_program":[],"entry_point_by_type":{},"abi":""}
            ]
		},
        "deprecated_declared_classes": {},
		"deployed_contracts": {},
		"replaced_classes": {}
    }
    "#;

    let (client, _) = crate::tests::sync::create_test_client();
    let client = Arc::new(client);
    let block_hash = client.info().best_hash;

    let state_diff_from_parser = decode_011_diff(&mut data.to_vec(), block_hash, client).unwrap();
    let state_diff_from_json = serde_json::from_str::<StateDiff>(state_diff_json).unwrap();

    assert_eq!(state_diff_from_json, state_diff_from_parser);
}
