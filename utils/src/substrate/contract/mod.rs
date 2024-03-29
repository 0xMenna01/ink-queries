// Copyright (C) 2022-2023 <company>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod builder;
mod error;
pub mod ink;
pub mod query;

use self::{
    error::ErrorVariant,
    ink::InkMeta,
    query::{Query, QueryBuilder},
};

use super::{Nonce, PairSigner};
use anyhow::Result;
use contract_transcode::{ContractMessageTranscoder, Value};

pub struct ContractInstance {
    pub signer: PairSigner,
    meta: InkMeta,
}

impl ContractInstance {
    pub fn new(meta: InkMeta, signer: PairSigner) -> Self {
        Self { meta, signer }
    }

    /// Allows to call a substrate based ink smart contract
    /// The nonce has to be provided if a phala smart contract is being called
    pub fn call_msg(
        &self,
        msg_name: &str,
        args: Vec<String>,
        nonce: Option<Nonce>,
    ) -> Result<Value, ErrorVariant> {
        let transcoder = self.get_transcoder()?;

        let call_data = transcoder.encode(msg_name, &args)?;

        let query = match (
            self.meta.ink_contract_id.clone(),
            self.meta.phala_contract_id,
        ) {
            (Some(ink_id), None) => Query::InkQuery(call_data, ink_id),
            (None, Some(phala_id)) => {
                let nonce = nonce.expect("Must provide nonce to call phala");
                Query::PhalaQuery(call_data, phala_id, nonce)
            }
            _ => {
                return Err(ErrorVariant::from(
                    "Contract Id Error: must provide only one contract address",
                ))
            }
        };

        let contract_query = QueryBuilder::new(msg_name.to_string(), transcoder)
            .query(query)
            .build();

        contract_query.call(self.meta.url.clone(), &self.signer)
    }

    fn get_transcoder(&self) -> Result<ContractMessageTranscoder> {
        let artifacts = self.meta.contract_artifacts()?;
        let transcoder = artifacts.contract_transcoder()?;
        Ok(transcoder)
    }
}
