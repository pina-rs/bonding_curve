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
class CreateLaunchInstructionData {
  const CreateLaunchInstructionData({required this.activationTime})
    : discriminator = 1;

  final int discriminator;
  final BigInt activationTime;
}

Encoder<CreateLaunchInstructionData> getCreateLaunchInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('activationTime', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (CreateLaunchInstructionData value) => <String, Object?>{
      'discriminator': 1,
      'activationTime': value.activationTime,
    },
  );
}

Decoder<CreateLaunchInstructionData> getCreateLaunchInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('activationTime', getU64Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'createLaunch instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (CreateLaunchInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(1)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      CreateLaunchInstructionData(
        activationTime: map['activationTime']! as BigInt,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<CreateLaunchInstructionData>(
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
      VariableSizeDecoder<CreateLaunchInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<CreateLaunchInstructionData, CreateLaunchInstructionData>
getCreateLaunchInstructionDataCodec() {
  return combineCodec(
    getCreateLaunchInstructionDataEncoder(),
    getCreateLaunchInstructionDataDecoder(),
  );
}

/// Creates a [CreateLaunch] instruction.
Instruction getCreateLaunchInstruction({
  required Address programAddress,
  required Address payer,
  required Address creator,
  required Address config,
  required Address baseMint,
  required Address quoteMint,
  required Address launch,
  required Address baseVault,
  required Address quoteVault,
  required Address baseTokenProgram,
  required Address quoteTokenProgram,
  required Address systemProgram,
  required BigInt activationTime,
}) {
  final instructionData = CreateLaunchInstructionData(
    activationTime: activationTime,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: payer, role: AccountRole.writableSigner),
      AccountMeta(address: creator, role: AccountRole.readonlySigner),
      AccountMeta(address: config, role: AccountRole.readonly),
      AccountMeta(address: baseMint, role: AccountRole.writable),
      AccountMeta(address: quoteMint, role: AccountRole.readonly),
      AccountMeta(address: launch, role: AccountRole.writable),
      AccountMeta(address: baseVault, role: AccountRole.writable),
      AccountMeta(address: quoteVault, role: AccountRole.writable),
      AccountMeta(address: baseTokenProgram, role: AccountRole.readonly),
      AccountMeta(address: quoteTokenProgram, role: AccountRole.readonly),
      AccountMeta(address: systemProgram, role: AccountRole.readonly),
    ],
    data: getCreateLaunchInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [CreateLaunch] instruction from raw instruction data.
CreateLaunchInstructionData parseCreateLaunchInstruction(
  Instruction instruction,
) {
  return getCreateLaunchInstructionDataDecoder().decode(instruction.data!);
}
