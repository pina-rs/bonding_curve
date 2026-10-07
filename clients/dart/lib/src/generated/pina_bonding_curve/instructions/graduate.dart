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
class GraduateInstructionData {
  const GraduateInstructionData() : discriminator = 4;

  final int discriminator;
}

Encoder<GraduateInstructionData> getGraduateInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (GraduateInstructionData value) => <String, Object?>{'discriminator': 4},
  );
}

Decoder<GraduateInstructionData> getGraduateInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'graduate instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (GraduateInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(4)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (GraduateInstructionData(), newOffset);
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<GraduateInstructionData>(
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
      VariableSizeDecoder<GraduateInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<GraduateInstructionData, GraduateInstructionData>
getGraduateInstructionDataCodec() {
  return combineCodec(
    getGraduateInstructionDataEncoder(),
    getGraduateInstructionDataDecoder(),
  );
}

/// Creates a [Graduate] instruction.
Instruction getGraduateInstruction({
  required Address programAddress,
  required Address payer,
  required Address config,
  required Address launch,
  required Address baseMint,
  required Address quoteMint,
  required Address baseVault,
  required Address quoteVault,
  required Address ammAuthority,
  required Address ammProgram,
  required Address ammConfig,
  required Address pool,
  required Address lpMint,
  required Address poolVault0,
  required Address poolVault1,
  required Address launchLpToken,
  required Address baseTokenProgram,
  required Address quoteTokenProgram,
  required Address lpTokenProgram,
  required Address associatedTokenProgram,
  required Address systemProgram,
  Address? creator,
  Address? creatorLpToken,
  Address? partner,
  Address? partnerLpToken,
}) {
  final instructionData = GraduateInstructionData();

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: payer, role: AccountRole.writableSigner),
      AccountMeta(address: config, role: AccountRole.readonly),
      AccountMeta(address: launch, role: AccountRole.writable),
      AccountMeta(address: baseMint, role: AccountRole.writable),
      AccountMeta(address: quoteMint, role: AccountRole.readonly),
      AccountMeta(address: baseVault, role: AccountRole.writable),
      AccountMeta(address: quoteVault, role: AccountRole.writable),
      AccountMeta(address: ammAuthority, role: AccountRole.readonly),
      AccountMeta(address: ammProgram, role: AccountRole.readonly),
      AccountMeta(address: ammConfig, role: AccountRole.readonly),
      AccountMeta(address: pool, role: AccountRole.writable),
      AccountMeta(address: lpMint, role: AccountRole.writable),
      AccountMeta(address: poolVault0, role: AccountRole.writable),
      AccountMeta(address: poolVault1, role: AccountRole.writable),
      AccountMeta(address: launchLpToken, role: AccountRole.writable),
      AccountMeta(address: baseTokenProgram, role: AccountRole.readonly),
      AccountMeta(address: quoteTokenProgram, role: AccountRole.readonly),
      AccountMeta(address: lpTokenProgram, role: AccountRole.readonly),
      AccountMeta(address: associatedTokenProgram, role: AccountRole.readonly),
      AccountMeta(address: systemProgram, role: AccountRole.readonly),
      if (creator != null)
        AccountMeta(address: creator, role: AccountRole.readonly)
      else
        AccountMeta(address: programAddress, role: AccountRole.readonly),
      if (creatorLpToken != null)
        AccountMeta(address: creatorLpToken, role: AccountRole.writable)
      else
        AccountMeta(address: programAddress, role: AccountRole.readonly),
      if (partner != null)
        AccountMeta(address: partner, role: AccountRole.readonly)
      else
        AccountMeta(address: programAddress, role: AccountRole.readonly),
      if (partnerLpToken != null)
        AccountMeta(address: partnerLpToken, role: AccountRole.writable)
      else
        AccountMeta(address: programAddress, role: AccountRole.readonly),
    ],
    data: getGraduateInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [Graduate] instruction from raw instruction data.
GraduateInstructionData parseGraduateInstruction(Instruction instruction) {
  return getGraduateInstructionDataDecoder().decode(instruction.data!);
}
