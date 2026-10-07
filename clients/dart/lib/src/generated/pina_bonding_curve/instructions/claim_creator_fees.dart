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
class ClaimCreatorFeesInstructionData {
  const ClaimCreatorFeesInstructionData() : discriminator = 6;

  final int discriminator;
}

Encoder<ClaimCreatorFeesInstructionData>
getClaimCreatorFeesInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (ClaimCreatorFeesInstructionData value) => <String, Object?>{
      'discriminator': 6,
    },
  );
}

Decoder<ClaimCreatorFeesInstructionData>
getClaimCreatorFeesInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'claimCreatorFees instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (ClaimCreatorFeesInstructionData, int) readTopLevel(
    Uint8List bytes,
    int offset,
  ) {
    getConstantDecoder(getU8Encoder().encode(6)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (ClaimCreatorFeesInstructionData(), newOffset);
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<ClaimCreatorFeesInstructionData>(
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
      VariableSizeDecoder<ClaimCreatorFeesInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<ClaimCreatorFeesInstructionData, ClaimCreatorFeesInstructionData>
getClaimCreatorFeesInstructionDataCodec() {
  return combineCodec(
    getClaimCreatorFeesInstructionDataEncoder(),
    getClaimCreatorFeesInstructionDataDecoder(),
  );
}

/// Creates a [ClaimCreatorFees] instruction.
Instruction getClaimCreatorFeesInstruction({
  required Address programAddress,
  required Address claimant,
  required Address config,
  required Address launch,
  required Address quoteVault,
  required Address destination,
  required Address quoteTokenProgram,
}) {
  final instructionData = ClaimCreatorFeesInstructionData();

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: claimant, role: AccountRole.readonlySigner),
      AccountMeta(address: config, role: AccountRole.readonly),
      AccountMeta(address: launch, role: AccountRole.writable),
      AccountMeta(address: quoteVault, role: AccountRole.writable),
      AccountMeta(address: destination, role: AccountRole.writable),
      AccountMeta(address: quoteTokenProgram, role: AccountRole.readonly),
    ],
    data: getClaimCreatorFeesInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [ClaimCreatorFees] instruction from raw instruction data.
ClaimCreatorFeesInstructionData parseClaimCreatorFeesInstruction(
  Instruction instruction,
) {
  return getClaimCreatorFeesInstructionDataDecoder().decode(instruction.data!);
}
