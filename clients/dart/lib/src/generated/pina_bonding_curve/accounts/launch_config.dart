// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:meta/meta.dart';
import 'package:solana_kit_accounts/solana_kit_accounts.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_core/solana_kit_codecs_core.dart';
import 'package:solana_kit_codecs_data_structures/solana_kit_codecs_data_structures.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';
import 'package:solana_kit_errors/solana_kit_errors.dart';

@immutable
class LaunchConfig {
  const LaunchConfig({
    required this.authority,
    required this.quoteMint,
    required this.ammConfig,
    required this.sqrtStartPrice,
    required this.migrationSqrtPrice,
    required this.curveSqrtPrices,
    required this.curveLiquidities,
    required this.totalSupply,
    required this.saleSupply,
    required this.migrationSupply,
    required this.creatorAllocation,
    required this.migrationQuoteThreshold,
    required this.creatorVestingCliff,
    required this.creatorVestingDuration,
    required this.feeDecayDuration,
    required this.index,
    required this.startFeeRate,
    required this.endFeeRate,
    required this.creatorFeeShare,
    required this.migrationFeeRate,
    required this.creatorLpShare,
    required this.partnerLpShare,
    required this.baseDecimals,
    required this.segmentCount,
    required this.poolCreatorMode,
    required this.bump,
  }) : discriminator = 1,
       migrationVersion = 0;

  final int discriminator;
  final int migrationVersion;
  final Address authority;
  final Address quoteMint;
  final Address ammConfig;
  final BigInt sqrtStartPrice;
  final BigInt migrationSqrtPrice;
  final List<BigInt> curveSqrtPrices;
  final List<BigInt> curveLiquidities;
  final BigInt totalSupply;
  final BigInt saleSupply;
  final BigInt migrationSupply;
  final BigInt creatorAllocation;
  final BigInt migrationQuoteThreshold;
  final BigInt creatorVestingCliff;
  final BigInt creatorVestingDuration;
  final BigInt feeDecayDuration;
  final BigInt index;
  final int startFeeRate;
  final int endFeeRate;
  final int creatorFeeShare;
  final int migrationFeeRate;
  final int creatorLpShare;
  final int partnerLpShare;
  final int baseDecimals;
  final int segmentCount;
  final int poolCreatorMode;
  final int bump;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is LaunchConfig &&
          runtimeType == other.runtimeType &&
          discriminator == other.discriminator &&
          migrationVersion == other.migrationVersion &&
          authority == other.authority &&
          quoteMint == other.quoteMint &&
          ammConfig == other.ammConfig &&
          sqrtStartPrice == other.sqrtStartPrice &&
          migrationSqrtPrice == other.migrationSqrtPrice &&
          curveSqrtPrices == other.curveSqrtPrices &&
          curveLiquidities == other.curveLiquidities &&
          totalSupply == other.totalSupply &&
          saleSupply == other.saleSupply &&
          migrationSupply == other.migrationSupply &&
          creatorAllocation == other.creatorAllocation &&
          migrationQuoteThreshold == other.migrationQuoteThreshold &&
          creatorVestingCliff == other.creatorVestingCliff &&
          creatorVestingDuration == other.creatorVestingDuration &&
          feeDecayDuration == other.feeDecayDuration &&
          index == other.index &&
          startFeeRate == other.startFeeRate &&
          endFeeRate == other.endFeeRate &&
          creatorFeeShare == other.creatorFeeShare &&
          migrationFeeRate == other.migrationFeeRate &&
          creatorLpShare == other.creatorLpShare &&
          partnerLpShare == other.partnerLpShare &&
          baseDecimals == other.baseDecimals &&
          segmentCount == other.segmentCount &&
          poolCreatorMode == other.poolCreatorMode &&
          bump == other.bump;

  @override
  int get hashCode => Object.hashAll([
    discriminator,
    migrationVersion,
    authority,
    quoteMint,
    ammConfig,
    sqrtStartPrice,
    migrationSqrtPrice,
    curveSqrtPrices,
    curveLiquidities,
    totalSupply,
    saleSupply,
    migrationSupply,
    creatorAllocation,
    migrationQuoteThreshold,
    creatorVestingCliff,
    creatorVestingDuration,
    feeDecayDuration,
    index,
    startFeeRate,
    endFeeRate,
    creatorFeeShare,
    migrationFeeRate,
    creatorLpShare,
    partnerLpShare,
    baseDecimals,
    segmentCount,
    poolCreatorMode,
    bump,
  ]);

  @override
  String toString() =>
      'LaunchConfig(discriminator: $discriminator, migrationVersion: $migrationVersion, authority: $authority, quoteMint: $quoteMint, ammConfig: $ammConfig, sqrtStartPrice: $sqrtStartPrice, migrationSqrtPrice: $migrationSqrtPrice, curveSqrtPrices: $curveSqrtPrices, curveLiquidities: $curveLiquidities, totalSupply: $totalSupply, saleSupply: $saleSupply, migrationSupply: $migrationSupply, creatorAllocation: $creatorAllocation, migrationQuoteThreshold: $migrationQuoteThreshold, creatorVestingCliff: $creatorVestingCliff, creatorVestingDuration: $creatorVestingDuration, feeDecayDuration: $feeDecayDuration, index: $index, startFeeRate: $startFeeRate, endFeeRate: $endFeeRate, creatorFeeShare: $creatorFeeShare, migrationFeeRate: $migrationFeeRate, creatorLpShare: $creatorLpShare, partnerLpShare: $partnerLpShare, baseDecimals: $baseDecimals, segmentCount: $segmentCount, poolCreatorMode: $poolCreatorMode, bump: $bump)';
}

Encoder<LaunchConfig> getLaunchConfigEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('migrationVersion', getU8Encoder()),
    ('authority', getAddressEncoder()),
    ('quoteMint', getAddressEncoder()),
    ('ammConfig', getAddressEncoder()),
    ('sqrtStartPrice', getU128Encoder()),
    ('migrationSqrtPrice', getU128Encoder()),
    (
      'curveSqrtPrices',
      getArrayEncoder(
        transformEncoder(getU128Encoder(), (BigInt value) => value),
        size: FixedArraySize(16),
      ),
    ),
    (
      'curveLiquidities',
      getArrayEncoder(
        transformEncoder(getU128Encoder(), (BigInt value) => value),
        size: FixedArraySize(16),
      ),
    ),
    ('totalSupply', getU64Encoder()),
    ('saleSupply', getU64Encoder()),
    ('migrationSupply', getU64Encoder()),
    ('creatorAllocation', getU64Encoder()),
    ('migrationQuoteThreshold', getU64Encoder()),
    ('creatorVestingCliff', getU64Encoder()),
    ('creatorVestingDuration', getU64Encoder()),
    ('feeDecayDuration', getU64Encoder()),
    ('index', getU64Encoder()),
    ('startFeeRate', getU32Encoder()),
    ('endFeeRate', getU32Encoder()),
    ('creatorFeeShare', getU32Encoder()),
    ('migrationFeeRate', getU32Encoder()),
    ('creatorLpShare', getU32Encoder()),
    ('partnerLpShare', getU32Encoder()),
    ('baseDecimals', getU8Encoder()),
    ('segmentCount', getU8Encoder()),
    ('poolCreatorMode', getU8Encoder()),
    ('bump', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (LaunchConfig value) => <String, Object?>{
      'discriminator': 1,
      'migrationVersion': 0,
      'authority': value.authority,
      'quoteMint': value.quoteMint,
      'ammConfig': value.ammConfig,
      'sqrtStartPrice': value.sqrtStartPrice,
      'migrationSqrtPrice': value.migrationSqrtPrice,
      'curveSqrtPrices': value.curveSqrtPrices,
      'curveLiquidities': value.curveLiquidities,
      'totalSupply': value.totalSupply,
      'saleSupply': value.saleSupply,
      'migrationSupply': value.migrationSupply,
      'creatorAllocation': value.creatorAllocation,
      'migrationQuoteThreshold': value.migrationQuoteThreshold,
      'creatorVestingCliff': value.creatorVestingCliff,
      'creatorVestingDuration': value.creatorVestingDuration,
      'feeDecayDuration': value.feeDecayDuration,
      'index': value.index,
      'startFeeRate': value.startFeeRate,
      'endFeeRate': value.endFeeRate,
      'creatorFeeShare': value.creatorFeeShare,
      'migrationFeeRate': value.migrationFeeRate,
      'creatorLpShare': value.creatorLpShare,
      'partnerLpShare': value.partnerLpShare,
      'baseDecimals': value.baseDecimals,
      'segmentCount': value.segmentCount,
      'poolCreatorMode': value.poolCreatorMode,
      'bump': value.bump,
    },
  );
}

Decoder<LaunchConfig> getLaunchConfigDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('migrationVersion', getU8Decoder()),
    ('authority', getAddressDecoder()),
    ('quoteMint', getAddressDecoder()),
    ('ammConfig', getAddressDecoder()),
    ('sqrtStartPrice', getU128Decoder()),
    ('migrationSqrtPrice', getU128Decoder()),
    (
      'curveSqrtPrices',
      getArrayDecoder(getU128Decoder(), size: FixedArraySize(16)),
    ),
    (
      'curveLiquidities',
      getArrayDecoder(getU128Decoder(), size: FixedArraySize(16)),
    ),
    ('totalSupply', getU64Decoder()),
    ('saleSupply', getU64Decoder()),
    ('migrationSupply', getU64Decoder()),
    ('creatorAllocation', getU64Decoder()),
    ('migrationQuoteThreshold', getU64Decoder()),
    ('creatorVestingCliff', getU64Decoder()),
    ('creatorVestingDuration', getU64Decoder()),
    ('feeDecayDuration', getU64Decoder()),
    ('index', getU64Decoder()),
    ('startFeeRate', getU32Decoder()),
    ('endFeeRate', getU32Decoder()),
    ('creatorFeeShare', getU32Decoder()),
    ('migrationFeeRate', getU32Decoder()),
    ('creatorLpShare', getU32Decoder()),
    ('partnerLpShare', getU32Decoder()),
    ('baseDecimals', getU8Decoder()),
    ('segmentCount', getU8Decoder()),
    ('poolCreatorMode', getU8Decoder()),
    ('bump', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'launchConfig account decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (LaunchConfig, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(1)).read(bytes, offset + 0);
    final (storedMigrationVersion, _) = getU8Decoder().read(bytes, offset + 1);
    if (storedMigrationVersion != 0) {
      throw StateError(
        storedMigrationVersion < 0
            ? 'migration version mismatch: expected 0, received $storedMigrationVersion (the data predates this client; migrate it by sending a transaction to the program, or decode it with a client generated from an older IDL)'
            : 'migration version mismatch: expected 0, received $storedMigrationVersion (the data was written by a newer program; upgrade this client)',
      );
    }
    final (map, newOffset) = structDecoder.read(bytes, offset);

    return (
      LaunchConfig(
        authority: map['authority']! as Address,
        quoteMint: map['quoteMint']! as Address,
        ammConfig: map['ammConfig']! as Address,
        sqrtStartPrice: map['sqrtStartPrice']! as BigInt,
        migrationSqrtPrice: map['migrationSqrtPrice']! as BigInt,
        curveSqrtPrices: map['curveSqrtPrices']! as List<BigInt>,
        curveLiquidities: map['curveLiquidities']! as List<BigInt>,
        totalSupply: map['totalSupply']! as BigInt,
        saleSupply: map['saleSupply']! as BigInt,
        migrationSupply: map['migrationSupply']! as BigInt,
        creatorAllocation: map['creatorAllocation']! as BigInt,
        migrationQuoteThreshold: map['migrationQuoteThreshold']! as BigInt,
        creatorVestingCliff: map['creatorVestingCliff']! as BigInt,
        creatorVestingDuration: map['creatorVestingDuration']! as BigInt,
        feeDecayDuration: map['feeDecayDuration']! as BigInt,
        index: map['index']! as BigInt,
        startFeeRate: map['startFeeRate']! as int,
        endFeeRate: map['endFeeRate']! as int,
        creatorFeeShare: map['creatorFeeShare']! as int,
        migrationFeeRate: map['migrationFeeRate']! as int,
        creatorLpShare: map['creatorLpShare']! as int,
        partnerLpShare: map['partnerLpShare']! as int,
        baseDecimals: map['baseDecimals']! as int,
        segmentCount: map['segmentCount']! as int,
        poolCreatorMode: map['poolCreatorMode']! as int,
        bump: map['bump']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() => FixedSizeDecoder<LaunchConfig>(
      fixedSize: structDecoder.fixedSize,
      read: (bytes, offset) {
        final bytesLength = bytes.length - offset;
        if (bytesLength < structDecoder.fixedSize) {
          throwInvalidByteLength(structDecoder.fixedSize, bytesLength);
        }
        return readTopLevel(bytes, offset);
      },
    ),
    VariableSizeDecoder<Map<String, Object?>>() =>
      VariableSizeDecoder<LaunchConfig>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<LaunchConfig, LaunchConfig> getLaunchConfigCodec() {
  return combineCodec(getLaunchConfigEncoder(), getLaunchConfigDecoder());
}

Account<LaunchConfig> decodeLaunchConfig(EncodedAccount encodedAccount) {
  return decodeAccount(encodedAccount, getLaunchConfigDecoder());
}

/// The account schema version this client was generated from.
const int launchConfigMigrationVersion = 0;

/// Cheap envelope check for fetched `LaunchConfig` bytes: returns true only when
/// the bytes carry this account's discriminator and a migration version older
/// than this client's schema — exactly the accounts [getMigrateInstruction]
/// can bring current. Decoding reports every other mismatch.
bool launchConfigNeedsMigration(List<int> data) {
  if (data.length < 2) {
    return false;
  }
  if (data[0] != 1) {
    return false;
  }
  return data[1] < 0;
}
