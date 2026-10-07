import 'dart:typed_data';

import 'package:pina_bonding_curve/pina_bonding_curve.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_instructions/solana_kit_instructions.dart';
import 'package:test/test.dart';

void main() {
  test('exports the deployed program address', () {
    expect(
      pinaBondingCurveProgramAddress,
      const Address('CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9'),
    );
  });

  test('builds a buy with the program account order and data layout', () {
    const placeholder = Address('11111111111111111111111111111111');
    final instruction = getBuyInstruction(
      programAddress: pinaBondingCurveProgramAddress,
      trader: placeholder,
      config: placeholder,
      launch: placeholder,
      traderBase: placeholder,
      traderQuote: placeholder,
      baseVault: placeholder,
      quoteVault: placeholder,
      baseTokenProgram: placeholder,
      quoteTokenProgram: placeholder,
      quoteAmountIn: BigInt.from(1000000000),
      minimumBaseOut: BigInt.from(42),
    );

    expect(instruction.accounts, hasLength(9));
    expect(instruction.accounts!.first.role, AccountRole.readonlySigner);
    final data = instruction.data!;
    expect(data, hasLength(17));
    expect(data[0], 2);
    final view = ByteData.sublistView(Uint8List.fromList(data));
    expect(view.getUint64(1, Endian.little), 1000000000);
    expect(view.getUint64(9, Endian.little), 42);
  });
}
