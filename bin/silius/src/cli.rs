use crate::utils::{parse_address, parse_u256, parse_uopool_mode};
use clap::{ArgAction::Set, Parser};
use ethers::types::{Address, U256};
use silius_primitives::UoPoolMode;
use std::net::SocketAddr;

#[derive(Clone, Debug, Parser, PartialEq)]
pub struct UoPoolServiceOpts {
    #[clap(long, default_value = "127.0.0.1:3001")]
    pub uopool_grpc_listen_address: SocketAddr,

    #[clap(long, value_parser=parse_u256, default_value = "1")]
    pub min_stake: U256,

    #[clap(long, value_parser=parse_u256, default_value = "0")]
    pub min_unstake_delay: U256,

    #[clap(long, value_parser=parse_u256, default_value = "0")]
    pub min_priority_fee_per_gas: U256,

    #[clap(long, value_delimiter=',', value_parser = parse_address)]
    pub whitelist: Vec<Address>,

    #[clap(long, default_value = "standard", value_parser=parse_uopool_mode)]
    pub uo_pool_mode: UoPoolMode,
}

#[derive(Clone, Debug, Parser, PartialEq)]
pub struct BundlerServiceOpts {
    #[clap(long, value_parser=parse_address)]
    pub beneficiary: Address,

    #[clap(long, default_value = "600", value_parser=parse_u256)]
    pub gas_factor: U256,

    #[clap(long, default_value = "1", value_parser=parse_u256)]
    pub min_balance: U256,

    #[clap(long, default_value = "127.0.0.1:3002")]
    pub bundler_grpc_listen_address: SocketAddr,

    #[clap(long, default_value = "10")]
    pub bundle_interval: u64,
}

#[derive(Clone, Debug, Parser, PartialEq)]
pub struct RpcServiceOpts {
    #[clap(long, default_value = "127.0.0.1:3000")]
    pub rpc_listen_address: String,

    #[clap(long, value_delimiter=',', default_value = "eth", value_parser = ["eth", "debug", "web3"])]
    pub rpc_api: Vec<String>,

    #[clap(long, action = Set, default_value = "true")]
    pub http: bool,

    #[clap(long, action = Set, default_value = "true")]
    pub ws: bool,

    #[clap(long, value_delimiter = ',', default_value = "*")]
    pub cors_domain: Vec<String>,

    #[clap(long)]
    pub build_fb_signer: Option<bool>,

    #[clap(long, default_value = "eth-client")]
    pub send_bundle_mode: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        net::{IpAddr, Ipv4Addr},
        str::FromStr,
    };

    #[test]
    fn bundler_opts() {
        let args = vec![
            "bundleropts",
            "--beneficiary",
            "0x690B9A9E9aa1C9dB991C7721a92d351Db4FaC990",
            "--gas-factor",
            "600",
            "--min-balance",
            "1",
            "--bundler-grpc-listen-address",
            "127.0.0.1:3002",
            "--bundle-interval",
            "10",
        ];
        assert_eq!(
            BundlerServiceOpts {
                beneficiary: Address::from_str("0x690B9A9E9aa1C9dB991C7721a92d351Db4FaC990")
                    .unwrap(),
                gas_factor: U256::from(600),
                min_balance: U256::from(1),
                bundler_grpc_listen_address: SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    3002
                ),
                bundle_interval: 10,
            },
            BundlerServiceOpts::try_parse_from(args).unwrap()
        );
    }

    #[test]
    fn rpc_service_opts() {
        let args = vec![
            "rpcserviceopts",
            "--rpc-listen-address",
            "127.0.0.1:1234",
            "--rpc-api",
            "eth,debug,web3",
            "--http",
            "true",
            "--ws",
            "false",
            "--cors-domain",
            "127.0.0.1:4321",
        ];
        assert_eq!(
            RpcServiceOpts {
                rpc_listen_address: String::from("127.0.0.1:1234"),
                rpc_api: vec![
                    String::from("eth"),
                    String::from("debug"),
                    String::from("web3")
                ],
                http: true,
                ws: false,
                cors_domain: vec![String::from("127.0.0.1:4321")],
            },
            RpcServiceOpts::try_parse_from(args).unwrap()
        );
    }

    #[test]
    fn rpc_service_opts_when_no_http_and_ws_flag() {
        let args = vec![
            "rpcserviceopts",
            "--rpc-listen-address",
            "127.0.0.1:1234",
            "--rpc-api",
            "eth,debug,web3",
            "--cors-domain",
            "127.0.0.1:4321",
        ];
        assert_eq!(
            RpcServiceOpts {
                rpc_listen_address: String::from("127.0.0.1:1234"),
                rpc_api: vec![
                    String::from("eth"),
                    String::from("debug"),
                    String::from("web3")
                ],
                http: true,
                ws: true,
                cors_domain: vec![String::from("127.0.0.1:4321")],
            },
            RpcServiceOpts::try_parse_from(args).unwrap()
        );
    }
}
