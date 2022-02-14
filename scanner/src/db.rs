use tokio_postgres::Client;

use crate::Result;
use module::db::block::Block as ModuleBlock;

pub async fn save(block: ModuleBlock, client: &Client) -> Result<()> {
    let _ = client
        .execute(
            "INSERT INTO block VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT(height) DO UPDATE SET block_id=$1, size=$3, time=$4, app_hash=$5, proposer=$6",
            &[
                &block.block_id,
                &block.height,
                &block.size,
                &block.timestamp,
                &block.app_hash,
                &block.proposer,
            ],
        )
        .await?;

    for tx in block.txs {
        let _ = client
            .execute(
                "INSERT INTO transaction VALUES ($1, $2, 0, $3, $4, $5) ON CONFLICT(txid) DO UPDATE SET ty=0, block_id=$2, value=$3, code=$4, log=$5",
                &[&tx.txid, &tx.block_id, &tx.value, &tx.code, &tx.log],
            )
            .await?;
    }

    for tx in block.evm_txs {
        let _ = client
            .execute(
                "INSERT INTO transaction VALUES ($1, $2, 1, $3, $4, $5) ON CONFLICT(txid) DO UPDATE SET ty=1, block_id=$2, value=$3, code=$4, log=$5",
                &[&tx.txid, &tx.block_id, &tx.value, &tx.code, &tx.log],
            )
            .await?;
    }

    for v in block.validators {
        let _ = client
            .execute(
                "INSERT INTO validators VALUES ($1, 0, $2) ON CONFLICT(address) DO UPDATE SET pubkey_type=0, pubkey=$2",
                &[&v.address, &v.pub_key.value],
            )
            .await?;

        let power: i64 = v.power.try_into()?;

        let _ = client
            .execute(
                "INSERT INTO block_generation VALUES($1, $2, $3, $4, $5, $6) ON CONFLICT(height, address) DO UPDATE SET power=$3, priority=$4, signature=$5, time=$6",
                &[
                    &block.height,
                    &v.address,
                    &power,
                    &v.priority,
                    &v.signature,
                    &v.timestamp,
                ],
            )
            .await?;
    }

    client
        .execute(
            "INSERT INTO last_height VALUES($1, $2) ON CONFLICT(tip) DO UPDATE SET height=$2",
            &[&"tip".to_string(), &block.height],
        )
        .await?;

    Ok(())
}
