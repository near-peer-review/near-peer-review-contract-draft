// use near_workspaces::{types::NearToken, Account, Contract};
// use serde_json::json;
// use std::{env, fs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// async fn test_default_message(
//     user: &Account,
//     contract: &Contract,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let greeting: String = user
//         .call(contract.id(), "get_greeting")
//         .args_json(json!({}))
//         .transact()
//         .await?
//         .json()?;

//     assert_eq!(greeting, "Hello".to_string());
//     println!("      Passed ✅ gets default greeting");
//     Ok(())
// }

// async fn test_changes_message(
//     user: &Account,
//     contract: &Contract,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     user.call(contract.id(), "set_greeting")
//         .args_json(json!({"greeting": "Howdy"}))
//         .transact()
//         .await?
//         .into_result()?;

//     let greeting: String = user
//         .call(contract.id(), "get_greeting")
//         .args_json(json!({}))
//         .transact()
//         .await?
//         .json()?;

//     assert_eq!(greeting, "Howdy".to_string());
//     println!("      Passed ✅ changes greeting");
//     Ok(())
// }
