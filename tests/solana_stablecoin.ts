// factory.test.ts
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaStablecoin } from "../target/types/solana_stablecoin";
import { expect } from "chai";
import { assert } from "chai";

describe("Solana Stablecoin Factory", () => {
  // Configure the client to use the devnet cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .SolanaStablecoin as Program<SolanaStablecoin>;

  // Generate a new keypair for admin
  const admin = anchor.web3.Keypair.generate();
  
  // Find PDA for factory state
  const [factoryState, factoryBump] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [Buffer.from("factory_state")],
      program.programId
    );

  before(async () => {
    // Airdrop SOL to admin for transactions
    const signature = await provider.connection.requestAirdrop(
      admin.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);
  });

  it("Initializes the factory", async () => {
    try {
      const minCollateralRatio = 150; // 150%
      const baseFeeRate = 30; // 0.3%

      await program.methods
        .initializeFactory(
          new anchor.BN(minCollateralRatio),
          baseFeeRate
        )
        .accounts({
          admin: admin.publicKey,
          factoryState,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      // Fetch the factory state and verify
      const factoryData = await program.account.factoryState.fetch(
        factoryState
      );

      assert.strictEqual(
        factoryData.admin.toString(),
        admin.publicKey.toString(),
        "Admin public key should match"
      );
      assert.strictEqual(
        factoryData.minCollateralRatio.toNumber(),
        minCollateralRatio,
        "Collateral ratio should match"
      );
      assert.strictEqual(
        factoryData.baseFeeRate,
        baseFeeRate,
        "Base fee rate should match"
      );
      assert.strictEqual(
        factoryData.isPaused,
        false,
        "Factory should not be paused initially"
      );
      assert.strictEqual(
        factoryData.totalStablecoins,
        0,
        "Initial stablecoin count should be 0"
      );
    } catch (error) {
      console.error("Error:", error);
      throw error;
    }
  });

  it("Updates factory configuration", async () => {
    try {
      const newAdmin = anchor.web3.Keypair.generate();
      const newCollateralRatio = 200; // 200%
      const newFeeRate = 50; // 0.5%
      const newFeeRecipient = anchor.web3.Keypair.generate().publicKey;

      await program.methods
        .updateFactoryConfig(
          newAdmin.publicKey,
          new anchor.BN(newCollateralRatio),
          newFeeRate,
          newFeeRecipient
        )
        .accounts({
          admin: admin.publicKey,
          newAdmin: newAdmin.publicKey, 
          factoryState,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      // Fetch and verify updated state
      const factoryData = await program.account.factoryState.fetch(
        factoryState
      );

      assert.strictEqual(
        factoryData.admin.toString(),
        newAdmin.publicKey.toString(),
        "New admin should be set"
      );
      assert.strictEqual(
        factoryData.minCollateralRatio.toNumber(),
        newCollateralRatio,
        "New collateral ratio should be set"
      );
      assert.strictEqual(
        factoryData.baseFeeRate,
        newFeeRate,
        "New fee rate should be set"
      );
      assert.strictEqual(
        factoryData.feeRecipient.toString(),
        newFeeRecipient.toString(),
        "New fee recipient should be set"
      );
    } catch (error) {
      console.error("Error:", error);
      throw error;
    }
  });

  it("Fails to update factory with unauthorized admin", async () => {
    const unauthorizedAdmin = anchor.web3.Keypair.generate();
    const newCollateralRatio = 180;

    try {
      await program.methods
        .updateFactoryConfig(
          null,
          new anchor.BN(newCollateralRatio),
          null,
          null
        )
        .accounts({
          admin: unauthorizedAdmin.publicKey,
          factoryState, 
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([unauthorizedAdmin])
        .rpc();
      
      // If we reach here, the call succeeded when it should have failed
      assert.fail("Expected an error but call succeeded");
    } catch (error) {
      // Verify that the error is what we expect
      assert.include(
        error.toString(),
        "Unauthorized",
        "Should fail with unauthorized error"
      );
    }
  });
});