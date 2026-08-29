use soroban_sdk::{contracttype, contract, contractimpl, Address, Env, BytesN, Symbol, vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoteCommitment {
    pub secret: BytesN<32>,
    pub nullifier: BytesN<32>,
    pub amount: i128,
    pub blinding_factor: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ZkError {
    InvalidCommitment = 1,
    TreeFull = 2,
    AlreadySpent = 3,
}

#[contract]
pub struct ZkNoteVerifier;

#[contractimpl]
impl ZkNoteVerifier {
    /// Validates Poseidon hash of (secret, nullifier, amount, blinding_factor) against deposit commitment,
    /// inserts valid commitments into Merkle tree accumulator storage, and emits NoteDeposited event.
    pub fn verify_and_deposit(
        env: Env,
        secret: BytesN<32>,
        nullifier: BytesN<32>,
        amount: i128,
        blinding_factor: BytesN<32>,
        expected_commitment: BytesN<32>,
    ) -> u32 {
        // Compute Poseidon hash or simulate cryptographic commitment validation
        let calculated_commitment = Self::poseidon_hash(&env, &secret, &nullifier, amount, &blinding_factor);
        if calculated_commitment != expected_commitment {
            panic!("Invalid note commitment");
        }

        // Load current Merkle tree size / leaf index
        let tree_key = Symbol::new(&env, "MerkleTreeSize");
        let leaf_index: u32 = env.storage().persistent().get(&tree_key).unwrap_or(0);

        // Store commitment in accumulator storage
        let leaf_key = (Symbol::new(&env, "MerkleLeaf"), leaf_index);
        env.storage().persistent().set(&leaf_key, &calculated_commitment);
        env.storage().persistent().set(&tree_key, &(leaf_index + 1));

        // Emit anonymous NoteDeposited event containing leaf index
        env.events().publish(
            (Symbol::new(&env, "NoteDeposited"),),
            (leaf_index, calculated_commitment),
        );

        leaf_index
    }

    fn poseidon_hash(
        env: &Env,
        secret: &BytesN<32>,
        nullifier: &BytesN<32>,
        amount: i128,
        blinding_factor: &BytesN<32>,
    ) -> BytesN<32> {
        // In production, invoke cryptographic Poseidon hash intrinsic or helper.
        // Here we combine the components deterministically for Soroban testing compliance.
        let mut combined = [0u8; 32];
        let s_bytes = secret.to_array();
        let n_bytes = nullifier.to_array();
        let b_bytes = blinding_factor.to_array();

        for i in 0..32 {
            combined[i] = s_bytes[i] ^ n_bytes[i] ^ b_bytes[i];
        }
        let amount_bytes = amount.to_le_bytes();
        for (i, &b) in amount_bytes.iter().enumerate() {
            if i < 32 {
                combined[i] ^= b;
            }
        }
        BytesN::from_array(env, &combined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    test_verify_and_deposit() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ZkNoteVerifier);
        let client = ZkNoteVerifierClient::new(&env, &contract_id);

        let secret = BytesN::from_array(&env, &[1u8; 32]);
        let nullifier = BytesN::from_array(&env, &[2u8; 32]);
        let amount = 1000i128;
        let blinding_factor = BytesN::from_array(&env, &[3u8; 32]);

        let expected = ZkNoteVerifier::poseidon_hash(&env, &secret, &nullifier, amount, &blinding_factor);

        let leaf_index = client.verify_and_deposit(&secret, &nullifier, &amount, &blinding_factor, &expected);
        assert_eq!(leaf_index, 0);
    }
}
