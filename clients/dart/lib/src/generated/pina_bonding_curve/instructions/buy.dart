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
class BuyInstructionData {
  const BuyInstructionData({
    required this.quoteAmountIn,
    required this.minimumBaseOut,
  }) : discriminator = 2;

  final int discriminator;
  final BigInt quoteAmountIn;
  final BigInt minimumBaseOut;
}

Encoder<BuyInstructionData> getBuyInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('quoteAmountIn', getU64Encoder()),
    ('minimumBaseOut', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (BuyInstructionData value) => <String, Object?>{
      'discriminator': 2,
      'quoteAmountIn': value.quoteAmountIn,
      'minimumBaseOut': value.minimumBaseOut,
    },
  );
}

Decoder<BuyInstructionData> getBuyInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('quoteAmountIn', getU64Decoder()),
    ('minimumBaseOut', getU64Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'buy instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (BuyInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(2)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      BuyInstructionData(
        quoteAmountIn: map['quoteAmountIn']! as BigInt,
        minimumBaseOut: map['minimumBaseOut']! as BigInt,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<BuyInstructionData>(
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
      VariableSizeDecoder<BuyInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<BuyInstructionData, BuyInstructionData> getBuyInstructionDataCodec() {
  return combineCodec(
    getBuyInstructionDataEncoder(),
    getBuyInstructionDataDecoder(),
  );
}

/// Creates a [Buy] instruction.
Instruction getBuyInstruction({
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
  required BigInt quoteAmountIn,
  required BigInt minimumBaseOut,
}) {
  final instructionData = BuyInstructionData(
    quoteAmountIn: quoteAmountIn,
    minimumBaseOut: minimumBaseOut,
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
    data: getBuyInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [Buy] instruction from raw instruction data.
BuyInstructionData parseBuyInstruction(Instruction instruction) {
  return getBuyInstructionDataDecoder().decode(instruction.data!);
}
