//! Tests for the module.

use frame_support::{
	assert_err, assert_noop, assert_ok,
	traits::{Currency, ExistenceRequirement::AllowDeath, ReservableCurrency},
};
use pallet_transaction_payment::ChargeTransactionPayment;
use sp_runtime::traits::SignedExtension;
use system::RawOrigin;

use crate::{mock::*, *};
use darwinia_support::{LockIdentifier, LockableCurrency, NormalLock, WithdrawLock, WithdrawReason, WithdrawReasons};

const ID_1: LockIdentifier = *b"1       ";
const ID_2: LockIdentifier = *b"2       ";
const ID_3: LockIdentifier = *b"3       ";

#[test]
fn basic_locking_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build()
		.execute_with(|| {
			assert_eq!(Ring::free_balance(&1), 10);
			Ring::set_lock(
				ID_1,
				&1,
				WithdrawLock::Normal(NormalLock {
					amount: 9,
					until: u64::max_value(),
				}),
				WithdrawReasons::all(),
			);
			assert_noop!(
				<Ring as Currency<_>>::transfer(&1, &2, 5, AllowDeath),
				Error::<Test, _>::LiquidityRestrictions
			);
		});
}

#[test]
fn partial_locking_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build()
		.execute_with(|| {
			Ring::set_lock(
				ID_1,
				&1,
				WithdrawLock::Normal(NormalLock {
					amount: 5,
					until: u64::max_value(),
				}),
				WithdrawReasons::all(),
			);
			assert_ok!(<Ring as Currency<_>>::transfer(&1, &2, 1, AllowDeath));
		});
}

#[test]
fn lock_removal_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build()
		.execute_with(|| {
			Ring::set_lock(
				ID_1,
				&1,
				WithdrawLock::Normal(NormalLock {
					amount: u64::max_value(),
					until: u64::max_value(),
				}),
				WithdrawReasons::all(),
			);
			Ring::remove_lock(ID_1, &1);
			assert_ok!(<Ring as Currency<_>>::transfer(&1, &2, 1, AllowDeath));
		});
}

#[test]
fn lock_replacement_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build()
		.execute_with(|| {
			Ring::set_lock(
				ID_1,
				&1,
				WithdrawLock::Normal(NormalLock {
					amount: u64::max_value(),
					until: u64::max_value(),
				}),
				WithdrawReasons::all(),
			);
			Ring::set_lock(
				ID_1,
				&1,
				WithdrawLock::Normal(NormalLock {
					amount: 5,
					until: u64::max_value(),
				}),
				WithdrawReasons::all(),
			);
			assert_ok!(<Ring as Currency<_>>::transfer(&1, &2, 1, AllowDeath));
		});
}

#[test]
fn double_locking_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build()
		.execute_with(|| {
			Ring::set_lock(
				ID_1,
				&1,
				WithdrawLock::Normal(NormalLock {
					amount: 5,
					until: u64::max_value(),
				}),
				WithdrawReasons::all(),
			);
			Ring::set_lock(
				ID_2,
				&1,
				WithdrawLock::Normal(NormalLock {
					amount: 5,
					until: u64::max_value(),
				}),
				WithdrawReasons::all(),
			);
			assert_ok!(<Ring as Currency<_>>::transfer(&1, &2, 1, AllowDeath));
		});
}

#[test]
fn combination_locking_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build()
		.execute_with(|| {
			Ring::set_lock(
				ID_1,
				&1,
				WithdrawLock::Normal(NormalLock {
					amount: u64::max_value(),
					until: 0,
				}),
				WithdrawReasons::none(),
			);
			Ring::set_lock(
				ID_2,
				&1,
				WithdrawLock::Normal(NormalLock {
					amount: 0,
					until: u64::max_value(),
				}),
				WithdrawReasons::none(),
			);
			Ring::set_lock(
				ID_3,
				&1,
				WithdrawLock::Normal(NormalLock { amount: 0, until: 0 }),
				WithdrawReasons::all(),
			);
			assert_ok!(<Ring as Currency<_>>::transfer(&1, &2, 1, AllowDeath));
		});
}

#[test]
fn lock_reasons_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build()
		.execute_with(|| {
			Ring::set_lock(
				ID_1,
				&1,
				WithdrawLock::Normal(NormalLock {
					amount: 10,
					until: u64::max_value(),
				}),
				WithdrawReason::Transfer.into(),
			);
			assert_noop!(
				<Ring as Currency<_>>::transfer(&1, &2, 1, AllowDeath),
				Error::<Test, _>::LiquidityRestrictions
			);
			assert_ok!(<Ring as ReservableCurrency<_>>::reserve(&1, 1));
			// NOTE: this causes a fee payment.
			assert!(<ChargeTransactionPayment<Test> as SignedExtension>::pre_dispatch(
				ChargeTransactionPayment::from(1),
				&1,
				CALL,
				info_from_weight(1),
				0,
			)
			.is_ok());

			Ring::set_lock(
				ID_1,
				&1,
				WithdrawLock::Normal(NormalLock {
					amount: 10,
					until: u64::max_value(),
				}),
				WithdrawReason::Reserve.into(),
			);
			assert_ok!(<Ring as Currency<_>>::transfer(&1, &2, 1, AllowDeath));
			assert_noop!(
				<Ring as ReservableCurrency<_>>::reserve(&1, 1),
				Error::<Test, _>::LiquidityRestrictions
			);
			assert!(<ChargeTransactionPayment<Test> as SignedExtension>::pre_dispatch(
				ChargeTransactionPayment::from(1),
				&1,
				CALL,
				info_from_weight(1),
				0,
			)
			.is_ok());

			Ring::set_lock(
				ID_1,
				&1,
				WithdrawLock::Normal(NormalLock {
					amount: 10,
					until: u64::max_value(),
				}),
				WithdrawReason::TransactionPayment.into(),
			);
			assert_ok!(<Ring as Currency<_>>::transfer(&1, &2, 1, AllowDeath));
			assert_ok!(<Ring as ReservableCurrency<_>>::reserve(&1, 1));
			assert!(<ChargeTransactionPayment<Test> as SignedExtension>::pre_dispatch(
				ChargeTransactionPayment::from(1),
				&1,
				CALL,
				info_from_weight(1),
				0,
			)
			.is_err());
		});
}

#[test]
fn lock_block_number_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build()
		.execute_with(|| {
			Ring::set_lock(
				ID_1,
				&1,
				WithdrawLock::Normal(NormalLock { amount: 10, until: 2 }),
				WithdrawReasons::all(),
			);
			assert_noop!(
				<Ring as Currency<_>>::transfer(&1, &2, 1, AllowDeath),
				Error::<Test, _>::LiquidityRestrictions
			);

			System::set_block_number(2);
			assert_ok!(<Ring as Currency<_>>::transfer(&1, &2, 1, AllowDeath));
		});
}

#[test]
fn default_indexing_on_new_accounts_should_not_work2() {
	ExtBuilder::default()
		.existential_deposit(10)
		.creation_fee(50)
		.monied(true)
		.build()
		.execute_with(|| {
			assert_eq!(Ring::is_dead_account(&5), true); // account 5 should not exist
											 // ext_deposit is 10, value is 9, not satisfies for ext_deposit
			assert_noop!(
				Ring::transfer(Some(1).into(), 5, 9),
				Error::<Test, _>::ExistentialDeposit,
			);
			assert_eq!(Ring::is_dead_account(&5), true); // account 5 should not exist
			assert_eq!(Ring::free_balance(&1), 100);
		});
}

#[test]
fn reserved_balance_should_prevent_reclaim_count() {
	ExtBuilder::default()
		.existential_deposit(256 * 1)
		.monied(true)
		.build()
		.execute_with(|| {
			System::inc_account_nonce(&2);
			assert_eq!(Ring::is_dead_account(&2), false);
			assert_eq!(Ring::is_dead_account(&5), true);
			assert_eq!(Ring::total_balance(&2), 256 * 20);

			assert_ok!(Ring::reserve(&2, 256 * 19 + 1)); // account 2 becomes mostly reserved
			assert_eq!(Ring::free_balance(&2), 0); // "free" account deleted."
			assert_eq!(Ring::total_balance(&2), 256 * 20); // reserve still exists.
			assert_eq!(Ring::is_dead_account(&2), false);
			assert_eq!(System::account_nonce(&2), 1);

			// account 4 tries to take index 1 for account 5.
			assert_ok!(Ring::transfer(Some(4).into(), 5, 256 * 1 + 0x69));
			assert_eq!(Ring::total_balance(&5), 256 * 1 + 0x69);
			assert_eq!(Ring::is_dead_account(&5), false);

			assert!(Ring::slash(&2, 256 * 19 + 2).1.is_zero()); // account 2 gets slashed
													// "reserve" account reduced to 255 (below ED) so account deleted
			assert_eq!(Ring::total_balance(&2), 0);
			assert_eq!(System::account_nonce(&2), 0); // nonce zero
			assert_eq!(Ring::is_dead_account(&2), true);

			// account 4 tries to take index 1 again for account 6.
			assert_ok!(Ring::transfer(Some(4).into(), 6, 256 * 1 + 0x69));
			assert_eq!(Ring::total_balance(&6), 256 * 1 + 0x69);
			assert_eq!(Ring::is_dead_account(&6), false);
		});
}

#[test]
fn reward_should_work() {
	ExtBuilder::default().monied(true).build().execute_with(|| {
		assert_eq!(Ring::total_balance(&1), 10);
		assert_ok!(Ring::deposit_into_existing(&1, 10).map(drop));
		assert_eq!(Ring::total_balance(&1), 20);
		assert_eq!(<TotalIssuance<Test>>::get(), 120);
	});
}

#[test]
fn dust_account_removal_should_work() {
	ExtBuilder::default()
		.existential_deposit(100)
		.monied(true)
		.build()
		.execute_with(|| {
			System::inc_account_nonce(&2);
			assert_eq!(System::account_nonce(&2), 1);
			assert_eq!(Ring::total_balance(&2), 2000);

			assert_ok!(Ring::transfer(Some(2).into(), 5, 1901)); // index 1 (account 2) becomes zombie
			assert_eq!(Ring::total_balance(&2), 0);
			assert_eq!(Ring::total_balance(&5), 1901);
			assert_eq!(System::account_nonce(&2), 0);
		});
}

#[test]
fn dust_account_removal_should_work2() {
	ExtBuilder::default()
		.existential_deposit(100)
		.creation_fee(50)
		.monied(true)
		.build()
		.execute_with(|| {
			System::inc_account_nonce(&2);
			assert_eq!(System::account_nonce(&2), 1);
			assert_eq!(Ring::total_balance(&2), 2000);
			// index 1 (account 2) becomes zombie for 256*10 + 50(fee) < 256 * 10 (ext_deposit)
			assert_ok!(Ring::transfer(Some(2).into(), 5, 1851));
			assert_eq!(Ring::total_balance(&2), 0);
			assert_eq!(Ring::total_balance(&5), 1851);
			assert_eq!(System::account_nonce(&2), 0);
		});
}

#[test]
fn balance_works() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 42);
		assert_eq!(Ring::free_balance(&1), 42);
		assert_eq!(Ring::reserved_balance(&1), 0);
		assert_eq!(Ring::total_balance(&1), 42);
		assert_eq!(Ring::free_balance(&2), 0);
		assert_eq!(Ring::reserved_balance(&2), 0);
		assert_eq!(Ring::total_balance(&2), 0);
	});
}

#[test]
fn balance_transfer_works() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 111);
		assert_ok!(Ring::transfer(Some(1).into(), 2, 69));
		assert_eq!(Ring::total_balance(&1), 42);
		assert_eq!(Ring::total_balance(&2), 69);
	});
}

#[test]
fn force_transfer_works() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 111);
		assert_err!(Ring::force_transfer(Some(2).into(), 1, 2, 69), DispatchError::BadOrigin,);
		assert_ok!(Ring::force_transfer(RawOrigin::Root.into(), 1, 2, 69));
		assert_eq!(Ring::total_balance(&1), 42);
		assert_eq!(Ring::total_balance(&2), 69);
	});
}

#[test]
fn reserving_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 111);

		assert_eq!(Ring::total_balance(&1), 111);
		assert_eq!(Ring::free_balance(&1), 111);
		assert_eq!(Ring::reserved_balance(&1), 0);

		assert_ok!(Ring::reserve(&1, 69));

		assert_eq!(Ring::total_balance(&1), 111);
		assert_eq!(Ring::free_balance(&1), 42);
		assert_eq!(Ring::reserved_balance(&1), 69);
	});
}

#[test]
fn balance_transfer_when_reserved_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 111);
		assert_ok!(Ring::reserve(&1, 69));
		assert_noop!(
			Ring::transfer(Some(1).into(), 2, 69),
			Error::<Test, _>::InsufficientBalance,
		);
	});
}

#[test]
fn deducting_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 111);
		assert_ok!(Ring::reserve(&1, 69));
		assert_eq!(Ring::free_balance(&1), 42);
	});
}

#[test]
fn refunding_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 42);
		Ring::set_reserved_balance(&1, 69);
		Ring::unreserve(&1, 69);
		assert_eq!(Ring::free_balance(&1), 111);
		assert_eq!(Ring::reserved_balance(&1), 0);
	});
}

#[test]
fn slashing_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 111);
		assert_ok!(Ring::reserve(&1, 69));
		assert!(Ring::slash(&1, 69).1.is_zero());
		assert_eq!(Ring::free_balance(&1), 0);
		assert_eq!(Ring::reserved_balance(&1), 42);
		assert_eq!(<TotalIssuance<Test>>::get(), 42);
	});
}

#[test]
fn slashing_incomplete_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 42);
		assert_ok!(Ring::reserve(&1, 21));
		assert_eq!(Ring::slash(&1, 69).1, 27);
		assert_eq!(Ring::free_balance(&1), 0);
		assert_eq!(Ring::reserved_balance(&1), 0);
		assert_eq!(<TotalIssuance<Test>>::get(), 0);
	});
}

#[test]
fn unreserving_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 111);
		assert_ok!(Ring::reserve(&1, 111));
		Ring::unreserve(&1, 42);
		assert_eq!(Ring::reserved_balance(&1), 69);
		assert_eq!(Ring::free_balance(&1), 42);
	});
}

#[test]
fn slashing_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 111);
		assert_ok!(Ring::reserve(&1, 111));
		assert_eq!(Ring::slash_reserved(&1, 42).1, 0);
		assert_eq!(Ring::reserved_balance(&1), 69);
		assert_eq!(Ring::free_balance(&1), 0);
		assert_eq!(<TotalIssuance<Test>>::get(), 69);
	});
}

#[test]
fn slashing_incomplete_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 111);
		assert_ok!(Ring::reserve(&1, 42));
		assert_eq!(Ring::slash_reserved(&1, 69).1, 27);
		assert_eq!(Ring::free_balance(&1), 69);
		assert_eq!(Ring::reserved_balance(&1), 0);
		assert_eq!(<TotalIssuance<Test>>::get(), 69);
	});
}

#[test]
fn transferring_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 110);
		let _ = Ring::deposit_creating(&2, 1);
		assert_ok!(Ring::reserve(&1, 110));
		assert_ok!(Ring::repatriate_reserved(&1, &2, 41), 0);
		assert_eq!(Ring::reserved_balance(&1), 69);
		assert_eq!(Ring::free_balance(&1), 0);
		assert_eq!(Ring::reserved_balance(&2), 0);
		assert_eq!(Ring::free_balance(&2), 42);
	});
}

#[test]
fn transferring_reserved_balance_to_nonexistent_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 111);
		assert_ok!(Ring::reserve(&1, 111));
		assert_noop!(Ring::repatriate_reserved(&1, &2, 42), Error::<Test, _>::DeadAccount);
	});
}

#[test]
fn transferring_incomplete_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 110);
		let _ = Ring::deposit_creating(&2, 1);
		assert_ok!(Ring::reserve(&1, 41));
		assert_ok!(Ring::repatriate_reserved(&1, &2, 69), 28);
		assert_eq!(Ring::reserved_balance(&1), 0);
		assert_eq!(Ring::free_balance(&1), 69);
		assert_eq!(Ring::reserved_balance(&2), 0);
		assert_eq!(Ring::free_balance(&2), 42);
	});
}

#[test]
fn transferring_too_high_value_should_not_panic() {
	ExtBuilder::default().build().execute_with(|| {
		<FreeBalance<Test>>::insert(1, u64::max_value());
		<FreeBalance<Test>>::insert(2, 1);

		assert_err!(
			Ring::transfer(Some(1).into(), 2, u64::max_value()),
			Error::<Test, _>::Overflow,
		);

		assert_eq!(Ring::free_balance(&1), u64::max_value());
		assert_eq!(Ring::free_balance(&2), 1);
	});
}

#[test]
fn account_create_on_free_too_low_with_other() {
	ExtBuilder::default().existential_deposit(100).build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 100);
		assert_eq!(<TotalIssuance<Test>>::get(), 100);

		// No-op.
		let _ = Ring::deposit_creating(&2, 50);
		assert_eq!(Ring::free_balance(&2), 0);
		assert_eq!(<TotalIssuance<Test>>::get(), 100);
	})
}

#[test]
fn account_create_on_free_too_low() {
	ExtBuilder::default().existential_deposit(100).build().execute_with(|| {
		// No-op.
		let _ = Ring::deposit_creating(&2, 50);
		assert_eq!(Ring::free_balance(&2), 0);
		assert_eq!(<TotalIssuance<Test>>::get(), 0);
	})
}

#[test]
fn account_removal_on_free_too_low() {
	ExtBuilder::default().existential_deposit(100).build().execute_with(|| {
		assert_eq!(<TotalIssuance<Test>>::get(), 0);

		// Setup two accounts with free balance above the existential threshold.
		let _ = Ring::deposit_creating(&1, 110);
		let _ = Ring::deposit_creating(&2, 110);

		assert_eq!(Ring::free_balance(&1), 110);
		assert_eq!(Ring::free_balance(&2), 110);
		assert_eq!(<TotalIssuance<Test>>::get(), 220);

		// Transfer funds from account 1 of such amount that after this transfer
		// the balance of account 1 will be below the existential threshold.
		// This should lead to the removal of all balance of this account.
		assert_ok!(Ring::transfer(Some(1).into(), 2, 20));

		// Verify free balance removal of account 1.
		assert_eq!(Ring::free_balance(&1), 0);
		assert_eq!(Ring::free_balance(&2), 130);

		// Verify that TotalIssuance tracks balance removal when free balance is too low.
		assert_eq!(<TotalIssuance<Test>>::get(), 130);
	});
}

#[test]
fn transfer_overflow_isnt_exploitable() {
	ExtBuilder::default().creation_fee(50).build().execute_with(|| {
		// Craft a value that will overflow if summed with `creation_fee`.
		let evil_value = u64::max_value() - 49;

		assert_err!(
			Ring::transfer(Some(1).into(), 5, evil_value),
			Error::<Test, _>::Overflow,
		);
	});
}

#[test]
fn check_vesting_status() {
	ExtBuilder::default()
		.existential_deposit(256)
		.monied(true)
		.vesting(true)
		.build()
		.execute_with(|| {
			assert_eq!(System::block_number(), 1);
			let user1_free_balance = Ring::free_balance(&1);
			let user2_free_balance = Ring::free_balance(&2);
			let user12_free_balance = Ring::free_balance(&12);
			assert_eq!(user1_free_balance, 256 * 10); // Account 1 has free balance
			assert_eq!(user2_free_balance, 256 * 20); // Account 2 has free balance
			assert_eq!(user12_free_balance, 256 * 10); // Account 12 has free balance
			let user1_vesting_schedule = VestingSchedule {
				locked: 256 * 5,
				per_block: 128, // Vesting over 10 blocks
				starting_block: 0,
			};
			let user2_vesting_schedule = VestingSchedule {
				locked: 256 * 20,
				per_block: 256, // Vesting over 20 blocks
				starting_block: 10,
			};
			let user12_vesting_schedule = VestingSchedule {
				locked: 256 * 5,
				per_block: 64, // Vesting over 20 blocks
				starting_block: 10,
			};
			assert_eq!(Ring::vesting(&1), Some(user1_vesting_schedule)); // Account 1 has a vesting schedule
			assert_eq!(Ring::vesting(&2), Some(user2_vesting_schedule)); // Account 2 has a vesting schedule
			assert_eq!(Ring::vesting(&12), Some(user12_vesting_schedule)); // Account 12 has a vesting schedule

			// Account 1 has only 128 units vested from their illiquid 256 * 5 units at block 1
			assert_eq!(Ring::vesting_balance(&1), 128 * 9);
			// Account 2 has their full balance locked
			assert_eq!(Ring::vesting_balance(&2), user2_free_balance);
			// Account 12 has only their illiquid funds locked
			assert_eq!(Ring::vesting_balance(&12), user12_free_balance - 256 * 5);

			System::set_block_number(10);
			assert_eq!(System::block_number(), 10);

			// Account 1 has fully vested by block 10
			assert_eq!(Ring::vesting_balance(&1), 0);
			// Account 2 has started vesting by block 10
			assert_eq!(Ring::vesting_balance(&2), user2_free_balance);
			// Account 12 has started vesting by block 10
			assert_eq!(Ring::vesting_balance(&12), user12_free_balance - 256 * 5);

			System::set_block_number(30);
			assert_eq!(System::block_number(), 30);

			assert_eq!(Ring::vesting_balance(&1), 0); // Account 1 is still fully vested, and not negative
			assert_eq!(Ring::vesting_balance(&2), 0); // Account 2 has fully vested by block 30
			assert_eq!(Ring::vesting_balance(&12), 0); // Account 2 has fully vested by block 30
		});
}

#[test]
fn unvested_balance_should_not_transfer() {
	ExtBuilder::default()
		.existential_deposit(10)
		.monied(true)
		.vesting(true)
		.build()
		.execute_with(|| {
			assert_eq!(System::block_number(), 1);
			let user1_free_balance = Ring::free_balance(&1);
			assert_eq!(user1_free_balance, 100); // Account 1 has free balance
									 // Account 1 has only 5 units vested at block 1 (plus 50 unvested)
			assert_eq!(Ring::vesting_balance(&1), 45);
			assert_noop!(Ring::transfer(Some(1).into(), 2, 56), Error::<Test, _>::VestingBalance,);
			// Account 1 cannot send more than vested amount
		});
}

#[test]
fn vested_balance_should_transfer() {
	ExtBuilder::default()
		.existential_deposit(10)
		.monied(true)
		.vesting(true)
		.build()
		.execute_with(|| {
			assert_eq!(System::block_number(), 1);
			let user1_free_balance = Ring::free_balance(&1);
			assert_eq!(user1_free_balance, 100); // Account 1 has free balance
									 // Account 1 has only 5 units vested at block 1 (plus 50 unvested)
			assert_eq!(Ring::vesting_balance(&1), 45);
			assert_ok!(Ring::transfer(Some(1).into(), 2, 55));
		});
}

#[test]
fn extra_balance_should_transfer() {
	ExtBuilder::default()
		.existential_deposit(10)
		.monied(true)
		.vesting(true)
		.build()
		.execute_with(|| {
			assert_eq!(System::block_number(), 1);
			assert_ok!(Ring::transfer(Some(3).into(), 1, 100));
			assert_ok!(Ring::transfer(Some(3).into(), 2, 100));

			let user1_free_balance = Ring::free_balance(&1);
			assert_eq!(user1_free_balance, 200); // Account 1 has 100 more free balance than normal

			let user2_free_balance = Ring::free_balance(&2);
			assert_eq!(user2_free_balance, 300); // Account 2 has 100 more free balance than normal

			// Account 1 has only 5 units vested at block 1 (plus 150 unvested)
			assert_eq!(Ring::vesting_balance(&1), 45);
			assert_ok!(Ring::transfer(Some(1).into(), 3, 155)); // Account 1 can send extra units gained

			// Account 2 has no units vested at block 1, but gained 100
			assert_eq!(Ring::vesting_balance(&2), 200);
			assert_ok!(Ring::transfer(Some(2).into(), 3, 100)); // Account 2 can send extra units gained
		});
}

#[test]
fn liquid_funds_should_transfer_with_delayed_vesting() {
	ExtBuilder::default()
		.existential_deposit(256)
		.monied(true)
		.vesting(true)
		.build()
		.execute_with(|| {
			assert_eq!(System::block_number(), 1);
			let user12_free_balance = Ring::free_balance(&12);

			assert_eq!(user12_free_balance, 2560); // Account 12 has free balance
									   // Account 12 has liquid funds
			assert_eq!(Ring::vesting_balance(&12), user12_free_balance - 256 * 5);

			// Account 12 has delayed vesting
			let user12_vesting_schedule = VestingSchedule {
				locked: 256 * 5,
				per_block: 64, // Vesting over 20 blocks
				starting_block: 10,
			};
			assert_eq!(Ring::vesting(&12), Some(user12_vesting_schedule));

			// Account 12 can still send liquid funds
			assert_ok!(Ring::transfer(Some(12).into(), 3, 256 * 5));
		});
}

#[test]
fn burn_must_work() {
	ExtBuilder::default().monied(true).build().execute_with(|| {
		let init_total_issuance = Ring::total_issuance();
		let imbalance = Ring::burn(10);
		assert_eq!(Ring::total_issuance(), init_total_issuance - 10);
		drop(imbalance);
		assert_eq!(Ring::total_issuance(), init_total_issuance);
	});
}

#[test]
fn transfer_keep_alive_works() {
	ExtBuilder::default().existential_deposit(1).build().execute_with(|| {
		let _ = Ring::deposit_creating(&1, 100);
		assert_err!(
			Ring::transfer_keep_alive(Some(1).into(), 2, 100),
			Error::<Test, _>::KeepAlive
		);
		assert_eq!(Ring::is_dead_account(&1), false);
		assert_eq!(Ring::total_balance(&1), 100);
		assert_eq!(Ring::total_balance(&2), 0);
	});
}

#[test]
#[should_panic = "the balance of any account should always be more than existential deposit."]
fn cannot_set_genesis_value_below_ed() {
	mock::EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = 11);
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let _ = GenesisConfig::<Test> {
		balances: vec![(1, 10)],
		vesting: vec![],
	}
	.assimilate_storage(&mut t)
	.unwrap();
}

#[test]
fn dust_moves_between_free_and_reserved() {
	ExtBuilder::default().existential_deposit(100).build().execute_with(|| {
		// Set balance to free and reserved at the existential deposit
		assert_ok!(Ring::set_balance(RawOrigin::Root.into(), 1, 100, 100));
		assert_ok!(Ring::set_balance(RawOrigin::Root.into(), 2, 100, 100));
		// Check balance
		assert_eq!(Ring::free_balance(1), 100);
		assert_eq!(Ring::reserved_balance(1), 100);
		assert_eq!(Ring::free_balance(2), 100);
		assert_eq!(Ring::reserved_balance(2), 100);

		// Drop 1 free_balance below ED
		assert_ok!(Ring::transfer(Some(1).into(), 2, 1));
		// Check balance, the other 99 should move to reserved_balance
		assert_eq!(Ring::free_balance(1), 0);
		assert_eq!(Ring::reserved_balance(1), 199);

		// Reset accounts
		assert_ok!(Ring::set_balance(RawOrigin::Root.into(), 1, 100, 100));
		assert_ok!(Ring::set_balance(RawOrigin::Root.into(), 2, 100, 100));

		// Drop 2 reserved_balance below ED
		Ring::unreserve(&2, 1);
		// Check balance, all 100 should move to free_balance
		assert_eq!(Ring::free_balance(2), 200);
		assert_eq!(Ring::reserved_balance(2), 0);

		// An account with both too little free and reserved is completely killed
		assert_ok!(Ring::set_balance(RawOrigin::Root.into(), 1, 99, 99));
		// Check balance is 0 for everything
		assert_eq!(Ring::free_balance(1), 0);
		assert_eq!(Ring::reserved_balance(1), 0);
	});
}
