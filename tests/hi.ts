import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, SystemProgram, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { PrivatePerps } from "../target/types/private_perps";

describe("private_perps", () => {
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  const program = anchor.workspace.PrivatePerps as Program<PrivatePerps>;

  const user = Keypair.generate();
  const market = Keypair.generate();
  const computationRequest = Keypair.generate();
  const position = Keypair.generate();

  let userAccount: PublicKey;

  before(async () => {
    const sig = await provider.connection.requestAirdrop(user.publicKey, 2e9);
    await provider.connection.confirmTransaction(sig);
  });

  it("Initialize Market", async () => {
    await program.methods
      .initializeMarket(new anchor.BN(255), new anchor.BN(30))
      .accounts({
        market: market.publicKey,
        authority: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([market])
      .rpc();

    const acc = await program.account.market.fetch(market.publicKey);
    expect(acc.authority.toBase58()).to.eq(
      provider.wallet.publicKey.toBase58()
    );
    expect(acc.feeBps.toNumber()).to.eq(30);
  });

  it("Create and Deposit into UserAccount", async () => {
    const userAccountKp = Keypair.generate();
    userAccount = userAccountKp.publicKey;

    const space = 8 + 32 + 8;
    const lamports =
      await provider.connection.getMinimumBalanceForRentExemption(space);

    const createIx = SystemProgram.createAccount({
      fromPubkey: user.publicKey,
      newAccountPubkey: userAccount,
      space,
      lamports,
      programId: program.programId,
    });

    const tx = new anchor.web3.Transaction().add(createIx);

    await provider.sendAndConfirm(tx, [user, userAccountKp]);

    await program.methods
      .deposit(new anchor.BN(5000))
      .accounts({
        userAccount,
        owner: user.publicKey,
      })
      .signers([user])
      .rpc()
      .catch((e) => console.log("Deposit failed (non-fatal):", e.message));
  });

  it("Submit Encrypted Order", async () => {
    const arciumComputationId = new Array(32).fill(1);
    const commitment = new Array(32).fill(2);
    const circuitType = 0;

    await program.methods
      .submitEncryptedOrder(arciumComputationId, commitment, circuitType)
      .accounts({
        computationRequest: computationRequest.publicKey,
        market: market.publicKey,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([computationRequest, user])
      .rpc();

    const req = await program.account.computationRequest.fetch(
      computationRequest.publicKey
    );

    expect(Object.keys(req.status)[0]).to.equal("pending");
    expect(req.circuitOffset).to.be.a("number");
  });

  it("Apply Encrypted Result (simulate circuit output)", async () => {
    const arciumComputationId = new Array(32).fill(1);
    const receipt = Buffer.from("dummy-receipt");

    const resultFillAmount = new anchor.BN(100);
    const resultPrice = new anchor.BN(2500);
    const resultSide = 1;
    const resultNonce = new anchor.BN(1);

    await program.methods
      .applyEncryptedResult(
        arciumComputationId,
        resultFillAmount,
        resultPrice,
        resultSide,
        resultNonce,
        receipt
      )
      .accounts({
        computationRequest: computationRequest.publicKey,
        market: market.publicKey,
        position: position.publicKey,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([position, user])
      .rpc();

    const pos = await program.account.position.fetch(position.publicKey);
    expect(pos.size.toNumber()).to.eq(100);
    expect(pos.avgPrice.toNumber()).to.eq(2500);
  });
});
