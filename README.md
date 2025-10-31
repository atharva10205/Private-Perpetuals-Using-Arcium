Private Perps (Anchor + Arcium)

Private Perps is a Solana program built using Anchor and Arcium.
It shows how confidential order flow can work using encrypted off-chain computation.

The main logic of the program runs on-chain with Anchor.
Sensitive calculations, such as order evaluation, are handled off-chain using Arcium’s confidential computing.
After Arcium processes the encrypted data, the results are verified and applied back on-chain.

Tech Stack

Anchor 0.32.x (Solana program framework)
Arcium 0.3.x (confidential computing co-processor)
Rust 1.75+ (2021 edition)
TypeScript tests (Mocha and Chai)

Project Structure

programs/Perpetual – Anchor program module (private_perps)
encrypted-ixs – Arcium circuits for confidential instructions
artifacts – localnet configs, generated accounts, and Docker setup for Arcium
build – compiled circuits and TypeScript bindings
tests – test files written in TypeScript
Anchor.toml – Anchor configuration file
Arcium.toml – Arcium configuration file


Run:

yarn / npm install
Configure Program ID

In Anchor.toml, set your program ID under [programs.localnet]:

[programs.localnet]
hi = "HVFgsYknF4UZuTTeHBpQFZYGvHjYK347mtxSfhseJ2ir"


The module name is private_perps and its IDL name is PrivatePerps.
Tests import it from target/types/private_perps.

Build

Build the Anchor program:
anchor build
arcium build

Build the Arcium circuits:
cargo build -p encrypted-ixs --release


This compiles the encrypted instructions and places the results in the build/ folder.
You will find files such as:

open_position_v1.ts

close_position_v1.ts

.arcis files

Deploy to Localnet

Start a local Solana validator using solana-test-validator or anchor localnet, then deploy:

anchor deploy

Run Tests

Tests are written with Mocha and Chai.
Run them using:

yarn test
or
anchor test

Tests cover:

Market initialization (initialize_market)
User account creation and collateral deposit (deposit)
Submitting encrypted orders (submit_encrypted_order)
Applying encrypted computation results (apply_encrypted_result)
Key Accounts

Market – holds market data such as authority, fees, and open interest
UserAccount – stores user information and collateral
ComputationRequest – stores Arcium computation metadata
Position – holds user’s position size and average price
Encrypted Order Flow

The client encrypts order data and sends it to Arcium for processing.
The program records a ComputationRequest with a circuit offset.
When Arcium completes, the client calls apply_encrypted_result with the results.
The program verifies and updates the on-chain state.


Make sure Docker is running before starting the Arcium localnet.



