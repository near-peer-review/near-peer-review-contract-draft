// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

import "@api3/airnode-protocol/contracts/rrp/requesters/RrpRequesterV0.sol";
// import "@openzeppelin/contracts@4.9.5/access/Ownable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

// contract PeerReview {
contract PeerReview is RrpRequesterV0, Ownable {
  // Event declaration
  event RequestedUint256(bytes32 indexed requestId);
  event ReceivedUint256(bytes32 indexed requestId, uint256 response);
  event WithdrawalRequested(address indexed airnode, address indexed sponsorWallet);

  event SubmissionCreated(uint256 submissionId);

  constructor(
    address[] memory _authors,
    address[] memory _reviewerAddresses,
    address _airnodeRrp
  ) RrpRequesterV0(_airnodeRrp) {
    authors = _authors;
    for (uint256 i = 0; i < _reviewerAddresses.length; i++) {
      reviewers.push(Reviewer(_reviewerAddresses[i], new string[](0)));
    }
  }

  struct Reviewer {
    address addr;
    string[] keywords;
  }

  struct Submission {
    address author;
    string question;
    string response;
    string metadata; // Added metadata
    mapping(address => bytes32) commits;
    mapping(address => bool) votes;
    address[] selectedReviewers;
    address[] shuffledReviewers; // Updated field to store shuffled reviewers
    bool votingEnded;
    bool revealPhase;
    uint256 revealCount;
    bool isApproved;
    uint256 seed; // New field to store seed
  }

  address[] public authors;
  Reviewer[] public reviewers;
  Submission[] public submissions;
  string public constant LICENSE = "CC BY-NC-SA";
  uint256 public constant ROI_FEE_DENOMINATOR = 100;
  address public airnode; /// The address of the QRNG Airnode
  bytes32 public endpointIdUint256; /// The endpoint ID for requesting a single random number
  address public sponsorWallet; /// The wallet that will cover the gas costs of the request
  uint256 public _qrngUint256; /// The random number returned by the QRNG Airnode

  mapping(bytes32 => bool) public expectingRequestWithIdToBeFulfilled;

  // Updated function for reviewers to add their keywords
  function addKeywords(string[] memory _keywords) public {
    bool isReviewer = false;
    for (uint256 i = 0; i < reviewers.length; i++) {
      if (reviewers[i].addr == msg.sender) {
        reviewers[i].keywords = _keywords;
        isReviewer = true;
        break;
      }
    }
    require(isReviewer, "Caller is not a reviewer.");
  }

  // Submit a data object
  function submitData(string memory _question, string memory _response) public returns (uint256) {
    Submission storage newSubmission = submissions.push();
    newSubmission.author = msg.sender;
    newSubmission.question = _question;
    newSubmission.response = _response;
    uint256 submissionId = submissions.length - 1;
    emit SubmissionCreated(submissionId);
    return submissionId;
  }

  /// @notice Sets the parameters for making requests
  function setRequestParameters(
    address _airnode,
    bytes32 _endpointIdUint256,
    // bytes32 _endpointIdUint256Array,
    address _sponsorWallet
  ) external {
    airnode = _airnode;
    endpointIdUint256 = _endpointIdUint256;
    sponsorWallet = _sponsorWallet;
  }

  /// @notice To receive funds from the sponsor wallet and send them to the owner.
  receive() external payable {
    payable(owner()).transfer(msg.value);
    emit WithdrawalRequested(airnode, sponsorWallet);
  }

  /// @notice Requests a `uint256`
  /// @dev This request will be fulfilled by the contract's sponsor wallet,
  /// which means spamming it may drain the sponsor wallet.
  function makeRequestUint256() external {
    bytes32 requestId = airnodeRrp.makeFullRequest(
      airnode,
      endpointIdUint256,
      address(this),
      sponsorWallet,
      address(this),
      this.fulfillUint256.selector,
      ""
    );
    expectingRequestWithIdToBeFulfilled[requestId] = true;
    emit RequestedUint256(requestId);
  }

  /// @notice Called by the Airnode through the AirnodeRrp contract to
  /// fulfill the request
  /// Function to assign a seed to a submission
  function fulfillUint256(bytes32 requestId, bytes calldata data) external onlyAirnodeRrp {
    require(expectingRequestWithIdToBeFulfilled[requestId], "Request ID not known");
    expectingRequestWithIdToBeFulfilled[requestId] = false;
    uint256 qrngUint256 = abi.decode(data, (uint256));
    _qrngUint256 = qrngUint256;
    // Do what you want with `qrngUint256` here...
    emit ReceivedUint256(requestId, qrngUint256);
  }

  // Function to assign a seed to a submission
  function assignQrndSeed(uint256 submissionId) public {
    require(submissionId < submissions.length, "Invalid submission ID");
    submissions[submissionId].seed = _qrngUint256;
  }

  // Function to assign a seed to a submission
  function assignSeed(uint256 submissionId, uint256 _seed) public {
    require(submissionId < submissions.length, "Invalid submission ID");
    submissions[submissionId].seed = _seed;
  }

  // Find top 3 matching reviewers for a submission
  function findReviewers(uint256 submissionId) public {
    // The shuffleReviewers call is updated to shuffle and store reviewers in the Submission struct
    shuffleReviewers(submissionId); // This call now populates the shuffledReviewers field in the Submission struct
    require(submissionId < submissions.length, "Invalid submission ID");
    Submission storage submission = submissions[submissionId];

    address[] memory topReviewers = new address[](3);
    uint256[] memory topReviewersValue = new uint256[](3);

    uint256[] memory scores = new uint256[](submission.shuffledReviewers.length);
    for (uint256 i = 0; i < submission.shuffledReviewers.length; i++) {
      address reviewerAddr = submission.shuffledReviewers[i];
      // Find the reviewer in the global reviewers array to access their keywords
      for (uint256 k = 0; k < reviewers.length; k++) {
        if (reviewers[k].addr == reviewerAddr) {
          for (uint256 j = 0; j < reviewers[k].keywords.length; j++) {
            if (
              contains(submission.question, reviewers[k].keywords[j]) ||
              contains(submission.response, reviewers[k].keywords[j])
            ) {
              scores[i]++;
            }
          }
          break; // Break the loop once the matching reviewer is found
        }
      }

      if (scores[i] >= topReviewersValue[0]) {
        topReviewersValue[2] = topReviewersValue[1];
        topReviewersValue[1] = topReviewersValue[0];
        topReviewersValue[0] = scores[i];
        topReviewers[2] = topReviewers[1];
        topReviewers[1] = topReviewers[0];
        topReviewers[0] = reviewerAddr;
      } else if (scores[i] > topReviewersValue[1]) {
        topReviewersValue[2] = topReviewersValue[1];
        topReviewersValue[1] = scores[i];
        topReviewers[2] = topReviewers[1];
        topReviewers[1] = reviewerAddr;
      } else if (scores[i] > topReviewersValue[2]) {
        topReviewersValue[2] = scores[i];
        topReviewers[2] = reviewerAddr;
      }
    }

    submission.selectedReviewers = topReviewers;
  }

  // Updated function to shuffle a copy of the reviewers and store it in the Submission struct
  function shuffleReviewers(uint256 submissionId) internal {
    require(submissionId < submissions.length, "Invalid submission ID");
    Submission storage submission = submissions[submissionId];
    address[] memory shuffledReviewers = new address[](reviewers.length);
    for (uint256 i = 0; i < reviewers.length; i++) {
      shuffledReviewers[i] = reviewers[i].addr;
    }
    uint256 seed = submission.seed;
    for (uint256 i = 0; i < shuffledReviewers.length; i++) {
      uint256 j = (uint256(keccak256(abi.encode(seed, i))) % (i + 1));
      (shuffledReviewers[i], shuffledReviewers[j]) = (shuffledReviewers[j], shuffledReviewers[i]);
    }
    submission.shuffledReviewers = shuffledReviewers;
  }

  // A simple function to check if a string contains a substring
  function contains(string memory _string, string memory _substring) public pure returns (bool) {
    bytes memory stringBytes = bytes(_string);
    bytes memory substringBytes = bytes(_substring);

    // Simple loop to check substring
    for (uint256 i = 0; i < stringBytes.length - substringBytes.length; i++) {
      bool isMatch = true;
      for (uint256 j = 0; j < substringBytes.length; j++) {
        if (stringBytes[i + j] != substringBytes[j]) {
          isMatch = false;
          break;
        }
      }
      if (isMatch) return true;
    }
    return false;
  }

  // Function to get selected reviewers for a submission
  function getSelectedReviewers(uint256 submissionId) public view returns (address[] memory) {
    require(submissionId < submissions.length, "Invalid submission ID");
    return submissions[submissionId].selectedReviewers;
  }

  // Commit a vote as a hash
  function commitVote(uint256 submissionId, bytes32 commitHash) public {
    require(submissionId < submissions.length, "Invalid submission ID");
    Submission storage submission = submissions[submissionId];
    require(!submission.votingEnded, "Voting has ended");

    submission.commits[msg.sender] = commitHash;
  }

  // End the voting phase
  function endVoting(uint256 submissionId) public {
    require(submissionId < submissions.length, "Invalid submission ID");
    Submission storage submission = submissions[submissionId];
    submission.votingEnded = true;
  }

  // Function to check if the voting phase has ended for a submission
  function getVotingEnded(uint256 submissionId) public view returns (bool) {
    require(submissionId < submissions.length, "Invalid submission ID");
    Submission storage submission = submissions[submissionId];
    return submission.votingEnded;
  }

  // Reveal a vote
  function revealVote(
    uint256 submissionId,
    bool vote,
    bytes32 secret
  ) public {
    require(submissionId < submissions.length, "Invalid submission ID");
    Submission storage submission = submissions[submissionId];
    require(submission.votingEnded, "Voting has not ended");
    require(keccak256(abi.encodePacked(vote, secret)) == submission.commits[msg.sender], "Invalid commit");

    submission.votes[msg.sender] = vote;
    submission.revealCount++;

    if (submission.revealCount == submission.selectedReviewers.length) {
      submission.revealPhase = true;
      determineApproval(submissionId); // Determine if the submission is approved
    }
  }

  // Determine if the submission is approved based on majority vote
  function determineApproval(uint256 submissionId) internal {
    Submission storage submission = submissions[submissionId];
    uint256 approveCount = 0;

    for (uint256 i = 0; i < submission.selectedReviewers.length; i++) {
      if (submission.votes[submission.selectedReviewers[i]]) {
        approveCount++;
      }
    }

    submission.isApproved = approveCount > submission.selectedReviewers.length / 2;
  }

  // Function to get the commit hash for a specific submission and reviewer
  function getCommitHash(uint256 submissionId, address reviewer) public view returns (bytes32) {
    require(submissionId < submissions.length, "Invalid submission ID");
    Submission storage submission = submissions[submissionId];
    return submission.commits[reviewer];
  }

  // Function to get the vote of a specific reviewer for a given submission
  function getReviewerVote(uint256 submissionId, address reviewer) public view returns (bool) {
    require(submissionId < submissions.length, "Invalid submission ID");
    Submission storage submission = submissions[submissionId];
    require(submission.revealPhase, "Reveal phase not completed");
    return submission.votes[reviewer];
  }

  // Function to check if a submission is approved based on majority vote
  function isApproved(uint256 submissionId) public view returns (bool) {
    require(submissionId < submissions.length, "Invalid submission ID");
    Submission storage submission = submissions[submissionId];
    require(submission.revealPhase, "Reveal phase not completed");

    uint256 approveCount = 0;
    for (uint256 i = 0; i < submission.selectedReviewers.length; i++) {
      if (submission.votes[submission.selectedReviewers[i]]) {
        approveCount++;
      }
    }

    return approveCount > submission.selectedReviewers.length / 2;
  }

  // Function to display how reviewers voted after reveal phase
  function getReviewersVotes(uint256 submissionId) public view returns (address[] memory, bool[] memory) {
    require(submissionId < submissions.length, "Invalid submission ID");
    Submission storage submission = submissions[submissionId];
    require(submission.revealPhase, "Reveal phase not completed");

    address[] memory selectedReviewers = submission.selectedReviewers;
    bool[] memory votes = new bool[](selectedReviewers.length);

    for (uint256 i = 0; i < selectedReviewers.length; i++) {
      votes[i] = submission.votes[selectedReviewers[i]];
    }

    return (selectedReviewers, votes);
  }

  // Get all approved reviewers for a submission
  function getApprovedReviewers(uint256 submissionId) public view returns (address[] memory) {
    require(submissionId < submissions.length, "Invalid submission ID");
    Submission storage submission = submissions[submissionId];
    require(submission.revealPhase, "Reveal phase not completed");

    address[] memory approvedReviewers = new address[](submission.selectedReviewers.length);
    uint256 count = 0;

    for (uint256 i = 0; i < submission.selectedReviewers.length; i++) {
      if (submission.votes[submission.selectedReviewers[i]]) {
        approvedReviewers[count] = submission.selectedReviewers[i];
        count++;
      }
    }

    // Resize the array to fit the actual number of approved reviewers
    address[] memory resizedApprovedReviewers = new address[](count);
    for (uint256 i = 0; i < count; i++) {
      resizedApprovedReviewers[i] = approvedReviewers[i];
    }

    return resizedApprovedReviewers;
  }

  // Function to create a commit hash for a true vote
  function createCommitHashTrue(bytes32 salt) public pure returns (bytes32) {
    return keccak256(abi.encodePacked(true, salt));
  }

  // Function to create a commit hash for a false vote
  function createCommitHashFalse(bytes32 salt) public pure returns (bytes32) {
    return keccak256(abi.encodePacked(false, salt));
  }

  // Function to get the number of reviewers
  function getReviewersCount() public view returns (uint256) {
    return reviewers.length;
  }

  // Function to get the number of authors
  function getAuthorsCount() public view returns (uint256) {
    return authors.length;
  }

  // Function to get keywords for a specific reviewer
  function getReviewerKeywords(address reviewerAddress) public view returns (string[] memory) {
    for (uint256 i = 0; i < reviewers.length; i++) {
      if (reviewers[i].addr == reviewerAddress) {
        return reviewers[i].keywords;
      }
    }
    revert("Reviewer not found.");
  }
}
