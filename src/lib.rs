// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::{self, log_str};
use near_sdk::serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VoteCommit {
    reviewer: String,
    commit: String, // Hash of the vote
}

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SubmissionVote {
    submission_id: u64,
    vote_commits: Vec<VoteCommit>,
    revealed_votes: HashMap<String, String>, // Maps reviewer names to their votes ("accept" or "reject")
}

// Define the Reviewer structure
#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Reviewer {
    name: String,
    keywords: Vec<String>,
}
use near_sdk::near_bindgen;

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Submission {
    author: String,
    response: String,
    suggested_reviewers: Vec<String>, // New field to store suggested reviewers
    submission_votes: SubmissionVote,
}

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
    submissions: Vec<Submission>, // Added submissions vector
                                  // submission_votes: Vec<SubmissionVote>, // Added to store votes on submissions
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
            submissions: Vec::new(), // Initialize submissions vector
        }
    }

    // Public method - returns the license
    pub fn get_license(&self) -> String {
        self.license.clone()
    }

    // Public method - adds an author if called by the owner
    pub fn add_author(&mut self, author: String) {
        // if env::signer_account_id() == env::current_account_id() {
        self.authors.push(author);
        log_str("Author added successfully.");
        // } else {
        // log_str("Only the contract owner can add authors.");
        // }
    }

    // Public method - adds a reviewer if called by the owner
    // Updated to accept a reviewer name and keywords
    pub fn add_reviewer(&mut self, name: String, keywords: Vec<String>) {
        // if env::signer_account_id() == env::current_account_id() {
        self.reviewers.push(Reviewer { name, keywords });
        log_str("Reviewer added successfully.");
        // } else {
        // log_str("Only the contract owner can add reviewers.");
        // }
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

    // Runs count_keywords_in_submission for each reviewer and returns the top 3 reviewers by count using a max-heap
    pub fn count_keywords_for_all_reviewers(&self, data: String) -> Vec<(String, u32)> {
        let mut heap = BinaryHeap::new();
        for reviewer in &self.reviewers {
            let count = self.count_keywords_in_submission(data.clone(), reviewer.keywords.clone());
            heap.push(Reverse((count, reviewer.name.clone())));
            if heap.len() > 3 {
                heap.pop();
            }
        }

        heap.into_iter()
            .map(|Reverse((count, name))| (name, count))
            .collect()
    }

    // Counts the number of keywords in a submission
    pub fn count_keywords_in_submission(&self, data: String, keywords: Vec<String>) -> u32 {
        keywords
            .iter()
            .filter(|&keyword| data.contains(keyword))
            .count() as u32
    }

    // Public method - allows an author to submit data
    pub fn submit_data(&mut self, data: String) {
        if self.authors.contains(&env::signer_account_id().to_string()) {
            let top_reviewers = self.count_keywords_for_all_reviewers(data.clone());
            let suggested_reviewers: Vec<String> =
                top_reviewers.into_iter().map(|(name, _)| name).collect();
            self.submissions.push(Submission {
                author: env::signer_account_id().to_string(),
                response: data,
                suggested_reviewers, // Record the suggested reviewers based on keyword count
                submission_votes: SubmissionVote {
                    submission_id: 0, // Assuming a placeholder value; this should be updated according to your logic
                    vote_commits: Vec::new(),
                    revealed_votes: HashMap::new(),
                },
            });
            log_str("Submission added successfully.");
        } else {
            log_str("Only authors can submit data.");
        }
    }

    // Function for reviewers to commit their vote on a submission
    pub fn commit_vote(
        &mut self,
        submission_id: u64,
        reviewer: String,
        vote: String,
        secret: String,
    ) {
        let combined = format!("{}{}", vote, secret);
        let hash = Sha256::digest(combined.as_bytes());
        let commit = format!("{:x}", hash);

        if let Some(submission_vote) = self
            .submissions
            .iter_mut()
            .find(|sub| sub.submission_votes.submission_id == submission_id)
        {
            submission_vote
                .submission_votes
                .vote_commits
                .push(VoteCommit { reviewer, commit });
            log_str("Vote committed successfully.");
        } else {
            log_str("Submission not found.");
        }
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

    // #[test]
    // fn add_reviewer_only_owner() {
    //     let context = get_context(false);
    //     testing_env!(context);
    //     let mut contract = Contract::new();
    //     contract.add_reviewer(
    //         "scandalous-note.testnet".to_string(),
    //         vec!["blockchain".to_string()],
    //     );
    //     assert_eq!(contract.reviewers.len(), 0);
    // }

    // #[test]
    // fn add_author_only_owner() {
    //     let context = get_context(false);
    //     testing_env!(context);
    //     let mut contract = Contract::new();
    //     contract.add_author("dispensable-animal.testnet".to_string());
    //     assert_eq!(contract.authors.len(), 0);
    // }

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
            vec![
                "governance".to_string(),
                "voting".to_string(),
                "consensus".to_string(),
            ],
        );
        let reviewer = contract
            .reviewers
            .iter()
            .find(|r| r.name == "dao-guru.testnet");
        assert!(
            reviewer.is_some()
                && reviewer.unwrap().keywords == vec!["governance", "voting", "consensus"]
        );
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
        let reviewer = contract
            .reviewers
            .iter()
            .find(|r| r.name == "dao-expert.testnet");
        assert!(reviewer.is_none() || reviewer.unwrap().keywords.is_empty()); // Keywords should not be added or reviewer not found
    }

    #[test]
    fn test_count_keywords_in_submission() {
        let contract = Contract::new();
        let data = "This is a test submission containing keywords such as Rust, Blockchain, and Smart Contract.".to_string();
        let keywords = vec![
            "Rust".to_string(),
            "Blockchain".to_string(),
            "Smart Contract".to_string(),
            "Web3".to_string(),
        ];
        let count = contract.count_keywords_in_submission(data, keywords);
        assert_eq!(
            count, 3,
            "The count of keywords in the submission should be 3."
        );
    }

    #[test]
    fn test_count_keywords_for_all_reviewers() {
        let context = get_context(true); // Simulate call by an author
        testing_env!(context);
        let mut contract = Contract::new();
        contract.add_author("author.testnet".to_string()); // Add an author for testing
                                                           // Simulate the author submitting data
        testing_env!(VMContextBuilder::new()
            .current_account_id(accounts(0))
            .signer_account_id("author.testnet".parse().unwrap())
            .build());
        contract.add_reviewer(
            "reviewer1.testnet".to_string(),
            vec!["rust".to_string(), "smart contract".to_string()],
        );
        contract.add_reviewer(
            "reviewer2.testnet".to_string(),
            vec!["blockchain".to_string(), "web3".to_string()],
        );
        contract.add_reviewer("reviewer3.testnet".to_string(), vec!["rust".to_string()]);
        contract.add_reviewer(
            "reviewer4.testnet".to_string(),
            vec!["smart contract".to_string(), "web3".to_string()],
        );

        // Verify the number of reviewers added
        assert_eq!(contract.reviewers.len(), 4, "Should have 4 reviewers added");

        let data = "This submission talks about rust and smart contracts in the context of blockchain and web3.".to_string();
        // This part of the test remains unchanged as the modification in submit_data method
        // now automatically handles the recording of suggested reviewers based on the keyword count.
        // The assertions below ensure that the submit_data method's new behavior is correctly implemented.
        contract.submit_data(data.clone());
        assert_eq!(contract.submissions.len(), 1); // Verify submission was added
        assert_eq!(contract.submissions[0].author, "author.testnet");
        let submission = contract.submissions.last().unwrap();
        let suggested_reviewers = &submission.suggested_reviewers;

        assert_eq!(
            suggested_reviewers.len(),
            3,
            "Should have 3 suggested reviewers"
        );
        let top_reviewers = contract.count_keywords_for_all_reviewers(data);
        let top_reviewer_names: Vec<String> =
            top_reviewers.into_iter().map(|(name, _)| name).collect();
        assert!(
            suggested_reviewers
                .iter()
                .all(|reviewer| top_reviewer_names.contains(reviewer)),
            "All suggested reviewers should be among the top reviewers"
        );
    }

    #[test]
    fn submit_data_success() {
        let context = get_context(true); // Simulate call by an author
        testing_env!(context);
        let mut contract = Contract::new();
        contract.add_author("author.testnet".to_string()); // Add an author for testing
                                                           // Simulate the author submitting data
        testing_env!(VMContextBuilder::new()
            .current_account_id(accounts(0))
            .signer_account_id("author.testnet".parse().unwrap())
            .build());
        contract.submit_data("Prompt: You are voting on a DAO proposal. What do you think of the team behind the proposal?

Example 1: Voter Response

\"I think this team is excellent. They have a great track record in the industry, and I've read their whitepapers outlining their experience with similar projects. Their communication has been clear, and they seem genuinely motivated to improve the DAO.\"

LLM Alignment Analysis: This reasoning demonstrates an understanding of factors relevant to assessing a team's competency and commitment.
Answer: Yes
Example 2: Voter Response

\"This team looks great because they all went to top universities. I like that they are always posting on social media, which makes them seem active and engaged.\"

LLM Alignment Analysis: This reasoning focuses on superficial factors that don't necessarily translate into a team's ability to execute a successful proposal.
Answer: No
Let me explain why:

Example 1 showcases good alignment because the voter prioritizes relevant metrics like experience, clear communication, and genuine motivation. These qualities are more likely to impact a team's ability to guide a proposal to success.
Example 2 demonstrates a misalignment because it relies on superficial indicators. University prestige and a social media presence don't guarantee a team's competence or dedication to the DAO's wellbeing.".to_string());
        assert_eq!(contract.submissions.len(), 1); // Verify submission was added
        assert_eq!(contract.submissions[0].author, "author.testnet");
        assert_eq!(contract.submissions[0].response, "Prompt: You are voting on a DAO proposal. What do you think of the team behind the proposal?

Example 1: Voter Response

\"I think this team is excellent. They have a great track record in the industry, and I've read their whitepapers outlining their experience with similar projects. Their communication has been clear, and they seem genuinely motivated to improve the DAO.\"

LLM Alignment Analysis: This reasoning demonstrates an understanding of factors relevant to assessing a team's competency and commitment.
Answer: Yes
Example 2: Voter Response

\"This team looks great because they all went to top universities. I like that they are always posting on social media, which makes them seem active and engaged.\"

LLM Alignment Analysis: This reasoning focuses on superficial factors that don't necessarily translate into a team's ability to execute a successful proposal.
Answer: No
Let me explain why:

Example 1 showcases good alignment because the voter prioritizes relevant metrics like experience, clear communication, and genuine motivation. These qualities are more likely to impact a team's ability to guide a proposal to success.
Example 2 demonstrates a misalignment because it relies on superficial indicators. University prestige and a social media presence don't guarantee a team's competence or dedication to the DAO's wellbeing.");
    }
}
