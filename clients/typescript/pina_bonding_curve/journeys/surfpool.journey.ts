/**
 * Drives every Pina Bonding Curve instruction against one live Surfnet,
 * end to end, through the generated TypeScript client.
 *
 * The Rust test `typescript_client_runs_every_instruction` starts the
 * Surfnet with the curve and the pinned Pina AMM deployed, prepares the AMM
 * tier, the token fixtures, and every derived address, and then runs this
 * script:
 *
 * ```sh
 * pnpm --dir clients/typescript/pina_bonding_curve run journey -- --rpc <url> --fixtures <file>
 * ```
 *
 * The script asserts that the client's own PDA helpers reproduce the
 * Rust-derived addresses before sending anything, then runs all nine
 * instructions in order.
 */

import { readFileSync } from "node:fs";

import {
	type Address,
	appendTransactionMessageInstructions,
	createDefaultRpcTransport,
	createKeyPairSignerFromBytes,
	createSolanaRpcFromTransport,
	createTransactionMessage,
	generateKeyPairSigner,
	getBase64EncodedWireTransaction,
	type Instruction,
	type KeyPairSigner,
	pipe,
	setTransactionMessageFeePayerSigner,
	setTransactionMessageLifetimeUsingBlockhash,
	signTransactionMessageWithSigners,
} from "@solana/kit";

import {
	findLaunchConfigPda,
	findLaunchPda,
	findLaunchVaultPda,
	getBuyInstruction,
	getClaimCreatorAllocationInstruction,
	getClaimCreatorFeesInstruction,
	getClaimPartnerFeesInstruction,
	getCreateConfigInstruction,
	getCreateLaunchInstruction,
	getGraduateInstruction,
	getSellInstruction,
	getSetLaunchCreatorInstruction,
} from "../src/index.js";

/** The original SPL Token program, the program of every quote account. */
const TOKEN_PROGRAM_ADDRESS =
	"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" as Address<
		"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
	>;
/** Token-2022, the program of every launch's base mint. */
const TOKEN_2022_PROGRAM_ADDRESS =
	"TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb" as Address<
		"TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
	>;

interface JourneyFixtures {
	partnerKey: number[];
	creatorKey: number[];
	configIndex: number;
	quoteMint: string;
	baseMint: string;
	creatorQuote: string;
	creatorBase: string;
	partnerQuote: string;
	launch: string;
	baseVault: string;
	quoteVault: string;
	ammAuthority: string;
	ammProgram: string;
	ammConfig: string;
	pool: string;
	lpMint: string;
	poolVault0: string;
	poolVault1: string;
	launchLpToken: string;
	terms: {
		threshold: number;
		totalSupply: number;
		startFeeRate: number;
		feeDecay: number;
	};
}

function assertEqual(actual: string, expected: string, label: string): void {
	if (actual !== expected) {
		throw new Error(
			`${label}: client derived ${actual}, fixtures say ${expected}`,
		);
	}
}

async function main(): Promise<void> {
	const flag = (name: string): string => {
		const index = process.argv.indexOf(name);
		const value = index === -1 ? undefined : process.argv[index + 1];
		if (value === undefined) {
			throw new Error(
				`usage: journey --rpc <url> --fixtures <file> (missing ${name})`,
			);
		}
		return value;
	};
	const rpcUrl = flag("--rpc");
	const fixturesPath = flag("--fixtures");
	const fixtures: JourneyFixtures = JSON.parse(
		readFileSync(fixturesPath, "utf8"),
	) as JourneyFixtures;

	const rpc = createSolanaRpcFromTransport(
		createDefaultRpcTransport({ url: rpcUrl }),
	);

	// The Rust side funds both signers before the journey starts.
	const partner = await createKeyPairSignerFromBytes(
		new Uint8Array(fixtures.partnerKey),
	);
	const creator = await createKeyPairSignerFromBytes(
		new Uint8Array(fixtures.creatorKey),
	);

	const quoteMint = fixtures.quoteMint as Address;
	const baseMint = fixtures.baseMint as Address;

	// The client must derive exactly the addresses the Rust side derived.
	assertEqual(
		(
			await findLaunchPda({
				baseMint,
			})
		)[0],
		fixtures.launch,
		"launch PDA",
	);
	assertEqual(
		(
			await findLaunchVaultPda({
				launch: fixtures.launch as Address,
				mint: baseMint,
			})
		)[0],
		fixtures.baseVault,
		"base vault PDA",
	);

	const config = (await import("../src/index.js")).findLaunchConfigPda;
	const launch = fixtures.launch as Address;
	const baseVault = fixtures.baseVault as Address;
	const quoteVault = fixtures.quoteVault as Address;

	async function sendAll(
		payer: KeyPairSigner,
		instructions: Instruction[],
	): Promise<void> {
		const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
		const message = pipe(
			createTransactionMessage({ version: 0 }),
			(message) => setTransactionMessageFeePayerSigner(payer, message),
			(message) =>
				setTransactionMessageLifetimeUsingBlockhash(
					{
						blockhash: latestBlockhash.blockhash,
						lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
					},
					message,
				),
			(message) => appendTransactionMessageInstructions(instructions, message),
		);
		const transaction = await signTransactionMessageWithSigners(message);
		const signature = await rpc
			.sendTransaction(getBase64EncodedWireTransaction(transaction), {
				encoding: "base64",
			})
			.send();
		for (let attempt = 0; attempt < 60; attempt += 1) {
			const statuses = await rpc.getSignatureStatuses([signature]).send();
			const status = statuses.value[0];
			if (
				status?.confirmationStatus === "confirmed" ||
				status?.confirmationStatus === "finalized"
			) {
				if (status.err !== null) {
					throw new Error(
						`transaction ${signature} failed: ${JSON.stringify(status.err)}`,
					);
				}
				return;
			}
			if (status?.err !== null && status?.err !== undefined) {
				throw new Error(
					`transaction ${signature} failed: ${JSON.stringify(status.err)}`,
				);
			}
			await new Promise((resolve) => setTimeout(resolve, 100));
		}
		throw new Error(`transaction ${signature} never confirmed`);
	}

	// 1. CreateConfig — the partner opens a configuration on the prepared
	// AMM tier. The terms mirror the suite's reference configuration.
	const sqrtStartPrice = 97_610_000_000_000_000n;
	const curveUppers = [sqrtStartPrice * 8n, ...Array<bigint>(15).fill(0n)];
	const curveLiquidities = [5_000_000_000_000n, ...Array<bigint>(15).fill(0n)];
	const configPda =
		(await findLaunchConfigPda({ authority: partner.address, index: 0n }))[0];
	const createConfig = getCreateConfigInstruction({
		payer: partner,
		authority: partner,
		config: configPda as Address,
		quoteMint,
		quoteTokenProgram: TOKEN_PROGRAM_ADDRESS,
		ammConfig: fixtures.ammConfig as Address,
		index: 0n,
		sqrtStartPrice,
		curveSqrtPrices: curveUppers,
		curveLiquidities,
		migrationQuoteThreshold: BigInt(fixtures.terms.threshold),
		totalSupply: BigInt(fixtures.terms.totalSupply),
		creatorAllocation: 50_000_000_000_000n,
		creatorVestingCliff: 0n,
		creatorVestingDuration: 0n,
		feeDecayDuration: BigInt(fixtures.terms.feeDecay),
		startFeeRate: fixtures.terms.startFeeRate,
		endFeeRate: 10_000,
		creatorFeeShare: 500_000,
		migrationFeeRate: 20_000,
		creatorLpShare: 0,
		partnerLpShare: 0,
		segmentCount: 1,
		baseDecimals: 6,
		poolCreatorMode: 0,
	});
	await sendAll(partner, [createConfig]);

	// 2. CreateLaunch — the creator launches on the new configuration.
	await sendAll(
		creator,
		[
			getCreateLaunchInstruction({
				payer: creator,
				creator,
				config: configPda as Address,
				baseMint,
				quoteMint,
				launch,
				baseVault,
				quoteVault,
				baseTokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
				quoteTokenProgram: TOKEN_PROGRAM_ADDRESS,
				activationTime: 0n,
			}),
		],
	);

	// 3. Buy — spend quote along the curve.
	await sendAll(
		creator,
		[
			getBuyInstruction({
				trader: creator,
				config: configPda as Address,
				launch,
				traderBase: fixtures.creatorBase as Address,
				traderQuote: fixtures.creatorQuote as Address,
				baseVault,
				quoteVault,
				baseTokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
				quoteTokenProgram: TOKEN_PROGRAM_ADDRESS,
				quoteAmountIn: 2_000_000_000n,
				minimumBaseOut: 0n,
			}),
		],
	);

	// 4. Sell — return some base to the curve.
	await sendAll(
		creator,
		[
			getSellInstruction({
				trader: creator,
				config: configPda as Address,
				launch,
				traderBase: fixtures.creatorBase as Address,
				traderQuote: fixtures.creatorQuote as Address,
				baseVault,
				quoteVault,
				baseTokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
				quoteTokenProgram: TOKEN_PROGRAM_ADDRESS,
				baseAmountIn: 100_000_000n,
				minimumQuoteOut: 0n,
			}),
		],
	);

	// 5. Graduate — complete the launch with a large buy and move it to the
	// AMM in the same step: the completing buy and Graduate are two
	// instructions on one journey, sent separately.
	await sendAll(
		creator,
		[
			getBuyInstruction({
				trader: creator,
				config: configPda as Address,
				launch,
				traderBase: fixtures.creatorBase as Address,
				traderQuote: fixtures.creatorQuote as Address,
				baseVault,
				quoteVault,
				baseTokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
				quoteTokenProgram: TOKEN_PROGRAM_ADDRESS,
				quoteAmountIn: 500_000_000_000n,
				minimumBaseOut: 0n,
			}),
		],
	);
	await sendAll(
		creator,
		[
			getGraduateInstruction({
				payer: creator,
				config: configPda as Address,
				launch,
				baseMint,
				quoteMint,
				baseVault,
				quoteVault,
				ammAuthority: fixtures.ammAuthority as Address,
				ammProgram: fixtures.ammProgram as Address,
				ammConfig: fixtures.ammConfig as Address,
				pool: fixtures.pool as Address,
				lpMint: fixtures.lpMint as Address,
				poolVault0: fixtures.poolVault0 as Address,
				poolVault1: fixtures.poolVault1 as Address,
				launchLpToken: fixtures.launchLpToken as Address,
				baseTokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
				quoteTokenProgram: TOKEN_PROGRAM_ADDRESS,
			}),
		],
	);

	// 6. ClaimPartnerFees — the partner empties its share.
	await sendAll(
		partner,
		[
			getClaimPartnerFeesInstruction({
				claimant: partner,
				config: configPda as Address,
				launch,
				quoteVault,
				destination: fixtures.partnerQuote as Address,
				quoteTokenProgram: TOKEN_PROGRAM_ADDRESS,
			}),
		],
	);

	// 7. ClaimCreatorFees — the creator empties its share.
	await sendAll(
		creator,
		[
			getClaimCreatorFeesInstruction({
				claimant: creator,
				config: configPda as Address,
				launch,
				quoteVault,
				destination: fixtures.creatorQuote as Address,
				quoteTokenProgram: TOKEN_PROGRAM_ADDRESS,
			}),
		],
	);

	// 8. ClaimCreatorAllocation — with zero vesting, everything is claimable.
	await sendAll(
		creator,
		[
			getClaimCreatorAllocationInstruction({
				creator,
				config: configPda as Address,
				launch,
				baseVault,
				destination: fixtures.creatorBase as Address,
				baseTokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
			}),
		],
	);

	// 9. SetLaunchCreator — hand the launch's creator rights to a fresh wallet.
	const successor = await generateKeyPairSigner();
	await sendAll(
		creator,
		[
			getSetLaunchCreatorInstruction({
				creator,
				launch,
				newCreator: successor.address,
			}),
		],
	);

	console.log("typescript journey: all nine instructions confirmed");
}

await main();
