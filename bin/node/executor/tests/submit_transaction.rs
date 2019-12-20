// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use node_runtime::{
	Call, Runtime, SubmitTransaction,
};
use sp_application_crypto::AppKey;
use sp_core::testing::KeyStore;
use sp_core::traits::KeystoreExt;
use sp_core::offchain::{
	TransactionPoolExt,
	testing::TestTransactionPoolExt,
};
use frame_system::offchain::{SubmitSignedTransaction, SubmitUnsignedTransaction};
use pallet_im_online::sr25519::AuthorityPair as Key;

mod common;
use self::common::*;

#[test]
fn should_submit_unsigned_transaction() {
	let mut t = new_test_ext(COMPACT_CODE, false);
	let (pool, state) = TestTransactionPoolExt::new();
	t.register_extension(TransactionPoolExt::new(pool));

	t.execute_with(|| {
		let signature = Default::default();
		let heartbeat_data = pallet_im_online::Heartbeat {
			block_number: 1,
			network_state: Default::default(),
			session_index: 1,
			authority_index: 0,
		};

		let call = pallet_im_online::Call::heartbeat(heartbeat_data, signature);
		<SubmitTransaction as SubmitUnsignedTransaction<Runtime, Call>>
			::submit_unsigned(call)
			.unwrap();

		assert_eq!(state.read().transactions.len(), 1)
	});
}

const PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";

#[test]
fn should_submit_signed_transaction() {
	let mut t = new_test_ext(COMPACT_CODE, false);
	let (pool, state) = TestTransactionPoolExt::new();
	t.register_extension(TransactionPoolExt::new(pool));

	let keystore = KeyStore::new();
	keystore.write().sr25519_generate_new(Key::ID, Some(&format!("{}/hunter1", PHRASE))).unwrap();
	keystore.write().sr25519_generate_new(Key::ID, Some(&format!("{}/hunter2", PHRASE))).unwrap();
	keystore.write().sr25519_generate_new(Key::ID, Some(&format!("{}/hunter3", PHRASE))).unwrap();
	t.register_extension(KeystoreExt(keystore));

	t.execute_with(|| {
		let keys = <SubmitTransaction as SubmitSignedTransaction<Runtime, Call>>
			::find_all_local_keys();
		assert_eq!(keys.len(), 3, "Missing keys: {:?}", keys);

		let can_sign = <SubmitTransaction as SubmitSignedTransaction<Runtime, Call>>
			::can_sign();
		assert!(can_sign, "Since there are keys, `can_sign` should return true");

		let call = pallet_balances::Call::transfer(Default::default(), Default::default());
		let results =
			<SubmitTransaction as SubmitSignedTransaction<Runtime, Call>>::submit_signed(call);

		let len = results.len();
		assert_eq!(len, 3);
		assert_eq!(results.into_iter().filter_map(|x| x.1.ok()).count(), len);
		assert_eq!(state.read().transactions.len(), len);
	});
}

#[test]
fn should_submit_signed_twice_from_the_same_account() {
	let mut t = new_test_ext(COMPACT_CODE, false);
	let (pool, state) = TestTransactionPoolExt::new();
	t.register_extension(TransactionPoolExt::new(pool));

	let keystore = KeyStore::new();
	keystore.write().sr25519_generate_new(Key::ID, Some(&format!("{}/hunter1", PHRASE))).unwrap();
	t.register_extension(KeystoreExt(keystore));

	t.execute_with(|| {
		let call = pallet_balances::Call::transfer(Default::default(), Default::default());
		let results =
			<SubmitTransaction as SubmitSignedTransaction<Runtime, Call>>::submit_signed(call);

		let len = results.len();
		assert_eq!(len, 1);
		assert_eq!(results.into_iter().filter_map(|x| x.1.ok()).count(), len);
		assert_eq!(state.read().transactions.len(), len);

		// submit another one from the same account. The nonce should be incremented.
				let call = pallet_balances::Call::transfer(Default::default(), Default::default());
		let results =
			<SubmitTransaction as SubmitSignedTransaction<Runtime, Call>>::submit_signed(call);

		let len = results.len();
		assert_eq!(len, 1);
		assert_eq!(results.into_iter().filter_map(|x| x.1.ok()).count(), len);
		assert_eq!(state.read().transactions.len(), len + 1);

		// now check that the transactions are not equal
		let s = state.read();
		assert!(
			s.transactions[0] != s.transactions[1],
			"Transactions should have different nonces."
		);
	});
}
