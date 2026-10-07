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
class Launch {
  const Launch({
    required this.config,
    required this.creator,
    required this.baseMint,
    required this.quoteMint,
    required this.baseVault,
    required this.quoteVault,
    required this.pool,
    required this.sqrtPrice,
    required this.quoteReserve,
    required this.partnerFees,
    required this.creatorFees,
    required this.creatorClaimed,
    required this.activationTime,
    required this.status,
    required this.bump,
  }) : discriminator = 2,
       migrationVersion = 0;

  final int discriminator;
  final int migrationVersion;
  final Address config;
  final Address creator;
  final Address baseMint;
  final Address quoteMint;
  final Address baseVault;
  final Address quoteVault;
  final Address pool;
  final BigInt sqrtPrice;
  final BigInt quoteReserve;
  final BigInt partnerFees;
  final BigInt creatorFees;
  final BigInt creatorClaimed;
  final BigInt activationTime;
  final int status;
  final int bump;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Launch &&
          runtimeType == other.runtimeType &&
          discriminator == other.discriminator &&
          migrationVersion == other.migrationVersion &&
          config == other.config &&
          creator == other.creator &&
          baseMint == other.baseMint &&
          quoteMint == other.quoteMint &&
          baseVault == other.baseVault &&
          quoteVault == other.quoteVault &&
          pool == other.pool &&
          sqrtPrice == other.sqrtPrice &&
          quoteReserve == other.quoteReserve &&
          partnerFees == other.partnerFees &&
          creatorFees == other.creatorFees &&
          creatorClaimed == other.creatorClaimed &&
          activationTime == other.activationTime &&
          status == other.status &&
          bump == other.bump;

  @override
  int get hashCode => Object.hash(
    discriminator,
    migrationVersion,
    config,
    creator,
    baseMint,
    quoteMint,
    baseVault,
    quoteVault,
    pool,
    sqrtPrice,
    quoteReserve,
    partnerFees,
    creatorFees,
    creatorClaimed,
    activationTime,
    status,
    bump,
  );

  @override
  String toString() =>
      'Launch(discriminator: $discriminator, migrationVersion: $migrationVersion, config: $config, creator: $creator, baseMint: $baseMint, quoteMint: $quoteMint, baseVault: $baseVault, quoteVault: $quoteVault, pool: $pool, sqrtPrice: $sqrtPrice, quoteReserve: $quoteReserve, partnerFees: $partnerFees, creatorFees: $creatorFees, creatorClaimed: $creatorClaimed, activationTime: $activationTime, status: $status, bump: $bump)';
}

Encoder<Launch> getLaunchEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('migrationVersion', getU8Encoder()),
    ('config', getAddressEncoder()),
    ('creator', getAddressEncoder()),
    ('baseMint', getAddressEncoder()),
    ('quoteMint', getAddressEncoder()),
    ('baseVault', getAddressEncoder()),
    ('quoteVault', getAddressEncoder()),
    ('pool', getAddressEncoder()),
    ('sqrtPrice', getU128Encoder()),
    ('quoteReserve', getU64Encoder()),
    ('partnerFees', getU64Encoder()),
    ('creatorFees', getU64Encoder()),
    ('creatorClaimed', getU64Encoder()),
    ('activationTime', getI64Encoder()),
    ('status', getU8Encoder()),
    ('bump', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (Launch value) => <String, Object?>{
      'discriminator': 2,
      'migrationVersion': 0,
      'config': value.config,
      'creator': value.creator,
      'baseMint': value.baseMint,
      'quoteMint': value.quoteMint,
      'baseVault': value.baseVault,
      'quoteVault': value.quoteVault,
      'pool': value.pool,
      'sqrtPrice': value.sqrtPrice,
      'quoteReserve': value.quoteReserve,
      'partnerFees': value.partnerFees,
      'creatorFees': value.creatorFees,
      'creatorClaimed': value.creatorClaimed,
      'activationTime': value.activationTime,
      'status': value.status,
      'bump': value.bump,
    },
  );
}

Decoder<Launch> getLaunchDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('migrationVersion', getU8Decoder()),
    ('config', getAddressDecoder()),
    ('creator', getAddressDecoder()),
    ('baseMint', getAddressDecoder()),
    ('quoteMint', getAddressDecoder()),
    ('baseVault', getAddressDecoder()),
    ('quoteVault', getAddressDecoder()),
    ('pool', getAddressDecoder()),
    ('sqrtPrice', getU128Decoder()),
    ('quoteReserve', getU64Decoder()),
    ('partnerFees', getU64Decoder()),
    ('creatorFees', getU64Decoder()),
    ('creatorClaimed', getU64Decoder()),
    ('activationTime', getI64Decoder()),
    ('status', getU8Decoder()),
    ('bump', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'launch account decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (Launch, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(2)).read(bytes, offset + 0);
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
      Launch(
        config: map['config']! as Address,
        creator: map['creator']! as Address,
        baseMint: map['baseMint']! as Address,
        quoteMint: map['quoteMint']! as Address,
        baseVault: map['baseVault']! as Address,
        quoteVault: map['quoteVault']! as Address,
        pool: map['pool']! as Address,
        sqrtPrice: map['sqrtPrice']! as BigInt,
        quoteReserve: map['quoteReserve']! as BigInt,
        partnerFees: map['partnerFees']! as BigInt,
        creatorFees: map['creatorFees']! as BigInt,
        creatorClaimed: map['creatorClaimed']! as BigInt,
        activationTime: map['activationTime']! as BigInt,
        status: map['status']! as int,
        bump: map['bump']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() => FixedSizeDecoder<Launch>(
      fixedSize: structDecoder.fixedSize,
      read: (bytes, offset) {
        final bytesLength = bytes.length - offset;
        if (bytesLength < structDecoder.fixedSize) {
          throwInvalidByteLength(structDecoder.fixedSize, bytesLength);
        }
        return readTopLevel(bytes, offset);
      },
    ),
    VariableSizeDecoder<Map<String, Object?>>() => VariableSizeDecoder<Launch>(
      read: readTopLevel,
      maxSize: structDecoder.maxSize,
    ),
  };
}

Codec<Launch, Launch> getLaunchCodec() {
  return combineCodec(getLaunchEncoder(), getLaunchDecoder());
}

Account<Launch> decodeLaunch(EncodedAccount encodedAccount) {
  return decodeAccount(encodedAccount, getLaunchDecoder());
}

/// The account schema version this client was generated from.
const int launchMigrationVersion = 0;

/// Cheap envelope check for fetched `Launch` bytes: returns true only when
/// the bytes carry this account's discriminator and a migration version older
/// than this client's schema — exactly the accounts [getMigrateInstruction]
/// can bring current. Decoding reports every other mismatch.
bool launchNeedsMigration(List<int> data) {
  if (data.length < 2) {
    return false;
  }
  if (data[0] != 2) {
    return false;
  }
  return data[1] < 0;
}
