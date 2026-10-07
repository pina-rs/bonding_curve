// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'package:meta/meta.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';

@immutable
class LaunchSeeds {
  const LaunchSeeds({required this.baseMint});

  final Address baseMint;
}

/// Finds the program derived address for [Launch].
Future<(Address, int)> findLaunchPda({
  required LaunchSeeds seeds,
  required Address programAddress,
}) async {
  final seedValues = <Object>[
    'launch',
    getAddressEncoder().encode(seeds.baseMint),
  ];

  return getProgramDerivedAddress(
    programAddress: programAddress,
    seeds: seedValues,
  );
}
