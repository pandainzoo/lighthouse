use super::errors::{DepositInvalid as Invalid, DepositValidationError as Error};
use hashing::hash;
use merkle_proof::verify_merkle_proof;
use ssz::ssz_encode;
use ssz_derive::Encode;
use types::*;

/// Indicates if a `Deposit` is valid to be included in a block in the current epoch of the given
/// state.
///
/// Returns `Ok(())` if the `Deposit` is valid, otherwise indicates the reason for invalidity.
///
/// This function _does not_ check `state.deposit_index` so this function may be run in parallel.
/// See the `verify_deposit_index` function for this.
///
/// Note: this function is incomplete.
///
/// Spec v0.5.1
pub fn verify_deposit(
    state: &BeaconState,
    deposit: &Deposit,
    verify_merkle_branch: bool,
    spec: &ChainSpec,
) -> Result<(), Error> {
    verify!(
        deposit
            .deposit_data
            .deposit_input
            .validate_proof_of_possession(
                state.slot.epoch(spec.slots_per_epoch),
                &state.fork,
                spec
            ),
        Invalid::BadProofOfPossession
    );

    if verify_merkle_branch {
        verify!(
            verify_deposit_merkle_proof(state, deposit, spec),
            Invalid::BadMerkleProof
        );
    }

    Ok(())
}

/// Verify that the `Deposit` index is correct.
///
/// Spec v0.5.1
pub fn verify_deposit_index(state: &BeaconState, deposit: &Deposit) -> Result<(), Error> {
    verify!(
        deposit.index == state.deposit_index,
        Invalid::BadIndex {
            state: state.deposit_index,
            deposit: deposit.index
        }
    );

    Ok(())
}

/// Returns a `Some(validator index)` if a pubkey already exists in the `validator_registry`,
/// otherwise returns `None`.
///
/// ## Errors
///
/// Errors if the state's `pubkey_cache` is not current.
pub fn get_existing_validator_index(
    state: &BeaconState,
    deposit: &Deposit,
) -> Result<Option<u64>, Error> {
    let deposit_input = &deposit.deposit_data.deposit_input;

    let validator_index = state.get_validator_index(&deposit_input.pubkey)?;

    match validator_index {
        None => Ok(None),
        Some(index) => {
            verify!(
                deposit_input.withdrawal_credentials
                    == state.validator_registry[index as usize].withdrawal_credentials,
                Invalid::BadWithdrawalCredentials
            );
            Ok(Some(index as u64))
        }
    }
}

/// Verify that a deposit is included in the state's eth1 deposit root.
///
/// Spec v0.6.0
fn verify_deposit_merkle_proof(state: &BeaconState, deposit: &Deposit, spec: &ChainSpec) -> bool {
    let leaf = deposit.data.tree_hash_root();
    verify_merkle_proof(
        Hash256::from_slice(&leaf),
        &deposit.proof,
        spec.deposit_contract_tree_depth as usize,
        deposit.index as usize,
        state.latest_eth1_data.deposit_root,
    )
}
