# TypeScript client

`@pina-rs/bonding-curve` is generated from the program's IDL for [`@solana/kit`](https://github.com/anza-xyz/kit). It ships ESM and CommonJS builds with bundled type declarations.

```sh
pnpm add @pina-rs/bonding-curve @solana/kit
# Optional, for token accounts, wrapped SOL, and new mints:
pnpm add @solana-program/token @solana-program/system
```

## What is exported

| Kind         | Examples                                                                                                    |
| ------------ | ----------------------------------------------------------------------------------------------------------- |
| Program      | `PINA_BONDING_CURVE_PROGRAM_ADDRESS`, `PinaBondingCurveInstruction`, `parsePinaBondingCurveInstruction`     |
| Instructions | `getBuyInstruction`, `getSellInstruction`, `getCreateLaunchInstructionAsync`, `getGraduateInstruction`      |
| Data codecs  | `getBuyInstructionDataEncoder`, `getLaunchDecoder`, `getLaunchConfigDecoder`, ...                           |
| Accounts     | `fetchLaunch`, `fetchMaybeLaunch`, `fetchLaunchConfig`, `fetchLaunchFromSeeds`, `decodeLaunch`, ...         |
| PDAs         | `findLaunchConfigPda`, `findLaunchPda`, `findLaunchVaultPda`, `findAmmAuthorityPda`                         |
| Events       | `parsePinaBondingCurveEventsFromLogs`, `decodeTradedEvent`, `TradedEvent`, ...                              |
| Errors       | `PINA_BONDING_CURVE_ERROR__SLIPPAGE_EXCEEDED`, `getPinaBondingCurveErrorMessage`, `isPinaBondingCurveError` |

## Units

| Value                                                               | Unit                                                                               |
| ------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| Token amounts                                                       | Base units of their mint, as `bigint`: `1_000_000n` is one 6-decimal token         |
| Prices                                                              | Quote base units per base base unit                                                |
| `sqrtPrice`, `sqrtStartPrice`, `migrationSqrtPrice`                 | Q64.64 square root of the price: `sqrt(price) * 2^64`, stored as a `u128` `bigint` |
| Fee rates and shares                                                | Parts per million, as `number`: `10_000` is 1%                                     |
| `activationTime`                                                    | Unix seconds                                                                       |
| `creatorVestingCliff`, `creatorVestingDuration`, `feeDecayDuration` | Seconds                                                                            |
| `Launch.status`                                                     | `0` trading, `1` completed, `2` graduated                                          |

[curves.md](curves.md) explains the price model and its formulas.

## Addresses

| Account        | Seeds                                    | Owner                         |
| -------------- | ---------------------------------------- | ----------------------------- |
| `LaunchConfig` | `["config", authority, index as u64 LE]` | Curve program                 |
| `Launch`       | `["launch", base_mint]`                  | Curve program                 |
| Launch vault   | `["launch_vault", launch, mint]`         | Token program, held by launch |
| AMM authority  | `["amm_authority"]`                      | None (signs pool creation)    |

```ts
import {
	findAmmAuthorityPda,
	findLaunchConfigPda,
	findLaunchPda,
	findLaunchVaultPda,
} from "@pina-rs/bonding-curve";

// A partner numbers its configurations from 0.
const [configAddress] = await findLaunchConfigPda({
	authority: partner,
	index: 0n,
});
// One launch per base mint.
const [launchAddress] = await findLaunchPda({ baseMint });
// The launch's token accounts. Both are also stored on the launch.
const [baseVault] = await findLaunchVaultPda({
	launch: launchAddress,
	mint: baseMint,
});
const [quoteVault] = await findLaunchVaultPda({
	launch: launchAddress,
	mint: quoteMint,
});
// Signs pool creation in the Pina AMM at graduation.
const [ammAuthority] = await findAmmAuthorityPda();
```

## Read a launch

```ts
import { fetchLaunch, fetchLaunchConfig } from "@pina-rs/bonding-curve";
import { createSolanaRpc } from "@solana/kit";

const rpc = createSolanaRpc("https://api.mainnet-beta.solana.com");

const launch = await fetchLaunch(rpc, launchAddress);
const config = await fetchLaunchConfig(rpc, launch.data.config);
console.log(
	launch.data.status,
	launch.data.quoteReserve,
	launch.data.sqrtPrice,
);
```

`fetchLaunch` throws when the account is missing and checks the discriminator and schema version. `fetchMaybeLaunch` returns an account whose `exists` is `false` instead, which answers "has this mint launched?", and `fetchLaunchFromSeeds(rpc, { baseMint })` derives and fetches in one call. The decoders do **not** check the owner. Reading an address you derived from the program, or the `config` a program-owned launch points at, is enough. When an address comes from outside your app, compare the owner first:

```ts
import {
	decodeLaunch,
	PINA_BONDING_CURVE_PROGRAM_ADDRESS,
} from "@pina-rs/bonding-curve";
import { fetchEncodedAccount } from "@solana/kit";

const encoded = await fetchEncodedAccount(rpc, untrustedAddress);
if (
	!encoded.exists ||
	encoded.programAddress !== PINA_BONDING_CURVE_PROGRAM_ADDRESS
) {
	throw new Error("not a Pina Bonding Curve account");
}
const checked = decodeLaunch(encoded);
```

A launch carries everything a launch page needs. These helpers mirror the program's own arithmetic:

```ts
/** Price in quote base units per base base unit. For display only. */
function priceFromSqrt(sqrtPrice: bigint): number {
	const root = Number(sqrtPrice) / 2 ** 64;
	return root * root;
}

/** Percent of the raise collected so far. */
const progress = Number(
	(launch.data.quoteReserve * 10_000n) / config.data.migrationQuoteThreshold,
) / 100;

/** The trading fee, in parts per million, at `now` in Unix seconds. */
function feeRateAt(now: bigint): number {
	const { startFeeRate, endFeeRate, feeDecayDuration } = config.data;
	if (feeDecayDuration === 0n || startFeeRate <= endFeeRate) {
		return endFeeRate;
	}
	const activation = launch.data.activationTime;
	const elapsed = now > activation ? now - activation : 0n;
	if (elapsed >= feeDecayDuration) {
		return endFeeRate;
	}
	const decayed = (BigInt(startFeeRate - endFeeRate) * elapsed) /
		feeDecayDuration;
	return startFeeRate - Number(decayed);
}
```

Multiply `priceFromSqrt(...)` by `10 ** (baseDecimals - quoteDecimals)` for the price of one whole token. [fees.md](fees.md) describes the fee schedule.

## Token accounts

Traders hold the base and quote tokens in associated token accounts. The token program is part of an associated token account's address, and either mint may be SPL Token or Token-2022, so read each mint's owner:

```ts
import { findAssociatedTokenPda } from "@solana-program/token";
import {
	type Address,
	assertAccountExists,
	fetchEncodedAccount,
} from "@solana/kit";

/** The token program that owns `mint`: SPL Token or Token-2022. */
async function tokenProgramOf(mint: Address): Promise<Address> {
	const account = await fetchEncodedAccount(rpc, mint);
	assertAccountExists(account);
	return account.programAddress;
}

const baseTokenProgram = await tokenProgramOf(launch.data.baseMint);
const quoteTokenProgram = await tokenProgramOf(launch.data.quoteMint);
const [walletBase] = await findAssociatedTokenPda({
	owner: wallet.address,
	mint: launch.data.baseMint,
	tokenProgram: baseTokenProgram,
});
const [walletQuote] = await findAssociatedTokenPda({
	owner: wallet.address,
	mint: launch.data.quoteMint,
	tokenProgram: quoteTokenProgram,
});
```

Pass the same token program as `tokenProgram` to `getCreateAssociatedTokenIdempotentInstructionAsync` and as the instruction's `baseTokenProgram` or `quoteTokenProgram`. `TOKEN_2022_PROGRAM_ADDRESS` lives in `@solana-program/token-2022` when you need the constant.

## Send and simulate

The rest of this guide sends instructions with these two helpers. `simulateTrade` is the authoritative quote: it runs the real program against the current state, without asking the wallet to sign, and returns its `Traded` event.

```ts
import {
	parsePinaBondingCurveEventsFromLogs,
	type TradedEvent,
} from "@pina-rs/bonding-curve";
import {
	appendTransactionMessageInstructions,
	assertIsTransactionWithBlockhashLifetime,
	compileTransaction,
	createSolanaRpcSubscriptions,
	createTransactionMessage,
	getBase64EncodedWireTransaction,
	getSignatureFromTransaction,
	getSolanaErrorFromTransactionError,
	type Instruction,
	pipe,
	sendAndConfirmTransactionFactory,
	setTransactionMessageFeePayerSigner,
	setTransactionMessageLifetimeUsingBlockhash,
	type Signature,
	signTransactionMessageWithSigners,
} from "@solana/kit";

const rpcSubscriptions = createSolanaRpcSubscriptions(
	"wss://api.mainnet-beta.solana.com",
);
const sendAndConfirm = sendAndConfirmTransactionFactory({
	rpc,
	rpcSubscriptions,
});

/** A transaction message paid for by `wallet`. */
async function message(instructions: readonly Instruction[]) {
	const { value: blockhash } = await rpc.getLatestBlockhash().send();
	return pipe(
		createTransactionMessage({ version: 0 }),
		(message) => setTransactionMessageFeePayerSigner(wallet, message),
		(message) =>
			setTransactionMessageLifetimeUsingBlockhash(blockhash, message),
		(message) => appendTransactionMessageInstructions(instructions, message),
	);
}

async function send(instructions: readonly Instruction[]): Promise<Signature> {
	const transaction = await signTransactionMessageWithSigners(
		await message(instructions),
	);
	assertIsTransactionWithBlockhashLifetime(transaction);
	await sendAndConfirm(transaction, { commitment: "confirmed" });
	return getSignatureFromTransaction(transaction);
}

/** Simulate without signing and return the trade the program reports. */
async function simulateTrade(
	instructions: readonly Instruction[],
): Promise<TradedEvent> {
	const transaction = compileTransaction(await message(instructions));
	const { value } = await rpc
		.simulateTransaction(getBase64EncodedWireTransaction(transaction), {
			encoding: "base64",
			replaceRecentBlockhash: true,
			sigVerify: false,
		})
		.send();
	if (value.err !== null) {
		throw getSolanaErrorFromTransactionError(value.err);
	}
	const traded = parsePinaBondingCurveEventsFromLogs(value.logs ?? []).find(
		(event) => event.name === "traded",
	);
	if (traded === undefined) {
		throw new Error("the simulated transaction did not trade");
	}
	return traded.data;
}

/** Lower `amount` by `bps` basis points, rounding down. */
function withSlippage(amount: bigint, bps: bigint): bigint {
	return (amount * (10_000n - bps)) / 10_000n;
}
```

Simulate with a zero minimum, then send with the minimum derived from the simulated amount. The trading fee only falls over time, so a trade that lands later never pays a higher fee than it was quoted; the slippage limit protects against other traders moving the price first. A simulation that fails throws the same program errors as a real send (see [Errors](#errors)).

## Buy

`quoteAmountIn` is the most quote to spend, fees included. A buy that reaches the migration price completes the launch and is charged only for the quote it used, so the `Traded` event's `quoteAmount` can be less than `quoteAmountIn`.

When the quote mint is wrapped SOL, the buyer's quote account must hold wSOL: create it, transfer lamports into it, and call `SyncNative`. Closing it after the buy returns any unspent wSOL (and the account's rent) as SOL.

```ts
import { getBuyInstruction } from "@pina-rs/bonding-curve";
import { getTransferSolInstruction } from "@solana-program/system";
import {
	getCloseAccountInstruction,
	getCreateAssociatedTokenIdempotentInstructionAsync,
	getSyncNativeInstruction,
} from "@solana-program/token";
import { address, type Instruction } from "@solana/kit";

const WRAPPED_SOL_MINT = address("So11111111111111111111111111111111111111112");
const quoteIsSol = launch.data.quoteMint === WRAPPED_SOL_MINT;

async function buyInstructions(
	quoteAmountIn: bigint,
	minimumBaseOut: bigint,
): Promise<Instruction[]> {
	const instructions: Instruction[] = [
		await getCreateAssociatedTokenIdempotentInstructionAsync({
			payer: wallet,
			owner: wallet.address,
			mint: launch.data.baseMint,
			tokenProgram: baseTokenProgram,
		}),
	];
	if (quoteIsSol) {
		instructions.push(
			await getCreateAssociatedTokenIdempotentInstructionAsync({
				payer: wallet,
				owner: wallet.address,
				mint: WRAPPED_SOL_MINT,
				tokenProgram: quoteTokenProgram,
			}),
			getTransferSolInstruction({
				source: wallet,
				destination: walletQuote,
				amount: quoteAmountIn,
			}),
			getSyncNativeInstruction({ account: walletQuote }),
		);
	}
	instructions.push(
		getBuyInstruction({
			trader: wallet,
			config: launch.data.config,
			launch: launchAddress,
			traderBase: walletBase,
			traderQuote: walletQuote,
			baseVault: launch.data.baseVault,
			quoteVault: launch.data.quoteVault,
			baseTokenProgram,
			quoteTokenProgram,
			quoteAmountIn,
			minimumBaseOut,
		}),
	);
	if (quoteIsSol) {
		instructions.push(
			getCloseAccountInstruction({
				account: walletQuote,
				destination: wallet.address,
				owner: wallet,
			}),
		);
	}
	return instructions;
}

const quoteAmountIn = 1_000_000_000n; // 1 SOL
const quoted = await simulateTrade(await buyInstructions(quoteAmountIn, 0n));
const minimumBaseOut = withSlippage(quoted.baseAmount, 100n); // 1%
await send(await buyInstructions(quoteAmountIn, minimumBaseOut));
```

Closing the wSOL account unwraps its whole balance, including wSOL the wallet already held. Leave the close out to keep it wrapped.

## Sell

`baseAmountIn` is exact, and `minimumQuoteOut` applies to the quote received **after** the fee. For a sale, the `Traded` event's `quoteAmount` is that net amount.

```ts
import { getSellInstruction } from "@pina-rs/bonding-curve";
import {
	getCloseAccountInstruction,
	getCreateAssociatedTokenIdempotentInstructionAsync,
} from "@solana-program/token";
import type { Instruction } from "@solana/kit";

async function sellInstructions(
	baseAmountIn: bigint,
	minimumQuoteOut: bigint,
): Promise<Instruction[]> {
	const instructions: Instruction[] = [
		await getCreateAssociatedTokenIdempotentInstructionAsync({
			payer: wallet,
			owner: wallet.address,
			mint: launch.data.quoteMint,
			tokenProgram: quoteTokenProgram,
		}),
		getSellInstruction({
			trader: wallet,
			config: launch.data.config,
			launch: launchAddress,
			traderBase: walletBase,
			traderQuote: walletQuote,
			baseVault: launch.data.baseVault,
			quoteVault: launch.data.quoteVault,
			baseTokenProgram,
			quoteTokenProgram,
			baseAmountIn,
			minimumQuoteOut,
		}),
	];
	if (quoteIsSol) {
		// Receive SOL instead of wSOL.
		instructions.push(
			getCloseAccountInstruction({
				account: walletQuote,
				destination: wallet.address,
				owner: wallet,
			}),
		);
	}
	return instructions;
}

const baseAmountIn = 5_000_000_000n;
const sale = await simulateTrade(await sellInstructions(baseAmountIn, 0n));
await send(
	await sellInstructions(baseAmountIn, withSlippage(sale.quoteAmount, 100n)),
);
```

Buys and sells need `status === 0` and a reached `activationTime`; otherwise they fail with `NotTrading` or `NotActive`.

## Create a launch

`CreateLaunch` needs a brand-new base mint: zero supply, no freeze authority, the creator as mint authority, and the configuration's `baseDecimals`. It mints the configuration's fixed `totalSupply` into the base vault and then revokes the mint authority. Create the mint and the launch in one transaction so the mint never exists without its launch:

```ts
import {
	fetchLaunchConfig,
	findLaunchPda,
	findLaunchVaultPda,
	getCreateLaunchInstructionAsync,
} from "@pina-rs/bonding-curve";
import { getCreateAccountInstruction } from "@solana-program/system";
import {
	getInitializeMint2Instruction,
	getMintSize,
	TOKEN_PROGRAM_ADDRESS,
} from "@solana-program/token";
import { generateKeyPairSigner } from "@solana/kit";

const launchpad = await fetchLaunchConfig(rpc, configAddress);
const mint = await generateKeyPairSigner();
const [newLaunch] = await findLaunchPda({ baseMint: mint.address });
const [newBaseVault] = await findLaunchVaultPda({
	launch: newLaunch,
	mint: mint.address,
});
const [newQuoteVault] = await findLaunchVaultPda({
	launch: newLaunch,
	mint: launchpad.data.quoteMint,
});
const space = BigInt(getMintSize());

await send([
	getCreateAccountInstruction({
		payer: wallet,
		newAccount: mint,
		lamports: await rpc.getMinimumBalanceForRentExemption(space).send(),
		space,
		programAddress: TOKEN_PROGRAM_ADDRESS,
	}),
	getInitializeMint2Instruction({
		mint: mint.address,
		decimals: launchpad.data.baseDecimals,
		mintAuthority: wallet.address,
	}),
	await getCreateLaunchInstructionAsync({
		payer: wallet,
		creator: wallet,
		config: configAddress,
		baseMint: mint.address,
		quoteMint: launchpad.data.quoteMint,
		baseVault: newBaseVault,
		quoteVault: newQuoteVault,
		baseTokenProgram: TOKEN_PROGRAM_ADDRESS,
		quoteTokenProgram: await tokenProgramOf(launchpad.data.quoteMint),
		activationTime: 0n,
	}),
]);
```

The async builder derives `launch` from the base mint and defaults the system program; derive the two vaults yourself. `activationTime` is a Unix timestamp: zero or a past time opens trading immediately, and a future time schedules the launch (for example `BigInt(Math.floor(Date.now() / 1000) + 3600)` for an hour from now). Trading opens at `startFeeRate`, so a creator who appends a buy to the same transaction pays the full opening fee too.

- **Token-2022.** Use `TOKEN_2022_PROGRAM_ADDRESS`, `getMintSize`, and `getInitializeMint2Instruction` from `@solana-program/token-2022` and pass the Token-2022 program as `baseTokenProgram`. Only transfer-neutral extensions are accepted: metadata pointer and metadata, group pointer, group, and member, interest-bearing, and scaled UI amount. Anything else fails with `UnsupportedMint`.
- **Metadata.** Add the token's name, symbol, and URI before `CreateLaunch`, because metadata programs need the mint authority that `CreateLaunch` revokes. With Token-2022, put the metadata pointer and metadata extensions on the mint itself.
- **Configurations.** A partner creates the `LaunchConfig` once with `getCreateConfigInstruction`; [launchpads.md](launchpads.md) explains every term. `curveSqrtPrices` and `curveLiquidities` always have 16 entries, with unused segments set to zero.

## Graduate

A buy that reaches the migration price sets `status` to `1` (completed) and emits `Completed`. Anyone can then send `Graduate`, which creates a Pina AMM pool for the pair at the curve's final price, seeds it from the launch's vaults, and splits the LP tokens. The payer covers the rent of the pool's accounts.

Graduation needs the AMM's addresses. `@pina-rs/amm` exports `findPoolPda`, `findPoolVaultPda`, and `findPoolLpMintPda`; the same derivation with `@solana/kit` alone:

```ts
import {
	findAmmAuthorityPda,
	getGraduateInstruction,
} from "@pina-rs/bonding-curve";
import {
	findAssociatedTokenPda,
	TOKEN_PROGRAM_ADDRESS,
} from "@solana-program/token";
import {
	type Address,
	address,
	getAddressEncoder,
	getProgramDerivedAddress,
	getUtf8Encoder,
} from "@solana/kit";

const PINA_AMM_PROGRAM_ADDRESS = address(
	"pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV",
);
const addressEncoder = getAddressEncoder();

/** A Pina AMM PDA from a string seed and address seeds. */
async function ammPda(seed: string, ...seeds: Address[]): Promise<Address> {
	const [pda] = await getProgramDerivedAddress({
		programAddress: PINA_AMM_PROGRAM_ADDRESS,
		seeds: [
			getUtf8Encoder().encode(seed),
			...seeds.map((value) => addressEncoder.encode(value)),
		],
	});
	return pda;
}

/** Sort two mints by their 32 address bytes, the order the AMM uses. */
function sortMints(a: Address, b: Address): [Address, Address] {
	const left = addressEncoder.encode(a);
	const right = addressEncoder.encode(b);
	for (let index = 0; index < 32; index += 1) {
		if (left[index] !== right[index]) {
			return left[index]! < right[index]! ? [a, b] : [b, a];
		}
	}
	throw new Error("a pool needs two different mints");
}

/** An LP token account. LP mints are always SPL Token. */
async function lpTokenOf(owner: Address, lpMint: Address): Promise<Address> {
	const [token] = await findAssociatedTokenPda({
		owner,
		mint: lpMint,
		tokenProgram: TOKEN_PROGRAM_ADDRESS,
	});
	return token;
}

const [mint0, mint1] = sortMints(launch.data.baseMint, launch.data.quoteMint);
const pool = await ammPda("pool", config.data.ammConfig, mint0, mint1);
const lpMint = await ammPda("pool_lp_mint", pool);
const [ammAuthority] = await findAmmAuthorityPda();
const payCreator = config.data.creatorLpShare > 0;
const payPartner = config.data.partnerLpShare > 0;

await send([
	getGraduateInstruction({
		payer: wallet,
		config: launch.data.config,
		launch: launchAddress,
		baseMint: launch.data.baseMint,
		quoteMint: launch.data.quoteMint,
		baseVault: launch.data.baseVault,
		quoteVault: launch.data.quoteVault,
		ammAuthority,
		ammProgram: PINA_AMM_PROGRAM_ADDRESS,
		ammConfig: config.data.ammConfig,
		pool,
		lpMint,
		poolVault0: await ammPda("pool_vault", pool, mint0),
		poolVault1: await ammPda("pool_vault", pool, mint1),
		launchLpToken: await lpTokenOf(launchAddress, lpMint),
		baseTokenProgram,
		quoteTokenProgram,
		creator: payCreator ? launch.data.creator : undefined,
		creatorLpToken: payCreator
			? await lpTokenOf(launch.data.creator, lpMint)
			: undefined,
		partner: payPartner ? config.data.authority : undefined,
		partnerLpToken: payPartner
			? await lpTokenOf(config.data.authority, lpMint)
			: undefined,
	}),
]);
```

`creator`/`creatorLpToken` and `partner`/`partnerLpToken` are required only when the configuration's `creatorLpShare` or `partnerLpShare` is above zero; a missing pair fails with `MissingLpAccount`. Leaving them out fills their slots with the program address, which the program reads as absent. The program creates those LP token accounts itself. The LP program, associated token program, and system program have defaults. [graduation.md](graduation.md) explains where every token goes.

## Claim fees and the creator allocation

Trading fees and the migration fee accrue in the launch's quote vault, split between creator and partner by `creatorFeeShare`, until claimed. Each claim pays everything accrued to any token account of the right mint, works before and after graduation, and fails with `NothingToClaim` when there is nothing to pay. Create the destination first:

```ts
import {
	getClaimCreatorFeesInstruction,
	getClaimPartnerFeesInstruction,
} from "@pina-rs/bonding-curve";
import { getCreateAssociatedTokenIdempotentInstructionAsync } from "@solana-program/token";

const createQuoteAccount =
	await getCreateAssociatedTokenIdempotentInstructionAsync({
		payer: wallet,
		owner: wallet.address,
		mint: launch.data.quoteMint,
		tokenProgram: quoteTokenProgram,
	});

// Signed by the configuration's authority (the partner).
if (launch.data.partnerFees > 0n) {
	await send([
		createQuoteAccount,
		getClaimPartnerFeesInstruction({
			claimant: wallet,
			config: launch.data.config,
			launch: launchAddress,
			quoteVault: launch.data.quoteVault,
			destination: walletQuote,
			quoteTokenProgram,
		}),
	]);
}

// Signed by the launch's creator.
if (launch.data.creatorFees > 0n) {
	await send([
		createQuoteAccount,
		getClaimCreatorFeesInstruction({
			claimant: wallet,
			config: launch.data.config,
			launch: launchAddress,
			quoteVault: launch.data.quoteVault,
			destination: walletQuote,
			quoteTokenProgram,
		}),
	]);
}
```

The creator allocation vests linearly over `creatorVestingDuration` seconds, starting `creatorVestingCliff` seconds after `activationTime`. This mirrors the program:

```ts
import { getClaimCreatorAllocationInstruction } from "@pina-rs/bonding-curve";
import { getCreateAssociatedTokenIdempotentInstructionAsync } from "@solana-program/token";

/** Base the creator can claim at `now` in Unix seconds. */
function claimableAllocation(now: bigint): bigint {
	const { creatorAllocation, creatorVestingCliff, creatorVestingDuration } =
		config.data;
	const start = launch.data.activationTime + creatorVestingCliff;
	if (now < start) {
		return 0n;
	}
	const elapsed = now - start < creatorVestingDuration
		? now - start
		: creatorVestingDuration;
	const vested = creatorVestingDuration === 0n
		? creatorAllocation
		: (creatorAllocation * elapsed) / creatorVestingDuration;
	return vested - launch.data.creatorClaimed;
}

if (claimableAllocation(BigInt(Math.floor(Date.now() / 1000))) > 0n) {
	await send([
		await getCreateAssociatedTokenIdempotentInstructionAsync({
			payer: wallet,
			owner: wallet.address,
			mint: launch.data.baseMint,
			tokenProgram: baseTokenProgram,
		}),
		getClaimCreatorAllocationInstruction({
			creator: wallet,
			config: launch.data.config,
			launch: launchAddress,
			baseVault: launch.data.baseVault,
			destination: walletBase,
			baseTokenProgram,
		}),
	]);
}
```

`getSetLaunchCreatorInstruction({ creator: wallet, launch: launchAddress, newCreator })` hands everything the creator receives from then on (unclaimed and future fees, the rest of the allocation, and graduation LP) to another address, such as a multisig.

## Events

```ts
import { parsePinaBondingCurveEventsFromLogs } from "@pina-rs/bonding-curve";

const transaction = await rpc
	.getTransaction(signature, {
		encoding: "json",
		maxSupportedTransactionVersion: 0,
	})
	.send();
for (
	const event of parsePinaBondingCurveEventsFromLogs(
		transaction?.meta?.logMessages ?? [],
	)
) {
	switch (event.name) {
		case "traded":
			console.log(
				event.data.isBuy === 1 ? "buy" : "sell",
				event.data.baseAmount,
			);
			break;
		case "completed":
			console.log("ready to graduate", event.data.launch);
			break;
		case "graduated":
			console.log("pool", event.data.pool);
			break;
		default:
			break;
	}
}
```

| Event           | Emitted by             | Fields                                                                                                                         |
| --------------- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `configCreated` | `CreateConfig`         | `config`, `authority`, `quoteMint`, `ammConfig`, `migrationSqrtPrice`, `saleSupply`, `migrationSupply`                         |
| `launchCreated` | `CreateLaunch`         | `launch`, `config`, `creator`, `baseMint`, `totalSupply`, `activationTime`                                                     |
| `traded`        | `Buy`, `Sell`          | `launch`, `trader`, `isBuy` (`1` buy, `0` sale), `baseAmount`, `quoteAmount`, `fee`, `creatorFee`, `sqrtPrice`, `quoteReserve` |
| `completed`     | The buy that completes | `launch`, `quoteReserve`                                                                                                       |
| `graduated`     | `Graduate`             | `launch`, `pool`, `poolBase`, `poolQuote`, `migrationFee`, `burnedBase`, `burnedLp`, `creatorLp`, `partnerLp`                  |
| `claimed`       | The three claims       | `launch`, `claimant`, `kind` (`0` partner fees, `1` creator fees, `2` allocation), `amount`                                    |

Always pass every log line of one transaction, in order: the parser only trusts `Program data:` lines written while the curve program is the innermost frame. `decodeTradedEvent` and the other per-event decoders read raw event bytes you already know came from the program.

## Errors

```ts
import {
	getPinaBondingCurveErrorMessage,
	isPinaBondingCurveError,
	PINA_BONDING_CURVE_ERROR__SLIPPAGE_EXCEEDED,
} from "@pina-rs/bonding-curve";
import { unwrapSimulationError } from "@solana/kit";

const instructions = await buyInstructions(quoteAmountIn, minimumBaseOut);
try {
	await send(instructions);
} catch (error) {
	const cause = unwrapSimulationError(error);
	if (
		isPinaBondingCurveError(
			cause,
			{ instructions },
			PINA_BONDING_CURVE_ERROR__SLIPPAGE_EXCEEDED,
		)
	) {
		// The price moved: quote again.
	} else if (isPinaBondingCurveError(cause, { instructions })) {
		console.error(getPinaBondingCurveErrorMessage(cause.context.code));
	} else {
		throw error;
	}
}
```

A rejected preflight wraps the program error, so unwrap it with `unwrapSimulationError` first. The errors a client meets most:

| Code | Constant suffix          | Meaning and next step                                                                   |
| ---- | ------------------------ | --------------------------------------------------------------------------------------- |
| 8    | `INVALID_BASE_MINT`      | The base mint has supply, a freeze authority, another mint authority, or other decimals |
| 11   | `NOT_TRADING`            | The launch has completed or graduated; trade on the AMM                                 |
| 12   | `NOT_ACTIVE`             | Trading opens at `activationTime`                                                       |
| 13   | `NOT_COMPLETED`          | `Graduate` before the launch completed                                                  |
| 14   | `SLIPPAGE_EXCEEDED`      | The price moved past the minimum; quote again                                           |
| 15   | `ZERO_AMOUNT`            | The amount is zero or rounds to nothing                                                 |
| 16   | `INSUFFICIENT_LIQUIDITY` | The curve cannot buy back that much base                                                |
| 20   | `NOTHING_TO_CLAIM`       | No fees accrued or nothing vested yet                                                   |
| 21   | `MISSING_LP_ACCOUNT`     | `Graduate` needs the creator or partner LP accounts                                     |

Every constant is prefixed with `PINA_BONDING_CURVE_ERROR__`. `getPinaBondingCurveErrorMessage` returns developer-facing text, and production bundles strip it; map codes to your own product language.

## After graduation

The launch's `pool` field holds the Pina AMM pool, and `status` is `2`. From then on the token trades on the AMM: use [`@pina-rs/amm`](https://github.com/pina-rs/amm/blob/main/docs/typescript.md) to quote, swap, and add liquidity, and its `CollectCreatorFees` to collect the pool's creator fees, which go to the launch creator or the partner as the configuration's `poolCreatorMode` chose. Fee and allocation claims stay on the curve program.
