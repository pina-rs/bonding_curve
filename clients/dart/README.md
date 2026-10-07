# pina_bonding_curve

Dart and Flutter client for the [Pina Bonding Curve](https://github.com/pina-rs/bonding_curve), a Solana launchpad that sells a new token along a configurable price curve and then graduates it into the [Pina AMM](https://github.com/pina-rs/amm), for [`solana_kit`](https://pub.dev/packages/solana_kit). Generated from the program's IDL.

```yaml
dependencies:
  pina_bonding_curve: ^0.1.0
```

```dart
import 'package:pina_bonding_curve/pina_bonding_curve.dart';
import 'package:solana_kit/solana_kit.dart';

final instruction = getBuyInstruction(
  programAddress: pinaBondingCurveProgramAddress,
  trader: wallet.address,
  config: launch.config,
  launch: launchAddress,
  traderBase: walletBase,
  traderQuote: walletQuote,
  baseVault: launch.baseVault,
  quoteVault: launch.quoteVault,
  baseTokenProgram: tokenProgramAddress,
  quoteTokenProgram: tokenProgramAddress,
  quoteAmountIn: BigInt.from(1000000000),
  minimumBaseOut: BigInt.from(990000000),
);
```

Full guide: [docs/dart.md](https://github.com/pina-rs/bonding_curve/blob/main/docs/dart.md).
