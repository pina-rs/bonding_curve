// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_core/solana_kit_codecs_core.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';
import 'package:solana_kit_errors/solana_kit_errors.dart';
import 'package:solana_kit_instructions/solana_kit_instructions.dart';

import '../instructions/instructions.dart';

/// The address of the PinaBondingCurve program.
const pinaBondingCurveProgramAddress = Address(
  'CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9',
);

/// Known accounts for the PinaBondingCurve program.
enum PinaBondingCurveAccount { launchConfig, launch }

/// Known instructions for the PinaBondingCurve program.
enum PinaBondingCurveInstruction {
  createConfig,
  createLaunch,
  buy,
  sell,
  graduate,
  claimPartnerFees,
  claimCreatorFees,
  claimCreatorAllocation,
  setLaunchCreator,
}

/// Identifies the type of a PinaBondingCurve instruction.
PinaBondingCurveInstruction identifyPinaBondingCurveInstruction(
  Uint8List data,
) {
  if (containsBytes(data, getU8Encoder().encode(0), 0)) {
    return PinaBondingCurveInstruction.createConfig;
  }
  if (containsBytes(data, getU8Encoder().encode(1), 0)) {
    return PinaBondingCurveInstruction.createLaunch;
  }
  if (containsBytes(data, getU8Encoder().encode(2), 0)) {
    return PinaBondingCurveInstruction.buy;
  }
  if (containsBytes(data, getU8Encoder().encode(3), 0)) {
    return PinaBondingCurveInstruction.sell;
  }
  if (containsBytes(data, getU8Encoder().encode(4), 0)) {
    return PinaBondingCurveInstruction.graduate;
  }
  if (containsBytes(data, getU8Encoder().encode(5), 0)) {
    return PinaBondingCurveInstruction.claimPartnerFees;
  }
  if (containsBytes(data, getU8Encoder().encode(6), 0)) {
    return PinaBondingCurveInstruction.claimCreatorFees;
  }
  if (containsBytes(data, getU8Encoder().encode(7), 0)) {
    return PinaBondingCurveInstruction.claimCreatorAllocation;
  }
  if (containsBytes(data, getU8Encoder().encode(8), 0)) {
    return PinaBondingCurveInstruction.setLaunchCreator;
  }

  throw SolanaError(SolanaErrorCode.programClientsFailedToIdentifyInstruction, {
    'instructionData': data,
    'programName': 'pinaBondingCurve',
  });
}

/// A parsed instruction from the PinaBondingCurve program.
sealed class ParsedPinaBondingCurveInstruction {
  const ParsedPinaBondingCurveInstruction(this.instructionType);

  final PinaBondingCurveInstruction instructionType;
}

/// A parsed CreateConfig instruction.
final class ParsedCreateConfig extends ParsedPinaBondingCurveInstruction {
  const ParsedCreateConfig({required this.data})
    : super(PinaBondingCurveInstruction.createConfig);

  final CreateConfigInstructionData data;
}

/// A parsed CreateLaunch instruction.
final class ParsedCreateLaunch extends ParsedPinaBondingCurveInstruction {
  const ParsedCreateLaunch({required this.data})
    : super(PinaBondingCurveInstruction.createLaunch);

  final CreateLaunchInstructionData data;
}

/// A parsed Buy instruction.
final class ParsedBuy extends ParsedPinaBondingCurveInstruction {
  const ParsedBuy({required this.data})
    : super(PinaBondingCurveInstruction.buy);

  final BuyInstructionData data;
}

/// A parsed Sell instruction.
final class ParsedSell extends ParsedPinaBondingCurveInstruction {
  const ParsedSell({required this.data})
    : super(PinaBondingCurveInstruction.sell);

  final SellInstructionData data;
}

/// A parsed Graduate instruction.
final class ParsedGraduate extends ParsedPinaBondingCurveInstruction {
  const ParsedGraduate({required this.data})
    : super(PinaBondingCurveInstruction.graduate);

  final GraduateInstructionData data;
}

/// A parsed ClaimPartnerFees instruction.
final class ParsedClaimPartnerFees extends ParsedPinaBondingCurveInstruction {
  const ParsedClaimPartnerFees({required this.data})
    : super(PinaBondingCurveInstruction.claimPartnerFees);

  final ClaimPartnerFeesInstructionData data;
}

/// A parsed ClaimCreatorFees instruction.
final class ParsedClaimCreatorFees extends ParsedPinaBondingCurveInstruction {
  const ParsedClaimCreatorFees({required this.data})
    : super(PinaBondingCurveInstruction.claimCreatorFees);

  final ClaimCreatorFeesInstructionData data;
}

/// A parsed ClaimCreatorAllocation instruction.
final class ParsedClaimCreatorAllocation
    extends ParsedPinaBondingCurveInstruction {
  const ParsedClaimCreatorAllocation({required this.data})
    : super(PinaBondingCurveInstruction.claimCreatorAllocation);

  final ClaimCreatorAllocationInstructionData data;
}

/// A parsed SetLaunchCreator instruction.
final class ParsedSetLaunchCreator extends ParsedPinaBondingCurveInstruction {
  const ParsedSetLaunchCreator({required this.data})
    : super(PinaBondingCurveInstruction.setLaunchCreator);

  final SetLaunchCreatorInstructionData data;
}

/// Parses a PinaBondingCurve instruction.
ParsedPinaBondingCurveInstruction parsePinaBondingCurveInstruction(
  Instruction instruction,
) {
  return switch (identifyPinaBondingCurveInstruction(
    instruction.data ?? Uint8List(0),
  )) {
    PinaBondingCurveInstruction.createConfig => ParsedCreateConfig(
      data: parseCreateConfigInstruction(instruction),
    ),
    PinaBondingCurveInstruction.createLaunch => ParsedCreateLaunch(
      data: parseCreateLaunchInstruction(instruction),
    ),
    PinaBondingCurveInstruction.buy => ParsedBuy(
      data: parseBuyInstruction(instruction),
    ),
    PinaBondingCurveInstruction.sell => ParsedSell(
      data: parseSellInstruction(instruction),
    ),
    PinaBondingCurveInstruction.graduate => ParsedGraduate(
      data: parseGraduateInstruction(instruction),
    ),
    PinaBondingCurveInstruction.claimPartnerFees => ParsedClaimPartnerFees(
      data: parseClaimPartnerFeesInstruction(instruction),
    ),
    PinaBondingCurveInstruction.claimCreatorFees => ParsedClaimCreatorFees(
      data: parseClaimCreatorFeesInstruction(instruction),
    ),
    PinaBondingCurveInstruction.claimCreatorAllocation =>
      ParsedClaimCreatorAllocation(
        data: parseClaimCreatorAllocationInstruction(instruction),
      ),
    PinaBondingCurveInstruction.setLaunchCreator => ParsedSetLaunchCreator(
      data: parseSetLaunchCreatorInstruction(instruction),
    ),
  };
}
