// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'package:meta/meta.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';

@immutable
class LaunchVaultSeeds {
  const LaunchVaultSeeds({required this.launch, required this.mint});

  final Address launch;
  final Address mint;
}

/// Finds the program derived address for [LaunchVault].
Future<(Address, int)> findLaunchVaultPda({
  required LaunchVaultSeeds seeds,
  required Address programAddress,
}) async {
  final seedValues = <Object>[
    'launch_vault',
    getAddressEncoder().encode(seeds.launch),
    getAddressEncoder().encode(seeds.mint),
  ];

  return getProgramDerivedAddress(
    programAddress: programAddress,
    seeds: seedValues,
  );
}
