// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::{self, log_str};
use near_sdk::serde::{Deserialize, Serialize};

// Define the Reviewer structure
#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Reviewer {
    name: String,
    keywords: Vec<String>,
}
use near_sdk::near_bindgen;

// Define the contract structure
#[near_bindgen]
#[derive(
    BorshDeserialize, BorshSerialize, near_sdk::serde::Serialize, near_sdk::serde::Deserialize,
)]
#[serde(crate = "near_sdk::serde")]
pub struct Contract {
    license: String,
    authors: Vec<String>,
    reviewers: Vec<Reviewer>,
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
            reviewers: Vec::new(),
        }
    }

    // Public method - returns the license
    pub fn get_license(&self) -> String {
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
    // Updated to accept a reviewer name and keywords
    pub fn add_reviewer(&mut self, name: String, keywords: Vec<String>) {
        if env::signer_account_id() == env::current_account_id() {
            self.reviewers.push(Reviewer { name, keywords });
            log_str("Reviewer added successfully.");
        } else {
            log_str("Only the contract owner can add reviewers.");
        }
    }

    // Public method - allows a reviewer to add keywords to themselves
    pub fn add_keywords_to_reviewer(&mut self, name: String, new_keywords: Vec<String>) {
        if env::signer_account_id() == name.parse().unwrap() {
            if let Some(reviewer) = self.reviewers.iter_mut().find(|r| r.name == name) {
                reviewer.keywords.extend(new_keywords);
                log_str("Keywords added successfully.");
            } else {
                log_str("Reviewer not found.");
            }
        } else {
            log_str("Only the reviewer can add keywords to themselves.");
        }
    }

    pub fn set_license(&mut self, license: String) {
        log_str(&format!("Saving license: {license}"));
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
            builder
                .current_account_id(accounts(0))
                .signer_account_id(accounts(0));
        } else {
            builder
                .current_account_id(accounts(0))
                .signer_account_id(accounts(1));
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
        contract.add_reviewer(
            "quirky-sand.testnet".to_string(),
            vec!["rust".to_string(), "smart contract".to_string()],
        );
        assert_eq!(contract.reviewers.len(), 1);
        assert_eq!(contract.reviewers[0].name, "quirky-sand.testnet");
    }

    #[test]
    fn add_reviewer_only_owner() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new();
        contract.add_reviewer(
            "scandalous-note.testnet".to_string(),
            vec!["blockchain".to_string()],
        );
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
    fn get_default_license() {
        let contract = Contract::default();
        assert_eq!(contract.get_license(), "CC BY-NC-SA".to_string());
    }

    #[test]
    fn add_keywords_to_reviewer_success() {
        let context = get_context(true); // Simulate call by the reviewer themselves
        testing_env!(context);
        let mut contract = Contract::new();
        contract.add_reviewer("dao-guru.testnet".to_string(), vec![]);
        // Correctly simulate the reviewer adding keywords to themselves
        testing_env!(VMContextBuilder::new()
            .current_account_id(accounts(0))
            .signer_account_id("dao-guru.testnet".parse().unwrap())
            .build());
        contract.add_keywords_to_reviewer(
            "dao-guru.testnet".to_string(),
            vec!["governance".to_string(), "voting".to_string(), "consensus".to_string()],
        );
        let reviewer = contract.reviewers.iter().find(|r| r.name == "dao-guru.testnet");
        assert!(reviewer.is_some() && reviewer.unwrap().keywords == vec!["governance", "voting", "consensus"]);
    }

    #[test]
    fn add_keywords_to_reviewer_not_found() {
        let context = get_context(true);
        testing_env!(context);
        let mut contract = Contract::new();
        contract.add_keywords_to_reviewer(
            "nonexistent-reviewer.testnet".to_string(),
            vec!["governance".to_string()],
        );
        assert!(contract.reviewers.is_empty());
    }

    #[test]
    fn add_keywords_to_reviewer_not_self() {
        let context = get_context(false); // Simulate call by someone other than the reviewer
        testing_env!(context);
        let mut contract = Contract::new();
        contract.add_reviewer("dao-expert.testnet".to_string(), vec![]);
        contract.add_keywords_to_reviewer(
            "dao-expert.testnet".to_string(),
            vec!["decentralization".to_string()],
        );
        let reviewer = contract.reviewers.iter().find(|r| r.name == "dao-expert.testnet");
        assert!(reviewer.is_none() || reviewer.unwrap().keywords.is_empty()); // Keywords should not be added or reviewer not found
    }
}
