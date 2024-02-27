// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::{self, log_str};
use near_sdk::near_bindgen;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, near_sdk::serde::Serialize, near_sdk::serde::Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Contract {
    license: String,
    authors: Vec<String>,
    reviewers: Vec<String>,
}

// Define the default, which automatically initializes the contract
// This block is removed as we now use `new` for initialization.

// Implement the Default trait for Contract
impl Default for Contract {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    // Initialize the authors list as empty
    pub fn new() -> Self {
        Self {
            license: "CC BY-NC-SA".to_string(),
            authors: Vec::new(),
        }
    }

    // Public method - returns the license
    pub fn get_greeting(&self) -> String {
        self.license.clone()
    }

    // Public method - adds an author if called by the owner
    pub fn add_author(&mut self, author: String) {
        if env::signer_account_id() == env::current_account_id() {
            self.authors.push(author);
            log_str("Author added successfully.");
        } else {
            log_str("Only the contract owner can add authors.");
        }
    }

    // Public method - adds a reviewer if called by the owner
    pub fn add_reviewer(&mut self, reviewer: String) {
        if env::signer_account_id() == env::current_account_id() {
            self.reviewers.push(reviewer);
            log_str("Reviewer added successfully.");
        } else {
            log_str("Only the contract owner can add reviewers.");
        }
    }

    // Public method - accepts a greeting, such as "howdy", and records it
    pub fn set_greeting(&mut self, license: String) {
        log_str(&format!("Saving greeting: {license}"));
        self.license = license;
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_owner: bool) -> VMContext {
        let mut builder = VMContextBuilder::new();
        if is_owner {
            builder.current_account_id(accounts(0)).signer_account_id(accounts(0));
        } else {
            builder.current_account_id(accounts(0)).signer_account_id(accounts(1));
        }
        builder.build()
    }

    #[test]
    fn add_author_success() {
        let context = get_context(true);
        testing_env!(context);
        let mut contract = Contract::new();
        contract.add_author("snotty-body.testnet".to_string());
        assert_eq!(contract.authors.len(), 1);
        assert_eq!(contract.authors[0], "snotty-body.testnet");
    }

    #[test]
    fn add_reviewer_success() {
        let context = get_context(true);
        testing_env!(context);
        let mut contract = Contract::new();
        contract.add_reviewer("quirky-sand.testnet".to_string());
        assert_eq!(contract.reviewers.len(), 1);
        assert_eq!(contract.reviewers[0], "quirky-sand.testnet");
    }

    #[test]
    fn add_reviewer_only_owner() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new();
        contract.add_reviewer("scandalous-note.testnet".to_string());
        assert_eq!(contract.reviewers.len(), 0);
    }

    #[test]
    fn add_author_only_owner() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new();
        contract.add_author("dispensable-animal.testnet".to_string());
        assert_eq!(contract.authors.len(), 0);
    }

    #[test]
    fn get_default_greeting() {
        let contract = Contract::default();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(contract.get_greeting(), "CC BY-NC-SA".to_string());
    }

    #[test]
    fn set_then_get_greeting() {
        let mut contract = Contract::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(contract.get_greeting(), "howdy".to_string());
    }
}
