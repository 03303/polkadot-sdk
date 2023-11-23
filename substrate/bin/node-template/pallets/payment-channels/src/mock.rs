// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Test environment for Payment Channels pallet.

use super::*;
use crate as pallet_payment_channels;

use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, ConstU64},
};
use sp_core::{H256, Pair};
use sp_keystore::{testing::MemoryKeystore, KeystoreExt};
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature,
};

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		PaymentChannels: pallet_payment_channels::{Pallet, Call, Storage, Event<T>},
	}
);

pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
pub type Balance = u64;

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type MaxHolds = ();
}

/// Money matters.
pub mod currency {
	use super::Balance;
	pub const MILLICENTS: Balance = 1_000_000_000;
	pub const UNIT: Balance = 1_000 * MILLICENTS;
}

parameter_types! {
	pub const OrganizationDeposit: Balance = currency::UNIT;
	pub const ServiceDeposit: Balance = 10 * currency::MILLICENTS;
	pub const PaymentChannelsPalletId: PalletId = PalletId(*b"py/paych");
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = PaymentChannelsPalletId;
	type MaxOrganizationMembers = ConstU32<8>;
	type MaxNameLength = ConstU32<64>;
	type MaxMetadataLength = ConstU32<1024>;
	type Currency = Balances;
	type OrganizationDeposit = OrganizationDeposit;
	type ServiceDeposit = ServiceDeposit;
	type Signature = Signature;
	type Signer = AccountPublic;
	type WeightInfo = ();
}

pub fn get_account(s: &str) -> (sp_core::sr25519::Pair, AccountId) {
	let pair = sp_core::sr25519::Pair::from_string(s, None).unwrap();
	(pair.clone(), pair.public().into())
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {

	let balances: Vec<(AccountId, Balance)> = vec![
		(get_account("//Alice").1, 1_000_000 * currency::UNIT),
		(get_account("//Bob").1, 1_000_000 * currency::UNIT),
		(get_account("//Charlie").1, 1_000_000 * currency::UNIT),
		(get_account("//Dave").1, 1_000_000 * currency::UNIT),
		(get_account("//Eve").1, 1_000_000 * currency::UNIT),
	];

	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt::new(MemoryKeystore::new()));
	ext.execute_with(|| System::set_block_number(1));
	ext
}
