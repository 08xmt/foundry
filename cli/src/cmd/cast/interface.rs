use crate::opts::EtherscanOpts;
use cast::{AbiPath, SimpleCast};
use clap::Parser;
use foundry_common::fs;
use foundry_config::Config;
use std::path::{Path, PathBuf};

/// CLI arguments for `cast interface`.
#[derive(Debug, Clone, Parser)]
pub struct InterfaceArgs {
    #[clap(
        help = "The contract address, or the path to an ABI file.",
        long_help = r#"The contract address, or the path to an ABI file.

If an address is specified, then the ABI is fetched from Etherscan."#,
        value_name = "PATH_OR_ADDRESS"
    )]
    path_or_address: String,

    #[clap(long, short, help = "The name to use for the generated interface", value_name = "NAME")]
    name: Option<String>,

    #[clap(
        long,
        short,
        default_value = "^0.8.10",
        help = "Solidity pragma version.",
        value_name = "VERSION"
    )]
    pragma: String,

    #[clap(
        short,
        help = "The path to the output file.",
        long_help = "The path to the output file. If not specified, the interface will be output to stdout.",
        value_name = "PATH"
    )]
    output_location: Option<PathBuf>,

    #[clap(flatten)]
    etherscan: EtherscanOpts,
}

impl InterfaceArgs {
    pub async fn run(self) -> eyre::Result<()> {
        let InterfaceArgs { path_or_address, name, pragma, output_location, etherscan } = self;
        let config = Config::from(&etherscan);
        let chain = config.chain_id.unwrap_or_default();
        let source = if Path::new(&path_or_address).exists() {
            AbiPath::Local { path: path_or_address, name }
        } else {
            let api_key = config.get_etherscan_api_key(Some(chain)).unwrap_or_default();
            let chain = chain.named()?;
            AbiPath::Etherscan { chain, api_key, address: path_or_address.parse()? }
        };
        let interfaces = SimpleCast::generate_interface(source).await?;

        // put it all together
        let pragma = format!("pragma solidity {pragma};");
        let interfaces =
            interfaces.iter().map(|iface| iface.source.to_string()).collect::<Vec<_>>().join("\n");
        let res = format!("{pragma}\n\n{interfaces}");

        // print or write to file
        if let Some(loc) = output_location {
            fs::create_dir_all(loc.parent().unwrap())?;
            fs::write(&loc, res)?;
            println!("Saved interface at {}", loc.display());
        } else {
            println!("{res}");
        }
        Ok(())
    }
}
