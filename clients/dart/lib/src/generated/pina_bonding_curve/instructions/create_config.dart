// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:meta/meta.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_core/solana_kit_codecs_core.dart';
import 'package:solana_kit_codecs_data_structures/solana_kit_codecs_data_structures.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';
import 'package:solana_kit_errors/solana_kit_errors.dart';
import 'package:solana_kit_instructions/solana_kit_instructions.dart';

@immutable
class CreateConfigInstructionData {
  const CreateConfigInstructionData({
    required this.index,
    required this.sqrtStartPrice,
    required this.curveSqrtPrices,
    required this.curveLiquidities,
    required this.migrationQuoteThreshold,
    required this.totalSupply,
    required this.creatorAllocation,
    required this.creatorVestingCliff,
    required this.creatorVestingDuration,
    required this.feeDecayDuration,
    required this.startFeeRate,
    required this.endFeeRate,
    required this.creatorFeeShare,
    required this.migrationFeeRate,
    required this.creatorLpShare,
    required this.partnerLpShare,
    required this.segmentCount,
    required this.baseDecimals,
    required this.poolCreatorMode,
  }) : discriminator = 0;

  final int discriminator;
  final BigInt index;
  final BigInt sqrtStartPrice;
  final List<BigInt> curveSqrtPrices;
  final List<BigInt> curveLiquidities;
  final BigInt migrationQuoteThreshold;
  final BigInt totalSupply;
  final BigInt creatorAllocation;
  final BigInt creatorVestingCliff;
  final BigInt creatorVestingDuration;
  final BigInt feeDecayDuration;
  final int startFeeRate;
  final int endFeeRate;
  final int creatorFeeShare;
  final int migrationFeeRate;
  final int creatorLpShare;
  final int partnerLpShare;
  final int segmentCount;
  final int baseDecimals;
  final int poolCreatorMode;
}

Encoder<CreateConfigInstructionData> getCreateConfigInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('index', getU64Encoder()),
    ('sqrtStartPrice', getU128Encoder()),
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
    ('migrationQuoteThreshold', getU64Encoder()),
    ('totalSupply', getU64Encoder()),
    ('creatorAllocation', getU64Encoder()),
    ('creatorVestingCliff', getU64Encoder()),
    ('creatorVestingDuration', getU64Encoder()),
    ('feeDecayDuration', getU64Encoder()),
    ('startFeeRate', getU32Encoder()),
    ('endFeeRate', getU32Encoder()),
    ('creatorFeeShare', getU32Encoder()),
    ('migrationFeeRate', getU32Encoder()),
    ('creatorLpShare', getU32Encoder()),
    ('partnerLpShare', getU32Encoder()),
    ('segmentCount', getU8Encoder()),
    ('baseDecimals', getU8Encoder()),
    ('poolCreatorMode', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (CreateConfigInstructionData value) => <String, Object?>{
      'discriminator': 0,
      'index': value.index,
      'sqrtStartPrice': value.sqrtStartPrice,
      'curveSqrtPrices': value.curveSqrtPrices,
      'curveLiquidities': value.curveLiquidities,
      'migrationQuoteThreshold': value.migrationQuoteThreshold,
      'totalSupply': value.totalSupply,
      'creatorAllocation': value.creatorAllocation,
      'creatorVestingCliff': value.creatorVestingCliff,
      'creatorVestingDuration': value.creatorVestingDuration,
      'feeDecayDuration': value.feeDecayDuration,
      'startFeeRate': value.startFeeRate,
      'endFeeRate': value.endFeeRate,
      'creatorFeeShare': value.creatorFeeShare,
      'migrationFeeRate': value.migrationFeeRate,
      'creatorLpShare': value.creatorLpShare,
      'partnerLpShare': value.partnerLpShare,
      'segmentCount': value.segmentCount,
      'baseDecimals': value.baseDecimals,
      'poolCreatorMode': value.poolCreatorMode,
    },
  );
}

Decoder<CreateConfigInstructionData> getCreateConfigInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('index', getU64Decoder()),
    ('sqrtStartPrice', getU128Decoder()),
    (
      'curveSqrtPrices',
      getArrayDecoder(getU128Decoder(), size: FixedArraySize(16)),
    ),
    (
      'curveLiquidities',
      getArrayDecoder(getU128Decoder(), size: FixedArraySize(16)),
    ),
    ('migrationQuoteThreshold', getU64Decoder()),
    ('totalSupply', getU64Decoder()),
    ('creatorAllocation', getU64Decoder()),
    ('creatorVestingCliff', getU64Decoder()),
    ('creatorVestingDuration', getU64Decoder()),
    ('feeDecayDuration', getU64Decoder()),
    ('startFeeRate', getU32Decoder()),
    ('endFeeRate', getU32Decoder()),
    ('creatorFeeShare', getU32Decoder()),
    ('migrationFeeRate', getU32Decoder()),
    ('creatorLpShare', getU32Decoder()),
    ('partnerLpShare', getU32Decoder()),
    ('segmentCount', getU8Decoder()),
    ('baseDecimals', getU8Decoder()),
    ('poolCreatorMode', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'createConfig instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (CreateConfigInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(0)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      CreateConfigInstructionData(
        index: map['index']! as BigInt,
        sqrtStartPrice: map['sqrtStartPrice']! as BigInt,
        curveSqrtPrices: map['curveSqrtPrices']! as List<BigInt>,
        curveLiquidities: map['curveLiquidities']! as List<BigInt>,
        migrationQuoteThreshold: map['migrationQuoteThreshold']! as BigInt,
        totalSupply: map['totalSupply']! as BigInt,
        creatorAllocation: map['creatorAllocation']! as BigInt,
        creatorVestingCliff: map['creatorVestingCliff']! as BigInt,
        creatorVestingDuration: map['creatorVestingDuration']! as BigInt,
        feeDecayDuration: map['feeDecayDuration']! as BigInt,
        startFeeRate: map['startFeeRate']! as int,
        endFeeRate: map['endFeeRate']! as int,
        creatorFeeShare: map['creatorFeeShare']! as int,
        migrationFeeRate: map['migrationFeeRate']! as int,
        creatorLpShare: map['creatorLpShare']! as int,
        partnerLpShare: map['partnerLpShare']! as int,
        segmentCount: map['segmentCount']! as int,
        baseDecimals: map['baseDecimals']! as int,
        poolCreatorMode: map['poolCreatorMode']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<CreateConfigInstructionData>(
        fixedSize: structDecoder.fixedSize,
        read: (bytes, offset) {
          final bytesLength = bytes.length - offset;
          if (bytesLength != structDecoder.fixedSize) {
            throwInvalidByteLength(structDecoder.fixedSize, bytesLength);
          }
          return readTopLevel(bytes, offset);
        },
      ),
    VariableSizeDecoder<Map<String, Object?>>() =>
      VariableSizeDecoder<CreateConfigInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<CreateConfigInstructionData, CreateConfigInstructionData>
getCreateConfigInstructionDataCodec() {
  return combineCodec(
    getCreateConfigInstructionDataEncoder(),
    getCreateConfigInstructionDataDecoder(),
  );
}

/// Creates a [CreateConfig] instruction.
Instruction getCreateConfigInstruction({
  required Address programAddress,
  required Address payer,
  required Address authority,
  required Address config,
  required Address quoteMint,
  required Address quoteTokenProgram,
  required Address ammConfig,
  required Address systemProgram,
  required BigInt index,
  required BigInt sqrtStartPrice,
  required List<BigInt> curveSqrtPrices,
  required List<BigInt> curveLiquidities,
  required BigInt migrationQuoteThreshold,
  required BigInt totalSupply,
  required BigInt creatorAllocation,
  required BigInt creatorVestingCliff,
  required BigInt creatorVestingDuration,
  required BigInt feeDecayDuration,
  required int startFeeRate,
  required int endFeeRate,
  required int creatorFeeShare,
  required int migrationFeeRate,
  required int creatorLpShare,
  required int partnerLpShare,
  required int segmentCount,
  required int baseDecimals,
  required int poolCreatorMode,
}) {
  final instructionData = CreateConfigInstructionData(
    index: index,
    sqrtStartPrice: sqrtStartPrice,
    curveSqrtPrices: curveSqrtPrices,
    curveLiquidities: curveLiquidities,
    migrationQuoteThreshold: migrationQuoteThreshold,
    totalSupply: totalSupply,
    creatorAllocation: creatorAllocation,
    creatorVestingCliff: creatorVestingCliff,
    creatorVestingDuration: creatorVestingDuration,
    feeDecayDuration: feeDecayDuration,
    startFeeRate: startFeeRate,
    endFeeRate: endFeeRate,
    creatorFeeShare: creatorFeeShare,
    migrationFeeRate: migrationFeeRate,
    creatorLpShare: creatorLpShare,
    partnerLpShare: partnerLpShare,
    segmentCount: segmentCount,
    baseDecimals: baseDecimals,
    poolCreatorMode: poolCreatorMode,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: payer, role: AccountRole.writableSigner),
      AccountMeta(address: authority, role: AccountRole.readonlySigner),
      AccountMeta(address: config, role: AccountRole.writable),
      AccountMeta(address: quoteMint, role: AccountRole.readonly),
      AccountMeta(address: quoteTokenProgram, role: AccountRole.readonly),
      AccountMeta(address: ammConfig, role: AccountRole.readonly),
      AccountMeta(address: systemProgram, role: AccountRole.readonly),
    ],
    data: getCreateConfigInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [CreateConfig] instruction from raw instruction data.
CreateConfigInstructionData parseCreateConfigInstruction(
  Instruction instruction,
) {
  return getCreateConfigInstructionDataDecoder().decode(instruction.data!);
}
