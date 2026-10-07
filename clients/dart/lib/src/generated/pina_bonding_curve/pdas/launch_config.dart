// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'package:meta/meta.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

@immutable
class LaunchConfigSeeds {
  const LaunchConfigSeeds({required this.authority, required this.index});

  final Address authority;
  final BigInt index;
}

/// Finds the program derived address for [LaunchConfig].
Future<(Address, int)> findLaunchConfigPda({
  required LaunchConfigSeeds seeds,
  required Address programAddress,
}) async {
  final seedValues = <Object>[
    'config',
    getAddressEncoder().encode(seeds.authority),
    getU64Encoder().encode(seeds.index),
  ];

  return getProgramDerivedAddress(
    programAddress: programAddress,
    seeds: seedValues,
  );
}
