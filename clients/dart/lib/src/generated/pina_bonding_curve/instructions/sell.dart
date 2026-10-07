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
class SellInstructionData {
  const SellInstructionData({
    required this.baseAmountIn,
    required this.minimumQuoteOut,
  }) : discriminator = 3;

  final int discriminator;
  final BigInt baseAmountIn;
  final BigInt minimumQuoteOut;
}

Encoder<SellInstructionData> getSellInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('baseAmountIn', getU64Encoder()),
    ('minimumQuoteOut', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (SellInstructionData value) => <String, Object?>{
      'discriminator': 3,
      'baseAmountIn': value.baseAmountIn,
      'minimumQuoteOut': value.minimumQuoteOut,
    },
  );
}

Decoder<SellInstructionData> getSellInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('baseAmountIn', getU64Decoder()),
    ('minimumQuoteOut', getU64Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'sell instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (SellInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(3)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      SellInstructionData(
        baseAmountIn: map['baseAmountIn']! as BigInt,
        minimumQuoteOut: map['minimumQuoteOut']! as BigInt,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<SellInstructionData>(
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
      VariableSizeDecoder<SellInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<SellInstructionData, SellInstructionData> getSellInstructionDataCodec() {
  return combineCodec(
    getSellInstructionDataEncoder(),
    getSellInstructionDataDecoder(),
  );
}

/// Creates a [Sell] instruction.
Instruction getSellInstruction({
  required Address programAddress,
  required Address trader,
  required Address config,
  required Address launch,
  required Address traderBase,
  required Address traderQuote,
  required Address baseVault,
  required Address quoteVault,
  required Address baseTokenProgram,
  required Address quoteTokenProgram,
  required BigInt baseAmountIn,
  required BigInt minimumQuoteOut,
}) {
  final instructionData = SellInstructionData(
    baseAmountIn: baseAmountIn,
    minimumQuoteOut: minimumQuoteOut,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: trader, role: AccountRole.readonlySigner),
      AccountMeta(address: config, role: AccountRole.readonly),
      AccountMeta(address: launch, role: AccountRole.writable),
      AccountMeta(address: traderBase, role: AccountRole.writable),
      AccountMeta(address: traderQuote, role: AccountRole.writable),
      AccountMeta(address: baseVault, role: AccountRole.writable),
      AccountMeta(address: quoteVault, role: AccountRole.writable),
      AccountMeta(address: baseTokenProgram, role: AccountRole.readonly),
      AccountMeta(address: quoteTokenProgram, role: AccountRole.readonly),
    ],
    data: getSellInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [Sell] instruction from raw instruction data.
SellInstructionData parseSellInstruction(Instruction instruction) {
  return getSellInstructionDataDecoder().decode(instruction.data!);
}
