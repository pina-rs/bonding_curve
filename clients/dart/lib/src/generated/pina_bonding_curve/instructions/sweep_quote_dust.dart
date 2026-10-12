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
class SweepQuoteDustInstructionData {
  const SweepQuoteDustInstructionData() : discriminator = 9;

  final int discriminator;
}

Encoder<SweepQuoteDustInstructionData>
getSweepQuoteDustInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (SweepQuoteDustInstructionData value) => <String, Object?>{
      'discriminator': 9,
    },
  );
}

Decoder<SweepQuoteDustInstructionData>
getSweepQuoteDustInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'sweepQuoteDust instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (SweepQuoteDustInstructionData, int) readTopLevel(
    Uint8List bytes,
    int offset,
  ) {
    getConstantDecoder(getU8Encoder().encode(9)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (SweepQuoteDustInstructionData(), newOffset);
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<SweepQuoteDustInstructionData>(
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
      VariableSizeDecoder<SweepQuoteDustInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<SweepQuoteDustInstructionData, SweepQuoteDustInstructionData>
getSweepQuoteDustInstructionDataCodec() {
  return combineCodec(
    getSweepQuoteDustInstructionDataEncoder(),
    getSweepQuoteDustInstructionDataDecoder(),
  );
}

/// Creates a [SweepQuoteDust] instruction.
Instruction getSweepQuoteDustInstruction({
  required Address programAddress,
  required Address config,
  required Address launch,
  required Address quoteVault,
  required Address quoteTokenProgram,
}) {
  final instructionData = SweepQuoteDustInstructionData();

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: config, role: AccountRole.readonly),
      AccountMeta(address: launch, role: AccountRole.writable),
      AccountMeta(address: quoteVault, role: AccountRole.readonly),
      AccountMeta(address: quoteTokenProgram, role: AccountRole.readonly),
    ],
    data: getSweepQuoteDustInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [SweepQuoteDust] instruction from raw instruction data.
SweepQuoteDustInstructionData parseSweepQuoteDustInstruction(
  Instruction instruction,
) {
  return getSweepQuoteDustInstructionDataDecoder().decode(instruction.data!);
}
