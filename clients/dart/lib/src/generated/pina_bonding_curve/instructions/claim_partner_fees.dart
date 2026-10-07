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
class ClaimPartnerFeesInstructionData {
  const ClaimPartnerFeesInstructionData() : discriminator = 5;

  final int discriminator;
}

Encoder<ClaimPartnerFeesInstructionData>
getClaimPartnerFeesInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (ClaimPartnerFeesInstructionData value) => <String, Object?>{
      'discriminator': 5,
    },
  );
}

Decoder<ClaimPartnerFeesInstructionData>
getClaimPartnerFeesInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'claimPartnerFees instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (ClaimPartnerFeesInstructionData, int) readTopLevel(
    Uint8List bytes,
    int offset,
  ) {
    getConstantDecoder(getU8Encoder().encode(5)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (ClaimPartnerFeesInstructionData(), newOffset);
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<ClaimPartnerFeesInstructionData>(
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
      VariableSizeDecoder<ClaimPartnerFeesInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<ClaimPartnerFeesInstructionData, ClaimPartnerFeesInstructionData>
getClaimPartnerFeesInstructionDataCodec() {
  return combineCodec(
    getClaimPartnerFeesInstructionDataEncoder(),
    getClaimPartnerFeesInstructionDataDecoder(),
  );
}

/// Creates a [ClaimPartnerFees] instruction.
Instruction getClaimPartnerFeesInstruction({
  required Address programAddress,
  required Address claimant,
  required Address config,
  required Address launch,
  required Address quoteVault,
  required Address destination,
  required Address quoteTokenProgram,
}) {
  final instructionData = ClaimPartnerFeesInstructionData();

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
    data: getClaimPartnerFeesInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [ClaimPartnerFees] instruction from raw instruction data.
ClaimPartnerFeesInstructionData parseClaimPartnerFeesInstruction(
  Instruction instruction,
) {
  return getClaimPartnerFeesInstructionDataDecoder().decode(instruction.data!);
}
