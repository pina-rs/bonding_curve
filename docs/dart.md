# Dart and Flutter client

`pina_bonding_curve` is the Dart client for the Pina Bonding Curve, generated from the program's IDL for [`solana_kit`](https://pub.dev/packages/solana_kit). It works in Flutter apps and Dart servers alike.

```yaml
dependencies:
  pina_bonding_curve: ^0.1.0
  solana_kit: ^0.10.0
  solana_kit_rpc_api: ^0.10.0
  # Optional, for token accounts, wrapped SOL, and new mints:
  solana_kit_token: ^0.9.0
  solana_kit_system: ^0.8.0
```

```dart
import 'package:pina_bonding_curve/pina_bonding_curve.dart';
```

## What is exported

| Kind         | Examples                                                                                                |
| ------------ | ------------------------------------------------------------------------------------------------------- |
| Program      | `pinaBondingCurveProgramAddress`, `PinaBondingCurveInstruction`, `parsePinaBondingCurveInstruction`     |
| Instructions | `getBuyInstruction`, `getSellInstruction`, `getCreateLaunchInstruction`, `getGraduateInstruction`, ...  |
| Accounts     | `Launch`, `LaunchConfig`, `decodeLaunch`, `decodeLaunchConfig`, `getLaunchDecoder`, ...                 |
| PDAs         | `findLaunchConfigPda`, `findLaunchPda`, `findLaunchVaultPda`, `findAmmAuthorityPda`                     |
| Events       | `parsePinaBondingCurveEventsFromLogs`, `PinaBondingCurveEvent` subclasses                               |
| Errors       | `pinaBondingCurveErrorSlippageExceeded` and the other code constants, `getPinaBondingCurveErrorMessage` |

Every builder and PDA helper takes `programAddress` explicitly; pass `pinaBondingCurveProgramAddress`. Token amounts, square-root prices, and timestamps are `BigInt`; fee rates and shares are `int` parts per million (`10000` is 1%). Square-root prices are Q64.64: `sqrt(price) * 2^64`, with the price in quote base units per base base unit. `Launch.status` is `0` trading, `1` completed, or `2` graduated. [curves.md](curves.md) explains the price model.

## Find a launch

```dart
final (configAddress, _) = await findLaunchConfigPda(
  seeds: LaunchConfigSeeds(authority: partner, index: BigInt.zero),
  programAddress: pinaBondingCurveProgramAddress,
);
final (launchAddress, _) = await findLaunchPda(
  seeds: LaunchSeeds(baseMint: baseMint),
  programAddress: pinaBondingCurveProgramAddress,
);
final (baseVault, _) = await findLaunchVaultPda(
  seeds: LaunchVaultSeeds(launch: launchAddress, mint: baseMint),
  programAddress: pinaBondingCurveProgramAddress,
);
final (ammAuthority, _) = await findAmmAuthorityPda(
  programAddress: pinaBondingCurveProgramAddress,
);
```

A configuration lives at `["config", authority, index as u64 LE]`, a launch at `["launch", base_mint]`, and each launch vault at `["launch_vault", launch, mint]`. The launch also stores both vault addresses.

## Read a launch

```dart
import 'package:solana_kit/solana_kit.dart';

/// Fetch an account the curve program owns, or throw.
Future<EncodedAccount> fetchCurveAccount(Rpc rpc, Address address) async {
  final account = switch (await fetchEncodedAccount(rpc, address)) {
    ExistingAccount(:final account) => account,
    NonExistingAccount() => throw StateError('no account at $address'),
  };
  if (account.programAddress != pinaBondingCurveProgramAddress) {
    throw StateError('$address is not a Pina Bonding Curve account');
  }
  return account;
}

final rpc = createSolanaRpc(url: 'https://api.mainnet-beta.solana.com');
final launch = decodeLaunch(await fetchCurveAccount(rpc, launchAddress)).data;
final config = decodeLaunchConfig(await fetchCurveAccount(rpc, launch.config))
    .data;
print('raised ${launch.quoteReserve} of ${config.migrationQuoteThreshold}');
```

Generated decoders check the discriminator and schema version but not the owner, so check `programAddress` first, as `fetchCurveAccount` does. For display, `sqrtPrice / BigInt.two.pow(64)` is the square root of the price as a `double`.

## Token accounts

```dart
import 'package:solana_kit_token/solana_kit_token.dart';

/// The token program that owns [mint]: SPL Token or Token-2022.
Future<Address> tokenProgramOf(Rpc rpc, Address mint) async {
  return switch (await fetchEncodedAccount(rpc, mint)) {
    ExistingAccount(:final programAddress) => programAddress,
    NonExistingAccount() => throw StateError('no mint at $mint'),
  };
}

final baseTokenProgram = await tokenProgramOf(rpc, launch.baseMint);
final quoteTokenProgram = await tokenProgramOf(rpc, launch.quoteMint);
final walletBase = getAssociatedTokenAddressSync(
  owner: wallet.address,
  tokenProgram: baseTokenProgram,
  mint: launch.baseMint,
);
final walletQuote = getAssociatedTokenAddressSync(
  owner: wallet.address,
  tokenProgram: quoteTokenProgram,
  mint: launch.quoteMint,
);
```

The token program is part of an associated token account's address, so a Token-2022 mint (`token2022ProgramAddress`) needs its own program in the address, in `getCreateAssociatedTokenIdempotentInstruction`, and as the instruction's `baseTokenProgram` or `quoteTokenProgram`.

## Send and simulate

`wallet` is a `KeyPairSigner` here; in a Flutter app, use your wallet adapter's signer. `simulateTrade` is the authoritative quote: it runs the program against the current state without asking the wallet to sign and returns its `Traded` event.

```dart
import 'package:solana_kit_rpc_api/solana_kit_rpc_api.dart';

/// A transaction message paid for by `wallet`.
Future<TransactionMessage> message(List<Instruction> instructions) async {
  final latest = (await rpc.getLatestBlockhashValue().send()).value;
  var draft = createTransactionMessage(version: TransactionVersion.v0);
  draft = setTransactionMessageFeePayer(wallet.address, draft);
  draft = setTransactionMessageLifetimeUsingBlockhash(
    BlockhashLifetimeConstraint(
      blockhash: latest.blockhash.value,
      lastValidBlockHeight: latest.lastValidBlockHeight,
    ),
    draft,
  );
  return appendTransactionMessageInstructions(instructions, draft);
}

/// Sign with `wallet` and any extra [signers], send, and confirm.
Future<Signature> send(
  List<Instruction> instructions, [
  List<KeyPairSigner> signers = const [],
]) async {
  final transaction = await signTransactionMessageWithSigners(
    addSignersToTransactionMessage([
      wallet,
      ...signers,
    ], await message(instructions)),
  );
  return sendAndConfirmTransaction(rpc: rpc, transaction: transaction);
}

/// Simulate without signing and return the trade the program reports.
Future<TradedEvent> simulateTrade(List<Instruction> instructions) async {
  final transaction = compileTransaction(await message(instructions));
  final response = await rpc
      .simulateTransaction(
        getBase64EncodedWireTransaction(transaction),
        const SimulateTransactionConfig(
          replaceRecentBlockhash: true,
          sigVerify: false,
        ),
      )
      .send();
  final logs = switch (response) {
    {'value': {'err': null, 'logs': final List<Object?> lines}} =>
      lines.whereType<String>().toList(),
    {'value': {'err': final Object error}} =>
      throw getSolanaErrorFromTransactionError(error),
    _ => throw StateError('unexpected simulateTransaction response'),
  };
  final trades = parsePinaBondingCurveEventsFromLogs(logs)
      .whereType<TradedEvent>();
  if (trades.isEmpty) {
    throw StateError('the simulated transaction did not trade');
  }
  return trades.first;
}

/// Lower [amount] by [bps] basis points, rounding down.
BigInt withSlippage(BigInt amount, int bps) =>
    amount * BigInt.from(10000 - bps) ~/ BigInt.from(10000);
```

Simulate with a zero minimum, then send with the minimum derived from the simulated amount. The trading fee only falls over time, so the slippage limit only has to cover other traders moving the price.

## Buy and sell

`quoteAmountIn` is the most quote to spend, fees included; a buy that completes the launch spends only what it needs. When the quote mint is wrapped SOL, fund a wSOL account first and close it afterwards to unwrap what is left:

```dart
import 'package:solana_kit_system/solana_kit_system.dart';

final quoteIsSol = launch.quoteMint == wrappedSolMintAddress;

/// Create [account] for [mint] if it does not exist yet.
Instruction createTokenAccount(
  Address account,
  Address mint,
  Address program,
) => getCreateAssociatedTokenIdempotentInstruction(
  programAddress: associatedTokenProgramAddress,
  payer: wallet.address,
  ata: account,
  owner: wallet.address,
  mint: mint,
  systemProgram: systemProgramAddress,
  tokenProgram: program,
);

final unwrapSol = getCloseAccountInstruction(
  programAddress: tokenProgramAddress,
  account: walletQuote,
  destination: wallet.address,
  owner: wallet.address,
);

List<Instruction> buyInstructions(
  BigInt quoteAmountIn,
  BigInt minimumBaseOut,
) => [
  createTokenAccount(walletBase, launch.baseMint, baseTokenProgram),
  if (quoteIsSol) ...[
    createTokenAccount(walletQuote, launch.quoteMint, quoteTokenProgram),
    getTransferSolInstruction(
      programAddress: systemProgramAddress,
      source: wallet.address,
      destination: walletQuote,
      amount: quoteAmountIn,
    ),
    getSyncNativeInstruction(
      programAddress: tokenProgramAddress,
      account: walletQuote,
    ),
  ],
  getBuyInstruction(
    programAddress: pinaBondingCurveProgramAddress,
    trader: wallet.address,
    config: launch.config,
    launch: launchAddress,
    traderBase: walletBase,
    traderQuote: walletQuote,
    baseVault: launch.baseVault,
    quoteVault: launch.quoteVault,
    baseTokenProgram: baseTokenProgram,
    quoteTokenProgram: quoteTokenProgram,
    quoteAmountIn: quoteAmountIn,
    minimumBaseOut: minimumBaseOut,
  ),
  if (quoteIsSol) unwrapSol,
];

final quoteAmountIn = BigInt.from(1000000000); // 1 SOL
final quoted = await simulateTrade(
  buyInstructions(quoteAmountIn, BigInt.zero),
);
await send(
  buyInstructions(quoteAmountIn, withSlippage(quoted.baseAmount, 100)),
);
```

`getSellInstruction` takes the exact `baseAmountIn` and a `minimumQuoteOut` measured **after** the fee; a sale's `TradedEvent.quoteAmount` is that net amount:

```dart
List<Instruction> sellInstructions(
  BigInt baseAmountIn,
  BigInt minimumQuoteOut,
) => [
  createTokenAccount(walletQuote, launch.quoteMint, quoteTokenProgram),
  getSellInstruction(
    programAddress: pinaBondingCurveProgramAddress,
    trader: wallet.address,
    config: launch.config,
    launch: launchAddress,
    traderBase: walletBase,
    traderQuote: walletQuote,
    baseVault: launch.baseVault,
    quoteVault: launch.quoteVault,
    baseTokenProgram: baseTokenProgram,
    quoteTokenProgram: quoteTokenProgram,
    baseAmountIn: baseAmountIn,
    minimumQuoteOut: minimumQuoteOut,
  ),
  if (quoteIsSol) unwrapSol,
];

final baseAmountIn = BigInt.from(5000000000);
final sale = await simulateTrade(sellInstructions(baseAmountIn, BigInt.zero));
await send(
  sellInstructions(baseAmountIn, withSlippage(sale.quoteAmount, 100)),
);
```

Closing the wSOL account unwraps its whole balance, including wSOL the wallet already held. Trades fail with `pinaBondingCurveErrorNotTrading` once the launch completes and `pinaBondingCurveErrorNotActive` before `activationTime`.

## Create a launch

`CreateLaunch` needs a brand-new base mint with no supply, no freeze authority, the creator as mint authority, and the configuration's `baseDecimals`. It mints the fixed `totalSupply` into the base vault and revokes the mint authority. Create the mint and the launch in one transaction:

```dart
final launchpad = decodeLaunchConfig(
  await fetchCurveAccount(rpc, configAddress),
).data;
final mint = generateKeyPairSigner();
final (newLaunch, _) = await findLaunchPda(
  seeds: LaunchSeeds(baseMint: mint.address),
  programAddress: pinaBondingCurveProgramAddress,
);
final (newBaseVault, _) = await findLaunchVaultPda(
  seeds: LaunchVaultSeeds(launch: newLaunch, mint: mint.address),
  programAddress: pinaBondingCurveProgramAddress,
);
final (newQuoteVault, _) = await findLaunchVaultPda(
  seeds: LaunchVaultSeeds(launch: newLaunch, mint: launchpad.quoteMint),
  programAddress: pinaBondingCurveProgramAddress,
);
final rent = await createClientWithGetMinimumBalanceFromRpc(rpc)
    .getMinimumBalance(mintSize);

await send(
  [
    getCreateAccountInstruction(
      instructionProgramAddress: systemProgramAddress,
      payer: wallet.address,
      newAccount: mint.address,
      lamports: rent,
      space: BigInt.from(mintSize),
      programAddress: tokenProgramAddress,
    ),
    getInitializeMint2Instruction(
      programAddress: tokenProgramAddress,
      mint: mint.address,
      decimals: launchpad.baseDecimals,
      mintAuthority: wallet.address,
    ),
    getCreateLaunchInstruction(
      programAddress: pinaBondingCurveProgramAddress,
      payer: wallet.address,
      creator: wallet.address,
      config: configAddress,
      baseMint: mint.address,
      quoteMint: launchpad.quoteMint,
      launch: newLaunch,
      baseVault: newBaseVault,
      quoteVault: newQuoteVault,
      baseTokenProgram: tokenProgramAddress,
      quoteTokenProgram: await tokenProgramOf(rpc, launchpad.quoteMint),
      systemProgram: systemProgramAddress,
      activationTime: BigInt.zero, // open trading now
    ),
  ],
  [mint],
);
```

A future `activationTime` (Unix seconds) schedules the launch. Base mints may be Token-2022 with only transfer-neutral extensions: metadata pointer and metadata, group pointer, group, and member, interest-bearing, and scaled UI amount. Add metadata before `CreateLaunch`, because it needs the mint authority that `CreateLaunch` revokes.

## Graduate

A buy that reaches the migration price completes the launch (`status == 1`). Anyone can then send `Graduate`, which creates the launch's Pina AMM pool through a CPI. The `pina_amm` package exports `findPoolPda`, `findPoolVaultPda`, and `findPoolLpMintPda`; the same derivation with `solana_kit` alone:

```dart
const pinaAmmProgramAddress = Address(
  'pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV',
);

/// A Pina AMM PDA from a string seed and address seeds.
Future<Address> ammPda(String seed, List<Address> addresses) async {
  final (pda, _) = await getProgramDerivedAddress(
    programAddress: pinaAmmProgramAddress,
    seeds: [
      seed,
      for (final address in addresses) getAddressEncoder().encode(address),
    ],
  );
  return pda;
}

/// Sort two mints by their 32 address bytes, the order the AMM uses.
(Address, Address) sortMints(Address a, Address b) {
  final left = getAddressEncoder().encode(a);
  final right = getAddressEncoder().encode(b);
  for (var index = 0; index < 32; index++) {
    if (left[index] != right[index]) {
      return left[index] < right[index] ? (a, b) : (b, a);
    }
  }
  throw ArgumentError('a pool needs two different mints');
}

/// An LP token account. LP mints are always SPL Token.
Address lpTokenOf(Address owner, Address lpMint) =>
    getAssociatedTokenAddressSync(
      owner: owner,
      tokenProgram: tokenProgramAddress,
      mint: lpMint,
    );

final (mint0, mint1) = sortMints(launch.baseMint, launch.quoteMint);
final pool = await ammPda('pool', [config.ammConfig, mint0, mint1]);
final lpMint = await ammPda('pool_lp_mint', [pool]);
final payCreator = config.creatorLpShare > 0;
final payPartner = config.partnerLpShare > 0;

await send([
  getGraduateInstruction(
    programAddress: pinaBondingCurveProgramAddress,
    payer: wallet.address,
    config: launch.config,
    launch: launchAddress,
    baseMint: launch.baseMint,
    quoteMint: launch.quoteMint,
    baseVault: launch.baseVault,
    quoteVault: launch.quoteVault,
    ammAuthority: ammAuthority,
    ammProgram: pinaAmmProgramAddress,
    ammConfig: config.ammConfig,
    pool: pool,
    lpMint: lpMint,
    poolVault0: await ammPda('pool_vault', [pool, mint0]),
    poolVault1: await ammPda('pool_vault', [pool, mint1]),
    launchLpToken: lpTokenOf(launchAddress, lpMint),
    baseTokenProgram: baseTokenProgram,
    quoteTokenProgram: quoteTokenProgram,
    lpTokenProgram: tokenProgramAddress,
    associatedTokenProgram: associatedTokenProgramAddress,
    systemProgram: systemProgramAddress,
    creator: payCreator ? launch.creator : null,
    creatorLpToken: payCreator ? lpTokenOf(launch.creator, lpMint) : null,
    partner: payPartner ? config.authority : null,
    partnerLpToken: payPartner ? lpTokenOf(config.authority, lpMint) : null,
  ),
]);
```

The creator and partner LP accounts are required only when `creatorLpShare` or `partnerLpShare` is above zero; a null fills the slot with the program address, which the program reads as absent. [graduation.md](graduation.md) explains where every token goes.

## Claim fees and the creator allocation

Fees accrue in the launch's quote vault until claimed, before and after graduation. Each claim pays everything owed to any token account of the right mint and fails with `pinaBondingCurveErrorNothingToClaim` when nothing is:

```dart
// Signed by the configuration's authority (the partner).
if (launch.partnerFees > BigInt.zero) {
  await send([
    createTokenAccount(walletQuote, launch.quoteMint, quoteTokenProgram),
    getClaimPartnerFeesInstruction(
      programAddress: pinaBondingCurveProgramAddress,
      claimant: wallet.address,
      config: launch.config,
      launch: launchAddress,
      quoteVault: launch.quoteVault,
      destination: walletQuote,
      quoteTokenProgram: quoteTokenProgram,
    ),
  ]);
}
```

`getClaimCreatorFeesInstruction` takes the same accounts, signed by the launch's creator, and pays `launch.creatorFees`. The creator allocation vests linearly over `creatorVestingDuration` seconds, starting `creatorVestingCliff` seconds after `activationTime`. This mirrors the program:

```dart
/// Base the creator can claim at [now] in Unix seconds.
BigInt claimableAllocation(BigInt now) {
  final start = launch.activationTime + config.creatorVestingCliff;
  if (now < start) {
    return BigInt.zero;
  }
  final duration = config.creatorVestingDuration;
  final elapsed = now - start < duration ? now - start : duration;
  final vested = duration == BigInt.zero
      ? config.creatorAllocation
      : config.creatorAllocation * elapsed ~/ duration;
  return vested - launch.creatorClaimed;
}

final now = BigInt.from(DateTime.now().millisecondsSinceEpoch ~/ 1000);
if (claimableAllocation(now) > BigInt.zero) {
  // Signed by the launch's creator.
  await send([
    createTokenAccount(walletBase, launch.baseMint, baseTokenProgram),
    getClaimCreatorAllocationInstruction(
      programAddress: pinaBondingCurveProgramAddress,
      creator: wallet.address,
      config: launch.config,
      launch: launchAddress,
      baseVault: launch.baseVault,
      destination: walletBase,
      baseTokenProgram: baseTokenProgram,
    ),
  ]);
}
```

`getSetLaunchCreatorInstruction` hands everything the creator receives from then on (unclaimed and future fees, the rest of the allocation, and graduation LP) to another address, such as a multisig.

## Events

```dart
final response = await rpc
    .getTransaction(
      signature,
      const GetTransactionConfig(maxSupportedTransactionVersion: 0),
    )
    .send();
final logs = switch (response) {
  {'meta': {'logMessages': final List<Object?> lines}} =>
    lines.whereType<String>().toList(),
  _ => const <String>[],
};
for (final event in parsePinaBondingCurveEventsFromLogs(logs)) {
  switch (event) {
    case TradedEvent(:final isBuy, :final baseAmount, :final quoteAmount):
      print('${isBuy == 1 ? 'buy' : 'sell'} $baseAmount for $quoteAmount');
    case CompletedEvent(:final launch):
      print('$launch is ready to graduate');
    case GraduatedEvent(:final pool):
      print('trading moved to $pool');
    default:
      break;
  }
}
```

The events are `ConfigCreatedEvent`, `LaunchCreatedEvent`, `TradedEvent`, `CompletedEvent`, `GraduatedEvent`, and `ClaimedEvent` (`kind` `0` partner fees, `1` creator fees, `2` allocation). Pass every log line of one transaction, in order, so records are attributed only to the curve program.

## Errors

```dart
/// Whether [error] is a curve program error, optionally with [code].
bool isCurveError(
  Object? error,
  List<Instruction> instructions, [
  int? code,
]) => isProgramError(
  unwrapSimulationError(error),
  TransactionMessageInput(
    instructions: {
      for (final (index, instruction) in instructions.indexed)
        index: InstructionInput(programAddress: instruction.programAddress),
    },
  ),
  pinaBondingCurveProgramAddress,
  code,
);

final instructions = buyInstructions(quoteAmountIn, minimumBaseOut);
try {
  await send(instructions);
} on SolanaError catch (error) {
  if (!isCurveError(
    error,
    instructions,
    pinaBondingCurveErrorSlippageExceeded,
  )) {
    rethrow;
  }
  // The price moved: quote again.
}
```

A rejected preflight wraps the program error, which `unwrapSimulationError` takes out. `getPinaBondingCurveErrorMessage(code)` returns the developer message for any code.

## Flutter notes

- Keep `BigInt` end to end; token amounts exceed JavaScript-safe and Dart `int` ranges on the web.
- Show users the quote _and_ the minimum they will accept, both from `simulateTrade` and `withSlippage`.
- `launch.status`, `quoteReserve`, and `config.migrationQuoteThreshold` drive a progress bar; refresh them after each trade or watch the launch account.
- Map error codes such as `pinaBondingCurveErrorSlippageExceeded` and `pinaBondingCurveErrorNotActive` to product language; the generated messages are written for developers.

## After graduation

The launch's `pool` field holds the Pina AMM pool. Trade, quote, and provide liquidity with [`pina_amm`](https://github.com/pina-rs/amm/blob/main/docs/dart.md); fee and allocation claims stay on the curve program.
