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
class SetLaunchCreatorInstructionData {
  const SetLaunchCreatorInstructionData({required this.newCreator})
    : discriminator = 8;

  final int discriminator;
  final Address newCreator;
}

Encoder<SetLaunchCreatorInstructionData>
getSetLaunchCreatorInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('newCreator', getAddressEncoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (SetLaunchCreatorInstructionData value) => <String, Object?>{
      'discriminator': 8,
      'newCreator': value.newCreator,
    },
  );
}

Decoder<SetLaunchCreatorInstructionData>
getSetLaunchCreatorInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('newCreator', getAddressDecoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'setLaunchCreator instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (SetLaunchCreatorInstructionData, int) readTopLevel(
    Uint8List bytes,
    int offset,
  ) {
    getConstantDecoder(getU8Encoder().encode(8)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      SetLaunchCreatorInstructionData(
        newCreator: map['newCreator']! as Address,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<SetLaunchCreatorInstructionData>(
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
      VariableSizeDecoder<SetLaunchCreatorInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<SetLaunchCreatorInstructionData, SetLaunchCreatorInstructionData>
getSetLaunchCreatorInstructionDataCodec() {
  return combineCodec(
    getSetLaunchCreatorInstructionDataEncoder(),
    getSetLaunchCreatorInstructionDataDecoder(),
  );
}

/// Creates a [SetLaunchCreator] instruction.
Instruction getSetLaunchCreatorInstruction({
  required Address programAddress,
  required Address creator,
  required Address launch,
  required Address newCreator,
}) {
  final instructionData = SetLaunchCreatorInstructionData(
    newCreator: newCreator,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: creator, role: AccountRole.readonlySigner),
      AccountMeta(address: launch, role: AccountRole.writable),
    ],
    data: getSetLaunchCreatorInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [SetLaunchCreator] instruction from raw instruction data.
SetLaunchCreatorInstructionData parseSetLaunchCreatorInstruction(
  Instruction instruction,
) {
  return getSetLaunchCreatorInstructionDataDecoder().decode(instruction.data!);
}
